// This file is part of Hyperspace.
//
// Copyright (C) 2018-2021 Hyperspace Network
// SPDX-License-Identifier: GPL-3.0
//
// Hyperspace is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Hyperspace is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Hyperspace. If not, see <https://www.gnu.org/licenses/>.

//! Module to process claims from Ethereum addresses.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod address;
#[cfg(feature = "std")]
pub use address::*;

mod types {
	// --- hyperspace ---
	use crate::*;

	pub type AddressT = [u8; 20];

	pub type EtpBalance<T> = <EtpCurrency<T> as Currency<AccountId<T>>>::Balance;

	type AccountId<T> = <T as frame_system::Config>::AccountId;
	type EtpCurrency<T> = <T as Config>::EtpCurrency;
}

// --- crates ---
use codec::{Decode, Encode};
// --- substrate ---
#[cfg(feature = "std")]
use frame_support::{debug::error, traits::WithdrawReasons};
use frame_support::{
	ensure,
	traits::{Currency, EnsureOrigin, ExistenceRequirement::KeepAlive, Get},
	weights::{DispatchClass, Pays},
	{decl_error, decl_event, decl_module, decl_storage},
};
use frame_system::{ensure_none, ensure_root};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
#[cfg(feature = "std")]
use sp_runtime::traits::{SaturatedConversion, Zero};
use sp_runtime::{
	traits::AccountIdConversion,
	transaction_validity::{
		InvalidTransaction, TransactionLongevity, TransactionSource, TransactionValidity,
		ValidTransaction,
	},
	ModuleId, RuntimeDebug,
};
use sp_std::prelude::*;
// --- hyperspace ---
use hyperspace_support::balance::lock::*;
use types::*;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	type ModuleId: Get<ModuleId>;

	type Prefix: Get<&'static [u8]>;

	/// The *ETP* currency.
	type EtpCurrency: LockableCurrency<Self::AccountId>;

	type MoveClaimOrigin: EnsureOrigin<Self::Origin>;
}

decl_event!(
	pub enum Event<T>
	where
		<T as frame_system::Config>::AccountId,
		EtpBalance = EtpBalance<T>,
	{
		/// Someone claimed some *ETP*s. [account, address, amount]
		Claimed(AccountId, AddressT, EtpBalance),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Invalid Ethereum signature.
		InvalidSignature,
		/// Ethereum address has no claim.
		SignerHasNoClaim,
		/// There's not enough in the pot to pay out some unvested amount. Generally implies a logic
		/// error.
		PotUnderflow,
		/// Can NOT Move Claim to an EXISTED Address.
		MoveToExistedAddress,
		/// New Address Type - MISMATCHED
		NewAddressTypeMis,
	}
}

