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

//! Prototype module for cross chain assets backing.

// TODO: https://github.com/mvs-org/Hyperspaceissues/372
#![allow(unused)]
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]

pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test_with_linear_relay;
#[cfg(test)]
mod test_with_relay;

#[frame_support::pallet]
pub mod pallet {
	pub mod types {
		// --- hyperspace ---
		use crate::pallet::*;

		// Simple type
		pub type Balance = u128;
		pub type DepositId = U256;
		pub type EcdsaSignature = [u8; 65];
		pub type EcdsaMessage = [u8; 32];
		// Generic type
		pub type AccountId<T> = <T as frame_system::Config>::AccountId;
		pub type BlockNumber<T> = <T as frame_system::Config>::BlockNumber;
		pub type EtpBalance<T> = <<T as Config>::EtpCurrency as Currency<AccountId<T>>>::Balance;
		pub type DnaBalance<T> = <<T as Config>::DnaCurrency as Currency<AccountId<T>>>::Balance;
		pub type EthereumReceiptProofThing<T> = <<T as Config>::EthereumRelay as EthereumReceipt<
			AccountId<T>,
			EtpBalance<T>,
		>>::EthereumReceiptProofThing;
	}
	pub use types::*;

	// --- substrate ---
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement},
	};
	use frame_system::pallet_prelude::*;
	use sp_io::{crypto, hashing};
	use sp_runtime::{
		traits::{AccountIdConversion, SaturatedConversion, Saturating, Zero},
		ModuleId,
	};
	#[cfg(not(feature = "std"))]
	use sp_std::borrow::ToOwned;
	use sp_std::{convert::TryFrom, prelude::*};
	// --- hyperspace ---
	use crate::weights::WeightInfo;
	use hyperspace_relay_primitives::relay_authorities::*;
	use hyperspace_support::{
		balance::*,
		traits::{EthereumReceipt, OnDepositRedeem},
	};
	use ethabi::{Event as EthEvent, EventParam as EthEventParam, ParamType, RawLog};
	use ethereum_primitives::{
		receipt::{EthereumTransactionIndex, LogEntry},
		EthereumAddress, U256,
	};

	// TODO
	// macro_rules! set_address_call {
	// 	($call_name:ident, $address:ty) => {
	// 		#[pallet::weight(10_000_000)]
	// 		pub fn $call_name(
	// 			origin: OriginFor<T>,
	// 			new: EthereumAddress,
	// 		) -> DispatchResultWithPostInfo {
	// 			ensure_root(origin)?;

	// 			$address::put(new);

	// 			Ok(().into())
	// 		}
	// 	}
	// }

	#[pallet::config]
	pub trait Config: frame_system::Config {
		// --- substrate ---
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		// --- hyperspace ---
		#[pallet::constant]
		type ModuleId: Get<ModuleId>;
		#[pallet::constant]
		type FeeModuleId: Get<ModuleId>;
		type EtpCurrency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		type DnaCurrency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		type RedeemAccountId: From<[u8; 32]> + Into<Self::AccountId>;
		type EthereumRelay: EthereumReceipt<Self::AccountId, EtpBalance<Self>>;
		type OnDepositRedeem: OnDepositRedeem<Self::AccountId, EtpBalance<Self>>;
		#[pallet::constant]
		type EtpLockLimit: Get<EtpBalance<Self>>;
		#[pallet::constant]
		type DnaLockLimit: Get<DnaBalance<Self>>;
		#[pallet::constant]
		type AdvancedFee: Get<EtpBalance<Self>>;
		#[pallet::constant]
		type SyncReward: Get<EtpBalance<Self>>;
		type EcdsaAuthorities: RelayAuthorityProtocol<Self::BlockNumber, Signer = EthereumAddress>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	#[pallet::metadata(
		AccountId<T> = "AccountId",
		EtpBalance<T> = "EtpBalance",
		DnaBalance<T> = "DnaBalance",
	)]
	pub enum Event<T: Config> {
		/// Someone redeem some *ETP*. \[account, amount, transaction index\]
		RedeemEtp(AccountId<T>, Balance, EthereumTransactionIndex),
		/// Someone redeem some *DNA*. \[account, amount, transaction index\]
		RedeemDna(AccountId<T>, Balance, EthereumTransactionIndex),
		/// Someone redeem a deposit. \[account, deposit id, amount, transaction index\]
		RedeemDeposit(
			AccountId<T>,
			DepositId,
			EtpBalance<T>,
			EthereumTransactionIndex,
		),
		/// Someone lock some *ETP*. \[account, ethereum account, asset address, amount\]
		LockEtp(
			AccountId<T>,
			EthereumAddress,
			EthereumAddress,
			EtpBalance<T>,
		),
		/// Someone lock some *DNA*. \[account, ethereum account, asset address, amount\]
		LockDna(
			AccountId<T>,
			EthereumAddress,
			EthereumAddress,
			DnaBalance<T>,
		),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Address Length - MISMATCHED
		AddrLenMis,
		/// Pubkey Prefix - MISMATCHED
		PubkeyPrefixMis,
		/// Bytes - CONVERSION FAILED
		BytesCF,
		/// Int - CONVERSION FAILED
		IntCF,
		/// Array - CONVERSION FAILED
		ArrayCF,
		/// Address - CONVERSION FAILED
		AddressCF,
		/// Asset - ALREADY REDEEMED
		AssetAR,
		/// Authorities Change - ALREADY SYNCED
		AuthoritiesChangeAR,
		/// EthereumReceipt Proof - INVALID
		ReceiptProofInv,
		/// Eth Log - PARSING FAILED
		EthLogPF,
		/// *DNA* Locked - NO SUFFICIENT BACKING ASSETS
		DnaLockedNSBA,
		/// *ETP* Locked - NO SUFFICIENT BACKING ASSETS
		EtpLockedNSBA,
		/// Log Entry - NOT EXISTED
		LogEntryNE,
		// TODO: remove fee?
		// /// Usable Balance for Paying Redeem Fee - INSUFFICIENT
		// FeeIns,
		/// Redeem - DISABLED
		RedeemDis,
		/// Etp Lock - LIMITED
		EtpLockLim,
		/// Dna Lock - LIMITED
		DnaLockLim,
	}

	#[pallet::storage]
	#[pallet::getter(fn verified_proof)]
	pub type VerifiedProof<T> = StorageMap<
		_,
		Blake2_128Concat,
		EthereumTransactionIndex,
		bool,
		ValueQuery,
		DefaultForVerifiedProof,
	>;
	#[pallet::type_value]
	pub fn DefaultForVerifiedProof() -> bool {
		false
	}

	#[pallet::storage]
	#[pallet::getter(fn token_redeem_address)]
	pub type TokenRedeemAddress<T> = StorageValue<_, EthereumAddress, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn deposit_redeem_address)]
	pub type DepositRedeemAddress<T> = StorageValue<_, EthereumAddress, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn set_authorities_address)]
	pub type SetAuthoritiesAddress<T> = StorageValue<_, EthereumAddress, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn etp_token_address)]
	pub type EtpTokenAddress<T> = StorageValue<_, EthereumAddress, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn dna_token_address)]
	pub type DnaTokenAddress<T> = StorageValue<_, EthereumAddress, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn redeem_status)]
	pub type RedeemStatus<T> = StorageValue<_, bool, ValueQuery, DefaultForRedeemStatus>;
	#[pallet::type_value]
	pub fn DefaultForRedeemStatus() -> bool {
		true
	}

	#[pallet::storage]
	#[pallet::getter(fn lock_asset_events)]
	pub type LockAssetEvents<T> =
		StorageValue<_, Vec<<T as frame_system::Config>::Event>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub token_redeem_address: EthereumAddress,
		pub deposit_redeem_address: EthereumAddress,
		pub set_authorities_address: EthereumAddress,
		pub etp_token_address: EthereumAddress,
		pub dna_token_address: EthereumAddress,
		pub backed_etp: EtpBalance<T>,
		pub backed_dna: DnaBalance<T>,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				token_redeem_address: Default::default(),
				deposit_redeem_address: Default::default(),
				set_authorities_address: Default::default(),
				etp_token_address: Default::default(),
				dna_token_address: Default::default(),
				backed_etp: Default::default(),
				backed_dna: Default::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<TokenRedeemAddress<T>>::put(self.token_redeem_address);
			<DepositRedeemAddress<T>>::put(self.deposit_redeem_address);
			<SetAuthoritiesAddress<T>>::put(self.set_authorities_address);
			<EtpTokenAddress<T>>::put(self.etp_token_address);
			<DnaTokenAddress<T>>::put(self.dna_token_address);

			let _ = T::EtpCurrency::make_free_balance_be(
				&<Pallet<T>>::account_id(),
				T::EtpCurrency::minimum_balance() + self.backed_etp,
			);
			let _ =
				T::DnaCurrency::make_free_balance_be(&<Pallet<T>>::account_id(), self.backed_dna);
			let _ = T::EtpCurrency::make_free_balance_be(
				&<Pallet<T>>::fee_account_id(),
				T::EtpCurrency::minimum_balance(),
			);
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: BlockNumber<T>) -> Weight {
			<LockAssetEvents<T>>::kill();

			T::DbWeight::get().writes(1)
		}
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Redeem balances
		///
		/// # <weight>
		/// - `O(1)`
		/// # </weight>
		#[pallet::weight(10_000_000)]
		pub fn redeem(
			origin: OriginFor<T>,
			act: RedeemFor,
			proof: EthereumReceiptProofThing<T>,
		) -> DispatchResultWithPostInfo {
			let redeemer = ensure_signed(origin)?;

			if <RedeemStatus<T>>::get() {
				match act {
					RedeemFor::Token => Self::redeem_token(&redeemer, &proof)?,
					RedeemFor::Deposit => Self::redeem_deposit(&redeemer, &proof)?,
				}
			} else {
				Err(<Error<T>>::RedeemDis)?;
			}

			Ok(().into())
		}

		/// Lock some balances into the module account
		/// which very similar to lock some assets into the contract on ethereum side
		///
		/// This might kill the account just like `balances::transfer`
		#[pallet::weight(10_000_000)]
		#[frame_support::transactional]
		pub fn lock(
			origin: OriginFor<T>,
			#[pallet::compact] etp_to_lock: EtpBalance<T>,
			#[pallet::compact] dna_to_lock: DnaBalance<T>,
			ethereum_account: EthereumAddress,
		) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;
			let fee_account = Self::fee_account_id();

			// 50 Etp for fee
			// https://github.com/mvs-org/Hyperspacepull/377#issuecomment-730369387
			T::EtpCurrency::transfer(
				&user,
				&fee_account,
				T::AdvancedFee::get(),
				ExistenceRequirement::KeepAlive,
			)?;

			let mut locked = false;

			if !etp_to_lock.is_zero() {
				ensure!(
					etp_to_lock < T::EtpLockLimit::get(),
					<Error<T>>::EtpLockLim
				);

				T::EtpCurrency::transfer(
					&user,
					&Self::account_id(),
					etp_to_lock,
					ExistenceRequirement::AllowDeath,
				)?;

				let event = Event::LockEtp(
					user.clone(),
					ethereum_account.clone(),
					<EtpTokenAddress<T>>::get(),
					etp_to_lock,
				);
				let module_event: <T as Config>::Event = event.clone().into();
				let system_event: <T as frame_system::Config>::Event = module_event.into();

				locked = true;

				<LockAssetEvents<T>>::append(system_event);
				Self::deposit_event(event);
			}
			if !dna_to_lock.is_zero() {
				ensure!(
					dna_to_lock < T::DnaLockLimit::get(),
					<Error<T>>::DnaLockLim
				);

				T::DnaCurrency::transfer(
					&user,
					&Self::account_id(),
					dna_to_lock,
					ExistenceRequirement::AllowDeath,
				)?;

				let event = Event::LockDna(
					user,
					ethereum_account,
					<DnaTokenAddress<T>>::get(),
					dna_to_lock,
				);
				let module_event: <T as Config>::Event = event.clone().into();
				let system_event: <T as frame_system::Config>::Event = module_event.into();

				locked = true;

				<LockAssetEvents<T>>::append(system_event);
				Self::deposit_event(event);
			}

			if locked {
				T::EcdsaAuthorities::schedule_mmr_root(
					(<frame_system::Pallet<T>>::block_number().saturated_into::<u32>() / 10 * 10
						+ 10)
						.saturated_into(),
				);
			}

			Ok(().into())
		}

		// Transfer should always return ok
		// Even it failed, still finish the syncing
		//
		// But should not dispatch the reward if the syncing failed
		#[pallet::weight(10_000_000)]
		pub fn sync_authorities_change(
			origin: OriginFor<T>,
			proof: EthereumReceiptProofThing<T>,
		) -> DispatchResultWithPostInfo {
			let bridger = ensure_signed(origin)?;
			let tx_index = T::EthereumRelay::gen_receipt_index(&proof);

			ensure!(
				!<VerifiedProof<T>>::contains_key(tx_index),
				<Error<T>>::AuthoritiesChangeAR
			);

			let (term, authorities, beneficiary) = Self::parse_authorities_set_proof(&proof)?;

			T::EcdsaAuthorities::check_authorities_change_to_sync(term, authorities)?;
			T::EcdsaAuthorities::sync_authorities_change()?;

			<VerifiedProof<T>>::insert(tx_index, true);

			let fee_account = Self::fee_account_id();
			let sync_reward = T::SyncReward::get().min(
				T::EtpCurrency::usable_balance(&fee_account)
					.saturating_sub(T::EtpCurrency::minimum_balance()),
			);

			if !sync_reward.is_zero() {
				T::EtpCurrency::transfer(
					&fee_account,
					&beneficiary,
					sync_reward,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			Ok(().into())
		}

		/// Set a new ring redeem address.
		///
		/// The dispatch origin of this call must be _Root_.
		///
		/// - `new`: The new ring redeem address.
		///
		/// # <weight>
		/// - `O(1)`.
		/// # </weight>
		#[pallet::weight(10_000_000)]
		pub fn set_token_redeem_address(
			origin: OriginFor<T>,
			new: EthereumAddress,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			<TokenRedeemAddress<T>>::put(new);

			Ok(().into())
		}

		/// Set a new deposit redeem address.
		///
		/// The dispatch origin of this call must be _Root_.
		///
		/// - `new`: The new deposit redeem address.
		///
		/// # <weight>
		/// - `O(1)`.
		/// # </weight>
		#[pallet::weight(10_000_000)]
		pub fn set_deposit_redeem_address(
			origin: OriginFor<T>,
			new: EthereumAddress,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			<DepositRedeemAddress<T>>::put(new);

			Ok(().into())
		}

		#[pallet::weight(10_000_000)]
		pub fn set_set_authorities_address(
			origin: OriginFor<T>,
			new: EthereumAddress,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			<SetAuthoritiesAddress<T>>::put(new);

			Ok(().into())
		}

		#[pallet::weight(10_000_000)]
		pub fn set_etp_token_address(
			origin: OriginFor<T>,
			new: EthereumAddress,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			<EtpTokenAddress<T>>::put(new);

			Ok(().into())
		}

		#[pallet::weight(10_000_000)]
		pub fn set_dna_token_address(
			origin: OriginFor<T>,
			new: EthereumAddress,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			<DnaTokenAddress<T>>::put(new);

			Ok(().into())
		}

		#[pallet::weight(10_000_000)]
		pub fn set_redeem_status(origin: OriginFor<T>, status: bool) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			<RedeemStatus<T>>::put(status);

			Ok(().into())
		}
	}
	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::ModuleId::get().into_account()
		}
		pub fn fee_account_id() -> T::AccountId {
			T::FeeModuleId::get().into_account()
		}

		pub fn account_id_try_from_bytes(bytes: &[u8]) -> Result<T::AccountId, DispatchError> {
			ensure!(bytes.len() == 32, <Error<T>>::AddrLenMis);

			let redeem_account_id: T::RedeemAccountId = array_bytes::dyn2array!(bytes, 32).into();

			Ok(redeem_account_id.into())
		}

		/// Return the amount of money in the pot.
		// The existential deposit is not part of the pot so backing account never gets deleted.
		pub fn pot<C: LockableCurrency<T::AccountId>>() -> C::Balance {
			C::usable_balance(&Self::account_id())
				// Must never be less than 0 but better be safe.
				.saturating_sub(C::minimum_balance())
		}

		fn redeem_token(
			redeemer: &T::AccountId,
			proof: &EthereumReceiptProofThing<T>,
		) -> DispatchResult {
			let tx_index = T::EthereumRelay::gen_receipt_index(proof);

			ensure!(
				!<VerifiedProof<T>>::contains_key(tx_index),
				<Error<T>>::AssetAR
			);

			// TODO: remove fee?
			let (hyperspace_account, (is_etp, redeem_amount), fee) =
				Self::parse_token_redeem_proof(&proof)?;

			if is_etp {
				Self::redeem_token_cast::<T::EtpCurrency>(
					redeemer,
					hyperspace_account,
					tx_index,
					true,
					redeem_amount,
					fee,
				)?;
			} else {
				Self::redeem_token_cast::<T::DnaCurrency>(
					redeemer,
					hyperspace_account,
					tx_index,
					false,
					redeem_amount,
					fee,
				)?;
			}

			Ok(())
		}
		fn redeem_token_cast<C: LockableCurrency<T::AccountId>>(
			redeemer: &T::AccountId,
			hyperspace_account: T::AccountId,
			tx_index: EthereumTransactionIndex,
			is_etp: bool,
			redeem_amount: Balance,
			fee: EtpBalance<T>,
		) -> DispatchResult {
			let raw_amount = redeem_amount;
			let redeem_amount: C::Balance = redeem_amount.saturated_into();

			ensure!(
				Self::pot::<C>() >= redeem_amount,
				if is_etp {
					<Error<T>>::EtpLockedNSBA
				} else {
					<Error<T>>::DnaLockedNSBA
				}
			);
			// // Checking redeemer have enough of balance to pay fee, make sure follow up transfer will success.
			// ensure!(
			// 	T::EtpCurrency::usable_balance(redeemer) >= fee,
			// 	<Error<T>>::FeeIns
			// );

			C::transfer(
				&Self::account_id(),
				&hyperspace_account,
				redeem_amount,
				ExistenceRequirement::KeepAlive,
			)?;
			// // Transfer the fee from redeemer.
			// T::EtpCurrency::transfer(redeemer, &T::EthereumRelay::account_id(), fee, KeepAlive)?;

			<VerifiedProof<T>>::insert(tx_index, true);

			if is_etp {
				Self::deposit_event(Event::RedeemEtp(hyperspace_account, raw_amount, tx_index));
			} else {
				Self::deposit_event(Event::RedeemDna(hyperspace_account, raw_amount, tx_index));
			}

			Ok(())
		}
		// event BurnAndRedeem(address indexed token, address indexed from, uint256 amount, bytes receiver);
		// Redeem ETP https://ropsten.etherscan.io/tx/0x1d3ef601b9fa4a7f1d6259c658d0a10c77940fa5db9e10ab55397eb0ce88807d
		// Redeem DNA https://ropsten.etherscan.io/tx/0x2878ae39a9e0db95e61164528bb1ec8684be194bdcc236848ff14d3fe5ba335d
		pub(super) fn parse_token_redeem_proof(
			proof_record: &EthereumReceiptProofThing<T>,
		) -> Result<(T::AccountId, (bool, Balance), EtpBalance<T>), DispatchError> {
			let verified_receipt = T::EthereumRelay::verify_receipt(proof_record)
				.map_err(|_| <Error<T>>::ReceiptProofInv)?;
			let fee = T::EthereumRelay::receipt_verify_fee();
			let result = {
				let eth_event = EthEvent {
					name: "BurnAndRedeem".to_owned(),
					inputs: vec![
						EthEventParam {
							name: "token".to_owned(),
							kind: ParamType::Address,
							indexed: true,
						},
						EthEventParam {
							name: "from".to_owned(),
							kind: ParamType::Address,
							indexed: true,
						},
						EthEventParam {
							name: "amount".to_owned(),
							kind: ParamType::Uint(256),
							indexed: false,
						},
						EthEventParam {
							name: "receiver".to_owned(),
							kind: ParamType::Bytes,
							indexed: false,
						},
					],
					anonymous: false,
				};
				let log_entry = verified_receipt
					.logs
					.into_iter()
					.find(|x| {
						x.address == <TokenRedeemAddress<T>>::get()
							&& x.topics[0] == eth_event.signature()
					})
					.ok_or(<Error<T>>::LogEntryNE)?;
				let log = RawLog {
					topics: vec![
						log_entry.topics[0],
						log_entry.topics[1],
						log_entry.topics[2],
					],
					data: log_entry.data.clone(),
				};

				eth_event.parse_log(log).map_err(|_| <Error<T>>::EthLogPF)?
			};
			let is_etp = {
				let token_address = result.params[0]
					.value
					.clone()
					.into_address()
					.ok_or(<Error<T>>::AddressCF)?;

				ensure!(
					token_address == <EtpTokenAddress<T>>::get()
						|| token_address == <DnaTokenAddress<T>>::get(),
					<Error<T>>::AssetAR
				);

				token_address == <EtpTokenAddress<T>>::get()
			};

			let redeemed_amount = {
				// TODO: div 10**18 and mul 10**9
				let amount = result.params[2]
					.value
					.clone()
					.into_uint()
					.map(|x| x / U256::from(1_000_000_000u64))
					.ok_or(<Error<T>>::IntCF)?;

				Balance::try_from(amount)?
			};
			let hyperspace_account = {
				let raw_account_id = result.params[3]
					.value
					.clone()
					.into_bytes()
					.ok_or(<Error<T>>::BytesCF)?;
				log::trace!("[ethereum-backing] Raw Account: {:?}", raw_account_id);

				Self::account_id_try_from_bytes(&raw_account_id)?
			};
			log::trace!(
				"[ethereum-backing] Hyperspace Account: {:?}",
				hyperspace_account
			);

			Ok((hyperspace_account, (is_etp, redeemed_amount), fee))
		}

		fn redeem_deposit(
			redeemer: &T::AccountId,
			proof: &EthereumReceiptProofThing<T>,
		) -> DispatchResult {
			let tx_index = T::EthereumRelay::gen_receipt_index(proof);

			ensure!(
				!<VerifiedProof<T>>::contains_key(tx_index),
				<Error<T>>::AssetAR
			);

			// TODO: remove fee?
			let (deposit_id, hyperspace_account, redeemed_etp, start_at, months, fee) =
				Self::parse_deposit_redeem_proof(&proof)?;

			ensure!(
				Self::pot::<T::EtpCurrency>() >= redeemed_etp,
				<Error<T>>::EtpLockedNSBA
			);
			// // Checking redeemer have enough of balance to pay fee, make sure follow up fee transfer will success.
			// ensure!(
			// 	T::EtpCurrency::usable_balance(redeemer) >= fee,
			// 	<Error<T>>::FeeIns
			// );

			T::OnDepositRedeem::on_deposit_redeem(
				&Self::account_id(),
				&hyperspace_account,
				redeemed_etp,
				start_at,
				months,
			)?;
			// // Transfer the fee from redeemer.
			// T::EtpCurrency::transfer(redeemer, &T::EthereumRelay::account_id(), fee, KeepAlive)?;

			// TODO: check deposit_id duplication
			// TODO: Ignore Unit Interest for now
			<VerifiedProof<T>>::insert(tx_index, true);

			Self::deposit_event(Event::RedeemDeposit(
				hyperspace_account,
				deposit_id,
				redeemed_etp,
				tx_index,
			));

			Ok(())
		}
		// event BurnAndRedeem(uint256 indexed _depositID,  address _depositor, uint48 _months, uint48 _startAt, uint64 _unitInterest, uint128 _value, bytes _data);
		// Redeem Deposit https://ropsten.etherscan.io/tx/0x5a7004126466ce763501c89bcbb98d14f3c328c4b310b1976a38be1183d91919
		fn parse_deposit_redeem_proof(
			proof_record: &EthereumReceiptProofThing<T>,
		) -> Result<
			(
				DepositId,
				T::AccountId,
				EtpBalance<T>,
				u64,
				u8,
				EtpBalance<T>,
			),
			DispatchError,
		> {
			let verified_receipt = T::EthereumRelay::verify_receipt(proof_record)
				.map_err(|_| <Error<T>>::ReceiptProofInv)?;
			let fee = T::EthereumRelay::receipt_verify_fee();
			let result = {
				let eth_event = EthEvent {
					name: "BurnAndRedeem".to_owned(),
					inputs: vec![
						EthEventParam {
							name: "_depositID".to_owned(),
							kind: ParamType::Uint(256),
							indexed: true,
						},
						EthEventParam {
							name: "_depositor".to_owned(),
							kind: ParamType::Address,
							indexed: false,
						},
						EthEventParam {
							name: "_months".to_owned(),
							kind: ParamType::Uint(48),
							indexed: false,
						},
						EthEventParam {
							name: "_startAt".to_owned(),
							kind: ParamType::Uint(48),
							indexed: false,
						},
						EthEventParam {
							name: "_unitInterest".to_owned(),
							kind: ParamType::Uint(64),
							indexed: false,
						},
						EthEventParam {
							name: "_value".to_owned(),
							kind: ParamType::Uint(128),
							indexed: false,
						},
						EthEventParam {
							name: "_data".to_owned(),
							kind: ParamType::Bytes,
							indexed: false,
						},
					],
					anonymous: false,
				};
				let log_entry = verified_receipt
					.logs
					.iter()
					.find(|&x| {
						x.address == <DepositRedeemAddress<T>>::get()
							&& x.topics[0] == eth_event.signature()
					})
					.ok_or(<Error<T>>::LogEntryNE)?;
				let log = RawLog {
					topics: vec![log_entry.topics[0], log_entry.topics[1]],
					data: log_entry.data.clone(),
				};

				eth_event.parse_log(log).map_err(|_| <Error<T>>::EthLogPF)?
			};
			let deposit_id = result.params[0]
				.value
				.clone()
				.into_uint()
				.ok_or(<Error<T>>::IntCF)?;
			let months = {
				let months = result.params[2]
					.value
					.clone()
					.into_uint()
					.ok_or(<Error<T>>::IntCF)?;

				months.saturated_into()
			};
			// The start_at here is in seconds, will be converted to milliseconds later in on_deposit_redeem
			let start_at = {
				let start_at = result.params[3]
					.value
					.clone()
					.into_uint()
					.ok_or(<Error<T>>::IntCF)?;

				start_at.saturated_into()
			};
			let redeemed_etp = {
				// The decimal in Ethereum is 10**18, and the decimal in Hyperspace is 10**9,
				// div 10**18 and mul 10**9
				let redeemed_etp = result.params[5]
					.value
					.clone()
					.into_uint()
					.map(|x| x / U256::from(1_000_000_000u64))
					.ok_or(<Error<T>>::IntCF)?;

				<EtpBalance<T>>::saturated_from(redeemed_etp.saturated_into::<u128>())
			};
			let hyperspace_account = {
				let raw_account_id = result.params[6]
					.value
					.clone()
					.into_bytes()
					.ok_or(<Error<T>>::BytesCF)?;
				log::trace!("[ethereum-backing] Raw Account: {:?}", raw_account_id);

				Self::account_id_try_from_bytes(&raw_account_id)?
			};
			log::trace!(
				"[ethereum-backing] Hyperspace Account: {:?}",
				hyperspace_account
			);

			Ok((
				deposit_id,
				hyperspace_account,
				redeemed_etp,
				start_at,
				months,
				fee,
			))
		}

		// event SetAuthritiesEvent(uint32 nonce, address[] authorities, bytes32 benifit);
		// https://github.com/hyperspace-network/hyperspace-bridge-on-ethereum/blob/51839e614c0575e431eabfd5c70b84f6aa37826a/contracts/Relay.sol#L22
		// https://ropsten.etherscan.io/tx/0x652528b9421ecb495610a734a4ab70d054b5510dbbf3a9d5c7879c43c7dde4e9#eventlog
		fn parse_authorities_set_proof(
			proof_record: &EthereumReceiptProofThing<T>,
		) -> Result<(Term, Vec<EthereumAddress>, AccountId<T>), DispatchError> {
			let log = {
				let verified_receipt = T::EthereumRelay::verify_receipt(proof_record)
					.map_err(|_| <Error<T>>::ReceiptProofInv)?;
				let eth_event = EthEvent {
					name: "SetAuthoritiesEvent".into(),
					inputs: vec![
						EthEventParam {
							name: "nonce".into(),
							kind: ParamType::Uint(32),
							indexed: false,
						},
						EthEventParam {
							name: "authorities".into(),
							kind: ParamType::Array(Box::new(ParamType::Address)),
							indexed: false,
						},
						EthEventParam {
							name: "beneficiary".into(),
							kind: ParamType::FixedBytes(32),
							indexed: false,
						},
					],
					anonymous: false,
				};
				let LogEntry { topics, data, .. } = verified_receipt
					.logs
					.into_iter()
					.find(|x| {
						x.address == <SetAuthoritiesAddress<T>>::get()
							&& x.topics[0] == eth_event.signature()
					})
					.ok_or(<Error<T>>::LogEntryNE)?;

				eth_event
					.parse_log(RawLog {
						topics: vec![topics[0]],
						data,
					})
					.map_err(|_| <Error<T>>::EthLogPF)?
			};
			let term = log.params[0]
				.value
				.clone()
				.into_uint()
				.ok_or(<Error<T>>::BytesCF)?
				.saturated_into();
			let authorities = {
				let mut authorities = vec![];

				for token in log.params[1]
					.value
					.clone()
					.into_array()
					.ok_or(<Error<T>>::ArrayCF)?
				{
					authorities.push(token.into_address().ok_or(<Error<T>>::AddressCF)?);
				}

				authorities
			};
			let beneficiary = {
				let raw_account_id = log.params[2]
					.value
					.clone()
					.into_fixed_bytes()
					.ok_or(<Error<T>>::BytesCF)?;

				log::trace!("[ethereum-backing] Raw Account: {:?}", raw_account_id);

				Self::account_id_try_from_bytes(&raw_account_id)?
			};

			Ok((term, authorities, beneficiary))
		}
	}

	impl<T: Config> Sign<BlockNumber<T>> for Pallet<T> {
		type Signature = EcdsaSignature;
		type Message = EcdsaMessage;
		type Signer = EthereumAddress;

		fn hash(raw_message: impl AsRef<[u8]>) -> Self::Message {
			hashing::keccak_256(raw_message.as_ref())
		}

		fn verify_signature(
			signature: &Self::Signature,
			message: &Self::Message,
			signer: &Self::Signer,
		) -> bool {
			fn eth_signable_message(message: &[u8]) -> Vec<u8> {
				let mut l = message.len();
				let mut rev = Vec::new();

				while l > 0 {
					rev.push(b'0' + (l % 10) as u8);
					l /= 10;
				}

				let mut v = b"\x19Ethereum Signed Message:\n".to_vec();

				v.extend(rev.into_iter().rev());
				v.extend_from_slice(message);

				v
			}

			let message = hashing::keccak_256(&eth_signable_message(message));

			if let Ok(public_key) = crypto::secp256k1_ecdsa_recover(signature, &message) {
				hashing::keccak_256(&public_key)[12..] == signer.0
			} else {
				false
			}
		}
	}

	#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
	pub enum RedeemFor {
		Token,
		Deposit,
	}
}
pub use pallet::*;

pub mod migration {
	const OLD_PALLET_NAME: &[u8] = b"HyperspaceEthereumBacking";

	#[cfg(feature = "try-runtime")]
	pub mod try_runtime {
		pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
			Ok(())
		}
	}

	pub fn migrate(new_pallet_name: &[u8]) {
		frame_support::migration::move_pallet(OLD_PALLET_NAME, new_pallet_name);
	}
}