decl_storage! {
	trait Store for Module<T: Config> as HyperspaceClaims {
		ClaimsFromEth
			get(fn claims_from_eth)
			: map hasher(identity) AddressT => Option<EtpBalance<T>>;
		ClaimsFromOldetp
			get(fn claims_from_oldetp)
			: map hasher(identity) AddressT => Option<EtpBalance<T>>;
	}
	add_extra_genesis {
		config(claims_list): ClaimsList;
		build(|config| {
			let ClaimsList {
				dot,
				eth,
				oldetp,
			} = &config.claims_list;
			let mut total = <EtpBalance<T>>::zero();

			if dot.is_empty() && eth.is_empty() && oldetp.is_empty() {
				error!("[hyperspace-claims] Genesis Claims List is Set to EMPTY");
			} else {
				// Eth Address
				for Account { address, backed_etp } in dot {
					// DOT:ETP = 1:50
					let backed_etp = (*backed_etp).saturated_into();
					<ClaimsFromEth<T>>::insert(address.0, backed_etp);
					total += backed_etp;
				}
				for Account { address, backed_etp } in eth {
					let backed_etp = (*backed_etp).saturated_into();
					<ClaimsFromEth<T>>::insert(address.0, backed_etp);
					total += backed_etp;
				}

				// Oldetp Address
				for Account { address, backed_etp } in oldetp {
					let backed_etp = (*backed_etp).saturated_into();
					<ClaimsFromOldetp<T>>::insert(address.0, backed_etp);
					total += backed_etp;
				}
			}

			let minimum_balance = T::EtpCurrency::minimum_balance();
			let _ = T::EtpCurrency::make_free_balance_be(
				&<Module<T>>::account_id(),
				total + minimum_balance,
			);
			T::EtpCurrency::set_lock(
				T::ModuleId::get().0,
				&<Module<T>>::account_id(),
				LockFor::Common { amount: minimum_balance },
				WithdrawReasons::all(),
			);
		});
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		const ModuleId: ModuleId = T::ModuleId::get();

		/// The Prefix that is used in signed Ethereum messages for this network
		const Prefix: &[u8] = T::Prefix::get();

		/// Deposit one of this module's events by using the default implementation.
		fn deposit_event() = default;

		/// Make a claim to collect your DOTs.
		///
		/// The dispatch origin for this call must be _None_.
		///
		/// Unsigned Validation:
		/// A call to claim is deemed valid if the signature provided matches
		/// the expected signed message of:
		///
		/// > Ethereum Signed Message:
		/// > (configured prefix string)(address)
		///
		/// and `address` matches the `dest` account.
		///
		/// Parameters:
		/// - `dest`: The destination account to payout the claim.
		/// - `ethereum_signature`: The signature of an ethereum signed message
		///    matching the format described above.
		///
		/// <weight>
		/// The weight of this call is invariant over the input parameters.
		/// - One `eth_recover` operation which involves a keccak hash and a
		///   ecdsa recover.
		/// - Three storage reads to check if a claim exists for the user, to
		///   get the current pot size, to see if there exists a vesting schedule.
		/// - Up to one storage write for adding a new vesting schedule.
		/// - One `deposit_creating` Currency call.
		/// - One storage write to update the total.
		/// - Two storage removals for vesting and claims information.
		/// - One deposit event.
		///
		/// Total Complexity: O(1)
		/// ----------------------------
		/// Base Weight: 269.7 µs
		/// DB Weight:
		/// - Read: Claims
		/// - Write: Account, Claims
		/// Validate Unsigned: +188.7 µs
		/// </weight>
		#[weight = T::DbWeight::get().reads_writes(1, 2) + 270_000_000 + 190_000_000]
		fn claim(origin, dest: T::AccountId, signature: OtherSignature) {
			ensure_none(origin)?;

			let data = dest.using_encoded(to_ascii_hex);

			match signature {
				OtherSignature::Eth(signature) => {
					let signer = Self::eth_recover(&signature, &data)
						.ok_or(<Error<T>>::InvalidSignature)?;
					let balance_due = <ClaimsFromEth<T>>::get(&signer)
						.ok_or(<Error<T>>::SignerHasNoClaim)?;

					ensure!(
						Self::pot::<T::EtpCurrency>() >= balance_due,
						<Error<T>>::PotUnderflow,
					);
					T::EtpCurrency::transfer(
						&Self::account_id(),
						&dest,
						balance_due,
						KeepAlive,
					)?;

					<ClaimsFromEth<T>>::remove(&signer);

					Self::deposit_event(RawEvent::Claimed(dest, signer, balance_due));
				}
				OtherSignature::Oldetp(signature) => {
					let signer = Self::oldetp_recover(&signature, &data)
						.ok_or(<Error<T>>::InvalidSignature)?;
					let balance_due = <ClaimsFromOldetp<T>>::get(&signer)
						.ok_or(<Error<T>>::SignerHasNoClaim)?;

					ensure!(
						Self::pot::<T::EtpCurrency>() >= balance_due,
						<Error<T>>::PotUnderflow,
					);
					T::EtpCurrency::transfer(
						&Self::account_id(),
						&dest,
						balance_due,
						KeepAlive,
					)?;

					<ClaimsFromOldetp<T>>::remove(&signer);

					Self::deposit_event(RawEvent::Claimed(dest, signer, balance_due));
				}
			}
		}

		/// Mint a new claim to collect DOTs.
		///
		/// The dispatch origin for this call must be _Root_.
		///
		/// Parameters:
		/// - `who`: The Ethereum address allowed to collect this claim.
		/// - `value`: The number of DOTs that will be claimed.
		/// - `vesting_schedule`: An optional vesting schedule for these DOTs.
		///
		/// <weight>
		/// The weight of this call is invariant over the input parameters.
		/// - One storage mutate to increase the total claims available.
		/// - One storage write to add a new claim.
		/// - Up to one storage write to add a new vesting schedule.
		///
		/// Total Complexity: O(1)
		/// ---------------------
		/// Base Weight: 10.46 µs
		/// DB Weight:
		/// - Reads:
		/// - Writes: Account, Claims
		/// - Maybe Write: Vesting, Statement
		/// </weight>
		#[weight =
			T::DbWeight::get().reads_writes(0, 2)
			+ 10_000_000
		]
		fn mint_claim(origin, who: OtherAddress, value: EtpBalance<T>) {
			ensure_root(origin)?;

			match who {
				OtherAddress::Eth(who) => {
					T::EtpCurrency::deposit_creating(&Self::account_id(), value);
					<ClaimsFromEth<T>>::insert(who, value);
				}
				OtherAddress::Oldetp(who) => {
					T::EtpCurrency::deposit_creating(&Self::account_id(), value);
					<ClaimsFromOldetp<T>>::insert(who, value);
				}
			}
		}

		#[weight = (
			T::DbWeight::get().reads_writes(4, 4) + 100_000_000_000,
			DispatchClass::Normal,
			Pays::No
		)]
		fn move_claim(origin,
			old: OtherAddress,
			new: OtherAddress,
		) {
			T::MoveClaimOrigin::try_origin(origin).map(|_| ()).or_else(ensure_root)?;

			match old {
				OtherAddress::Eth(old) => if let OtherAddress::Eth(new) = new {
					ensure!(
						!<ClaimsFromEth<T>>::contains_key(&new),
						<Error<T>>::MoveToExistedAddress
					);

					<ClaimsFromEth<T>>::take(&old).map(|c| <ClaimsFromEth<T>>::insert(&new, c));
				} else {
					Err(<Error<T>>::NewAddressTypeMis)?;
				},
				OtherAddress::Oldetp(old) => if let OtherAddress::Oldetp(new) = new {
					ensure!(
						!<ClaimsFromOldetp<T>>::contains_key(&new),
						<Error<T>>::MoveToExistedAddress
					);

					<ClaimsFromOldetp<T>>::take(&old).map(|c| <ClaimsFromOldetp<T>>::insert(&new, c));
				} else {
					Err(<Error<T>>::NewAddressTypeMis)?;
				}
			}
		}
	}
}

impl<T: Config> Module<T> {
	fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}

	fn pot<C: LockableCurrency<T::AccountId>>() -> C::Balance {
		// Already lock minimal balance in the account, no need to worry about to be 0.
		C::usable_balance(&Self::account_id())
	}

	// Constructs the message that RPC's `personal_sign` and `sign` would sign.
	fn eth_signable_message(what: &[u8], signed_message: &[u8]) -> Vec<u8> {
		let prefix = T::Prefix::get();
		let mut l = prefix.len() + what.len();
		let mut rev = Vec::new();
		while l > 0 {
			rev.push(b'0' + (l % 10) as u8);
			l /= 10;
		}
		let mut v = signed_message.to_vec();
		v.extend(rev.into_iter().rev());
		v.extend_from_slice(&prefix[..]);
		v.extend_from_slice(what);
		v
	}

	// Constructs the message that RPC's `personal_sign` and `sign` would sign.
	// Oldetp have different signing specs: https://github.com/oldetpprotocol/tips/issues/104
	fn oldetp_signable_message(what: &[u8], signed_message: &[u8]) -> Vec<u8> {
		let prefix = T::Prefix::get();
		let mut l = 32;
		let mut rev = Vec::new();
		while l > 0 {
			rev.push(b'0' + (l % 10) as u8);
			l /= 10;
		}
		let mut v = signed_message.to_vec();
		v.extend(rev.into_iter().rev());
		v.extend_from_slice(&prefix[..]);
		v.extend_from_slice(what);
		v
	}

	// Attempts to recover the Ethereum address from a message signature signed by using
	// the Ethereum RPC's `personal_sign` and `eth_sign`.
	fn eth_recover(s: &EcdsaSignature, what: &[u8]) -> Option<AddressT> {
		let msg = keccak_256(&Self::eth_signable_message(
			what,
			b"\x19Ethereum Signed Message:\n",
		));
		let mut res = AddressT::default();
		res.copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
		Some(res)
	}

	// Attempts to recover the Oldetp address from a message signature signed by using
	// the Oldetp RPC's `personal_sign` and `oldetp_sign`.
	fn oldetp_recover(s: &EcdsaSignature, what: &[u8]) -> Option<AddressT> {
		let msg = keccak_256(&Self::oldetp_signable_message(
			what,
			b"\x19OLDETP Signed Message:\n",
		));
		let mut res = AddressT::default();
		res.copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
		Some(res)
	}
}

impl<T: Config> sp_runtime::traits::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		const PRIORITY: u64 = 100;

		match call {
			Call::claim(account, signature) => {
				let data = account.using_encoded(to_ascii_hex);

				match signature {
					OtherSignature::Eth(signature) => {
						let maybe_signer = Self::eth_recover(&signature, &data);
						let signer = if let Some(s) = maybe_signer {
							s
						} else {
							return InvalidTransaction::Custom(
								ValidityError::InvalidSignature as _,
							)
							.into();
						};

						if !<ClaimsFromEth<T>>::contains_key(&signer) {
							return Err(InvalidTransaction::Custom(
								ValidityError::SignerHasNoClaim as _,
							)
							.into());
						}

						Ok(ValidTransaction {
							priority: PRIORITY,
							requires: vec![],
							provides: vec![("claims", signer).encode()],
							longevity: TransactionLongevity::max_value(),
							propagate: true,
						})
					}
					OtherSignature::Oldetp(signature) => {
						let maybe_signer = Self::oldetp_recover(&signature, &data);
						let signer = if let Some(s) = maybe_signer {
							s
						} else {
							return InvalidTransaction::Custom(
								ValidityError::InvalidSignature as _,
							)
							.into();
						};

						if !<ClaimsFromOldetp<T>>::contains_key(&signer) {
							return Err(InvalidTransaction::Custom(
								ValidityError::SignerHasNoClaim as _,
							)
							.into());
						}

						Ok(ValidTransaction {
							priority: PRIORITY,
							requires: vec![],
							provides: vec![("claims", signer).encode()],
							longevity: TransactionLongevity::max_value(),
							propagate: true,
						})
					}
				}
			}
			_ => Err(InvalidTransaction::Call.into()),
		}
	}
}

#[repr(u8)]
enum ValidityError {
	/// The signature is invalid.
	InvalidSignature = 0,
	/// The signer has no claim.
	SignerHasNoClaim = 1,
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum OtherSignature {
	Eth(EcdsaSignature),
	Oldetp(EcdsaSignature),
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum OtherAddress {
	Eth(AddressT),
	Oldetp(AddressT),
}

#[derive(Clone, Encode, Decode)]
pub struct EcdsaSignature(pub [u8; 65]);
impl PartialEq for EcdsaSignature {
	fn eq(&self, other: &Self) -> bool {
		&self.0[..] == &other.0[..]
	}
}
impl sp_std::fmt::Debug for EcdsaSignature {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		write!(f, "EcdsaSignature({:?})", &self.0[..])
	}
}

/// Converts the given binary data into ASCII-encoded hex. It will be twice the length.
fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
	let mut r = Vec::with_capacity(data.len() * 2);
	let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
	for &b in data.iter() {
		push_nibble(b / 16);
		push_nibble(b % 16);
	}
	r
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod secp_utils {
	// --- crates ---
	use sp_io::hashing::keccak_256;
	// --- custom ---
	use crate::*;

	pub fn public(secret: &secp256k1::SecretKey) -> secp256k1::PublicKey {
		secp256k1::PublicKey::from_secret_key(secret)
	}

	pub fn addr(secret: &secp256k1::SecretKey) -> AddressT {
		let mut res = AddressT::default();
		res.copy_from_slice(&keccak_256(&public(secret).serialize()[1..65])[12..]);
		res
	}

	pub fn eth_sig<T: Config>(
		secret: &secp256k1::SecretKey,
		what: &[u8],
		signed_message: &[u8],
	) -> EcdsaSignature {
		let msg = keccak_256(&<super::Module<T>>::eth_signable_message(
			&to_ascii_hex(what)[..],
			signed_message,
		));
		let (sig, recovery_id) = secp256k1::sign(&secp256k1::Message::parse(&msg), secret);
		let mut r = [0u8; 65];
		r[0..64].copy_from_slice(&sig.serialize()[..]);
		r[64] = recovery_id.serialize();
		EcdsaSignature(r)
	}

	pub fn oldetp_sig<T: Config>(
		secret: &secp256k1::SecretKey,
		what: &[u8],
		signed_message: &[u8],
	) -> EcdsaSignature {
		let msg = keccak_256(&<super::Module<T>>::oldetp_signable_message(
			&to_ascii_hex(what)[..],
			signed_message,
		));
		let (sig, recovery_id) = secp256k1::sign(&secp256k1::Message::parse(&msg), secret);
		let mut r = [0u8; 65];
		r[0..64].copy_from_slice(&sig.serialize()[..]);
		r[64] = recovery_id.serialize();
		EcdsaSignature(r)
	}
}

#[cfg(test)]
mod tests {
	// --- crates ---
	use codec::Encode;
	// --- substrate ---
	use frame_support::{
		assert_err, assert_noop, assert_ok, dispatch::DispatchError::BadOrigin,
		ord_parameter_types, parameter_types,
	};
	use frame_system::mocking::*;
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
	};
	// --- hyperspace ---
	use crate::{self as hyperspace_claims, secp_utils::*, *};
	

	type Balance = u64;

	type Block = MockBlock<Test>;
	type UncheckedExtrinsic = MockUncheckedExtrinsic<Test>;

	const ETHEREUM_SIGNED_MESSAGE: &'static [u8] = b"\x19Ethereum Signed Message:\n";
	const OLDETP_SIGNED_MESSAGE: &'static [u8] = b"\x19OLDETP Signed Message:\n";

	hyperspace_support::impl_test_account_data! {}

	impl frame_system::Config for Test {
		type BaseCallFilter = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = ();
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
	}

	parameter_types! {
		pub const ExistentialDeposit: Balance = 1;
	}
	impl hyperspace_balances::Config<EtpInstance> for Test {
		type Balance = Balance;
		type DustRemoval = ();
		type Event = ();
		type ExistentialDeposit = ExistentialDeposit;
		type BalanceInfo = AccountData<Balance>;
		type AccountStore = System;
		type MaxLocks = ();
		type WeightInfo = ();
		type OtherCurrencies = ();
	}

	parameter_types! {
		pub const ClaimsModuleId: ModuleId = ModuleId(*b"da/claim");
		pub Prefix: &'static [u8] = b"Pay RUSTs to the TEST account:";
	}
	ord_parameter_types! {
		pub const Six: u64 = 6;
	}
	impl Config for Test {
		type Event = ();
		type ModuleId = ClaimsModuleId;
		type Prefix = Prefix;
		type EtpCurrency = Etp;
		type MoveClaimOrigin = frame_system::EnsureSignedBy<Six, u64>;
	}

	frame_support::construct_runtime! {
		pub enum Test
		where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic
		{
			System: frame_system::{Module, Call, Storage, Config},
			Etp: hyperspace_balances::<Instance0>::{Module, Call, Storage, Config<T>},
			Claims: hyperspace_claims::{Module, Call, Storage, Config}
		}
	}

	fn alice() -> secp256k1::SecretKey {
		secp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
	}
	fn bob() -> secp256k1::SecretKey {
		secp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
	}
	fn carol() -> secp256k1::SecretKey {
		secp256k1::SecretKey::parse(&keccak_256(b"Carol")).unwrap()
	}

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		// We use default for brevity, but you can configure as desired if needed.
		<hyperspace_balances::GenesisConfig<Test, EtpInstance>>::default()
			.assimilate_storage(&mut t)
			.unwrap();
		hyperspace_claims::GenesisConfig {
			claims_list: ClaimsList {
				dot: vec![Account {
					address: EthereumAddress(addr(&alice())),
					backed_etp: 100,
				}],
				eth: vec![Account {
					address: EthereumAddress(addr(&bob())),
					backed_etp: 200,
				}],
				oldetp: vec![Account {
					address: OldetpAddress(addr(&carol())),
					backed_etp: 300,
				}],
			},
		}
		.assimilate_storage::<Test>(&mut t)
		.unwrap();
		t.into()
	}

	fn total_claims() -> u64 {
		100 + 200 + 300
	}

	#[test]
	fn basic_setup_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(Etp::usable_balance(&Claims::account_id()), 600);

			assert_eq!(Claims::claims_from_eth(&addr(&alice())), Some(100));
			assert_eq!(Claims::claims_from_oldetp(&addr(&alice())), None);

			assert_eq!(Claims::claims_from_eth(&addr(&bob())), Some(200));
			assert_eq!(Claims::claims_from_oldetp(&addr(&bob())), None);

			assert_eq!(Claims::claims_from_eth(&addr(&carol())), None);
			assert_eq!(Claims::claims_from_oldetp(&addr(&carol())), Some(300));
		});
	}

	#[test]
	fn serde_works() {
		let x = EthereumAddress(array_bytes::hex2array_unchecked!(
			"0x0123456789abcdef0123456789abcdef01234567",
			20
		));
		let y = serde_json::to_string(&x).unwrap();
		assert_eq!(y, "\"0x0123456789abcdef0123456789abcdef01234567\"");
		let z: EthereumAddress = serde_json::from_str(&y).unwrap();
		assert_eq!(x.0, z.0);

		let x = OldetpAddress(array_bytes::hex2array_unchecked!(
			"0x0123456789abcdef0123456789abcdef01234567",
			20
		));
		let y = serde_json::to_string(&x).unwrap();
		assert_eq!(y, "\"410123456789abcdef0123456789abcdef01234567\"");
		let z: OldetpAddress = serde_json::from_str(&y).unwrap();
		assert_eq!(x.0, z.0);
	}

	#[test]
	fn claiming_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(Etp::free_balance(1), 0);
			assert_ok!(Claims::claim(
				Origin::none(),
				1,
				OtherSignature::Eth(eth_sig::<Test>(
					&alice(),
					&1u64.encode(),
					ETHEREUM_SIGNED_MESSAGE
				)),
			));
			assert_eq!(Etp::free_balance(&1), 100);
			assert_eq!(Etp::usable_balance(&Claims::account_id()), 500);

			assert_eq!(Etp::free_balance(2), 0);
			assert_ok!(Claims::claim(
				Origin::none(),
				2,
				OtherSignature::Eth(eth_sig::<Test>(
					&bob(),
					&2u64.encode(),
					ETHEREUM_SIGNED_MESSAGE
				)),
			));
			assert_eq!(Etp::free_balance(&2), 200);
			assert_eq!(Etp::usable_balance(&Claims::account_id()), 300);

			assert_eq!(Etp::free_balance(3), 0);
			assert_ok!(Claims::claim(
				Origin::none(),
				3,
				OtherSignature::Oldetp(oldetp_sig::<Test>(
					&carol(),
					&3u64.encode(),
					OLDETP_SIGNED_MESSAGE
				)),
			));
			assert_eq!(Etp::free_balance(&3), 300);
			assert_eq!(Etp::usable_balance(&Claims::account_id()), 0);
		});
	}

	#[test]
	fn basic_claim_moving_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(Etp::free_balance(42), 0);
			assert_noop!(
				Claims::move_claim(
					Origin::signed(1),
					OtherAddress::Eth(addr(&alice())),
					OtherAddress::Eth(addr(&carol())),
				),
				BadOrigin
			);
			assert_noop!(
				Claims::move_claim(
					Origin::signed(6),
					OtherAddress::Eth(addr(&alice())),
					OtherAddress::Oldetp(addr(&carol())),
				),
				<Error<Test>>::NewAddressTypeMis
			);
			assert_noop!(
				Claims::move_claim(
					Origin::signed(6),
					OtherAddress::Eth(addr(&alice())),
					OtherAddress::Eth(addr(&bob())),
				),
				<Error<Test>>::MoveToExistedAddress
			);
			assert_ok!(Claims::move_claim(
				Origin::signed(6),
				OtherAddress::Eth(addr(&alice())),
				OtherAddress::Eth(addr(&carol())),
			));
			assert_noop!(
				Claims::claim(
					Origin::none(),
					42,
					OtherSignature::Eth(eth_sig::<Test>(
						&alice(),
						&42u64.encode(),
						ETHEREUM_SIGNED_MESSAGE
					))
				),
				<Error<Test>>::SignerHasNoClaim
			);
			assert_ok!(Claims::claim(
				Origin::none(),
				42,
				OtherSignature::Eth(eth_sig::<Test>(
					&carol(),
					&42u64.encode(),
					ETHEREUM_SIGNED_MESSAGE
				))
			));
			assert_eq!(Etp::free_balance(&42), 100);
			assert_eq!(
				Etp::usable_balance(&Claims::account_id()),
				total_claims() - 100
			);
		});
	}

	#[test]
	fn add_claim_works() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Claims::mint_claim(Origin::signed(42), OtherAddress::Eth(addr(&carol())), 200),
				sp_runtime::traits::BadOrigin,
			);
			assert_eq!(Etp::free_balance(42), 0);
			assert_noop!(
				Claims::claim(
					Origin::none(),
					69,
					OtherSignature::Eth(eth_sig::<Test>(
						&carol(),
						&69u64.encode(),
						ETHEREUM_SIGNED_MESSAGE
					)),
				),
				<Error<Test>>::SignerHasNoClaim,
			);
			assert_ok!(Claims::mint_claim(
				Origin::root(),
				OtherAddress::Eth(addr(&carol())),
				200
			));
			assert_eq!(Etp::usable_balance(&Claims::account_id()), 800);
			assert_ok!(Claims::claim(
				Origin::none(),
				69,
				OtherSignature::Eth(eth_sig::<Test>(
					&carol(),
					&69u64.encode(),
					ETHEREUM_SIGNED_MESSAGE
				)),
			));
			assert_eq!(Etp::free_balance(&69), 200);
			assert_eq!(Etp::usable_balance(&Claims::account_id()), 600);
		});
	}

	#[test]
	fn origin_signed_claiming_fail() {
		new_test_ext().execute_with(|| {
			assert_eq!(Etp::free_balance(42), 0);
			assert_err!(
				Claims::claim(
					Origin::signed(42),
					42,
					OtherSignature::Eth(eth_sig::<Test>(
						&alice(),
						&42u64.encode(),
						ETHEREUM_SIGNED_MESSAGE
					)),
				),
				sp_runtime::traits::BadOrigin,
			);
		});
	}

	#[test]
	fn double_claiming_doesnt_work() {
		new_test_ext().execute_with(|| {
			assert_eq!(Etp::free_balance(42), 0);
			assert_ok!(Claims::claim(
				Origin::none(),
				42,
				OtherSignature::Eth(eth_sig::<Test>(
					&alice(),
					&42u64.encode(),
					ETHEREUM_SIGNED_MESSAGE
				)),
			));
			assert_noop!(
				Claims::claim(
					Origin::none(),
					42,
					OtherSignature::Eth(eth_sig::<Test>(
						&alice(),
						&42u64.encode(),
						ETHEREUM_SIGNED_MESSAGE
					)),
				),
				<Error<Test>>::SignerHasNoClaim,
			);
		});
	}

	#[test]
	fn non_sender_sig_doesnt_work() {
		new_test_ext().execute_with(|| {
			assert_eq!(Etp::free_balance(42), 0);
			assert_noop!(
				Claims::claim(
					Origin::none(),
					42,
					OtherSignature::Eth(eth_sig::<Test>(
						&alice(),
						&69u64.encode(),
						ETHEREUM_SIGNED_MESSAGE
					)),
				),
				<Error<Test>>::SignerHasNoClaim,
			);
		});
	}

	#[test]
	fn non_claimant_doesnt_work() {
		new_test_ext().execute_with(|| {
			assert_eq!(Etp::free_balance(42), 0);
			assert_noop!(
				Claims::claim(
					Origin::none(),
					42,
					OtherSignature::Eth(eth_sig::<Test>(
						&carol(),
						&69u64.encode(),
						ETHEREUM_SIGNED_MESSAGE
					)),
				),
				<Error<Test>>::SignerHasNoClaim,
			);
		});
	}

	#[test]
	fn real_eth_sig_works() {
		new_test_ext().execute_with(|| {
				// "Pay RUSTs to the TEST account:2a00000000000000"
				let sig = array_bytes::hex2array_unchecked!("0x444023e89b67e67c0562ed0305d252a5dd12b2af5ac51d6d3cb69a0b486bc4b3191401802dc29d26d586221f7256cd3329fe82174bdf659baea149a40e1c495d1c", 65);
				let sig = EcdsaSignature(sig);
				let who = 42u64.using_encoded(to_ascii_hex);
				let signer = Claims::eth_recover(&sig, &who).unwrap();
				assert_eq!(signer, array_bytes::hex2array_unchecked!("0x6d31165d5d932d571f3b44695653b46dcc327e84", 20));
			});
	}

	#[test]
	fn real_oldetp_sig_works() {
		new_test_ext().execute_with(|| {
			// "Pay RUSTs to the TEST account:0c0529c66a44e1861e5e1502b4a87009f23c792518a7a2091363f5a0e38abd57"
			let sig = array_bytes::hex2array_unchecked!("0x34c3d5afc7f8fa08f9d00a1ec4ac274c63ebce99460b556de85258c94f41ab2f52ad5188bd9fc51251cf5dcdd53751b1bd577828db3f2e8fe8ef77907d7f3f6a1b", 65);
			let sig = EcdsaSignature(sig);
			let who = array_bytes::hex2array_unchecked!("0x0c0529c66a44e1861e5e1502b4a87009f23c792518a7a2091363f5a0e38abd57", 32).using_encoded(to_ascii_hex);
			let signer = Claims::oldetp_recover(&sig, &who).unwrap();
			assert_eq!(signer, array_bytes::hex2array_unchecked!("0x11974bce18a43243ede78beec2fd8e0ba4fe17ae", 20));
		});
	}

	#[test]
	fn validate_unsigned_works() {
		// --- substrate ---
		use sp_runtime::traits::ValidateUnsigned;

		let source = sp_runtime::transaction_validity::TransactionSource::External;

		new_test_ext().execute_with(|| {
			assert_eq!(
				Claims::validate_unsigned(
					source,
					&hyperspace_claims::Call::claim(
						1,
						OtherSignature::Eth(eth_sig::<Test>(
							&alice(),
							&1u64.encode(),
							ETHEREUM_SIGNED_MESSAGE
						)),
					)
				),
				Ok(ValidTransaction {
					priority: 100,
					requires: vec![],
					provides: vec![("claims", addr(&alice())).encode()],
					longevity: TransactionLongevity::max_value(),
					propagate: true,
				})
			);
			assert_eq!(
				Claims::validate_unsigned(
					source,
					&hyperspace_claims::Call::claim(0, OtherSignature::Eth(EcdsaSignature([0; 65])))
				),
				InvalidTransaction::Custom(ValidityError::InvalidSignature as _).into(),
			);
			assert_eq!(
				Claims::validate_unsigned(
					source,
					&hyperspace_claims::Call::claim(
						1,
						OtherSignature::Eth(eth_sig::<Test>(
							&carol(),
							&1u64.encode(),
							ETHEREUM_SIGNED_MESSAGE
						)),
					)
				),
				InvalidTransaction::Custom(ValidityError::SignerHasNoClaim as _).into(),
			);
			assert_eq!(
				Claims::validate_unsigned(
					source,
					&hyperspace_claims::Call::claim(
						0,
						OtherSignature::Eth(eth_sig::<Test>(
							&carol(),
							&1u64.encode(),
							ETHEREUM_SIGNED_MESSAGE
						)),
					)
				),
				InvalidTransaction::Custom(ValidityError::SignerHasNoClaim as _).into(),
			);
		});
	}
}
