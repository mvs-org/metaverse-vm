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

//! The Hyperspace Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

pub mod constants {
	// --- substrate ---
	use sp_staking::SessionIndex;
	// --- hyperspace ---
	use crate::*;

	pub const NANO: Balance = 1;
	pub const MICRO: Balance = 1_000 * NANO;
	pub const MILLI: Balance = 1_000 * MICRO;
	pub const COIN: Balance = 1_000 * MILLI;

	pub const CAP: Balance = 10_000_000_000 * COIN;
	pub const TOTAL_POWER: Power = 1_000_000_000;

	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = 60 * MINUTES;
	pub const DAYS: BlockNumber = 24 * HOURS;

	pub const MILLISECS_PER_BLOCK: Moment = 23000;
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
	pub const BLOCKS_PER_SESSION: BlockNumber = 2057 * MINUTES;
	pub const SESSIONS_PER_ERA: SessionIndex = 6;

	// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 20 * COIN + (bytes as Balance) * 100 * MICRO
	}
}

pub mod impls {
	//! Some configurable implementations as associated type for the substrate runtime.

	pub mod relay {
		// --- hyperspace ---
		use crate::*;
		use hyperspace_relay_primitives::relayer_game::*;
		use ethereum_primitives::EthereumBlockNumber;

		pub struct EthereumRelayerGameAdjustor;
		impl AdjustableRelayerGame for EthereumRelayerGameAdjustor {
			type Moment = BlockNumber;
			type Balance = Balance;
			type RelayHeaderId = EthereumBlockNumber;

			fn max_active_games() -> u8 {
				32
			}

			fn affirm_time(round: u32) -> Self::Moment {
				match round {
					// 1.5 mins
					0 => 15,
					// 0.5 mins
					_ => 5,
				}
			}

			fn complete_proofs_time(round: u32) -> Self::Moment {
				match round {
					// 1.5 mins
					0 => 15,
					// 0.5 mins
					_ => 5,
				}
			}

			fn update_sample_points(sample_points: &mut Vec<Vec<Self::RelayHeaderId>>) {
				sample_points.push(vec![sample_points.last().unwrap().last().unwrap() - 1]);
			}

			fn estimate_stake(round: u32, affirmations_count: u32) -> Self::Balance {
				match round {
					0 => match affirmations_count {
						0 => 1000 * COIN,
						_ => 1500 * COIN,
					},
					_ => 100 * COIN,
				}
			}
		}
	}

	// --- crates ---
	use smallvec::smallvec;
	// --- substrate ---
	use frame_support::{
		traits::{Currency, Imbalance, OnUnbalanced},
		weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
	};
	// --- hyperspace ---
	use crate::*;

	hyperspace_support::impl_account_data! {
		struct AccountData<Balance>
		for
			EtpInstance,
			DnaInstance
		where
			Balance = Balance
		{
			// other data
		}
	}

	pub struct Author;
	impl OnUnbalanced<NegativeImbalance> for Author {
		fn on_nonzero_unbalanced(amount: NegativeImbalance) {
			Etp::resolve_creating(&Authorship::author(), amount);
		}
	}

	pub struct DealWithFees;
	impl OnUnbalanced<NegativeImbalance> for DealWithFees {
		fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
			if let Some(fees) = fees_then_tips.next() {
				// for fees, 80% to treasury, 20% to author
				let mut split = fees.ration(80, 20);
				if let Some(tips) = fees_then_tips.next() {
					// for tips, if any, 80% to treasury, 20% to author (though this can be anything)
					tips.ration_merge_into(80, 20, &mut split);
				}
				Treasury::on_unbalanced(split.0);
				Author::on_unbalanced(split.1);
			}
		}
	}

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Oldetp, extrinsic base weight (smallest non-zero weight) is mapped to 100 MILLI:
			let p = 100 * MILLI;
			let q = Balance::from(ExtrinsicBaseWeight::get());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational_approximation(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

pub mod wasm {
	//! Make the WASM binary available.

	#[cfg(all(feature = "std", any(target_arch = "x86_64", target_arch = "x86")))]
	include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

	#[cfg(all(feature = "std", not(any(target_arch = "x86_64", target_arch = "x86"))))]
	pub const WASM_BINARY: &[u8] = include_bytes!("../../../../wasm/hyperspace_runtime.compact.wasm");
	#[cfg(all(feature = "std", not(any(target_arch = "x86_64", target_arch = "x86"))))]
	pub const WASM_BINARY_BLOATY: &[u8] = include_bytes!("../../../../wasm/hyperspace_runtime.wasm");

	/// Wasm binary unwrapped. If built with `BUILD_DUMMY_WASM_BINARY`, the function panics.
	#[cfg(feature = "std")]
	pub fn wasm_binary_unwrap() -> &'static [u8] {
		#[cfg(all(feature = "std", any(target_arch = "x86_64", target_arch = "x86")))]
		return WASM_BINARY.expect(
			"Development wasm binary is not available. This means the client is \
			built with `SKIP_WASM_BUILD` flag and it is only usable for \
			production chains. Please rebuild with the flag disabled.",
		);
		#[cfg(all(feature = "std", not(any(target_arch = "x86_64", target_arch = "x86"))))]
		return WASM_BINARY;
	}
}

pub mod system;
pub use system::*;

pub mod babe;
pub use babe::*;

pub mod timestamp;
pub use timestamp::*;

pub mod balances;
pub use balances::*;

pub mod transaction_payment;
pub use transaction_payment::*;

pub mod authorship;
pub use authorship::*;

pub mod staking;
pub use staking::*;

pub mod offences;
pub use offences::*;

pub mod session_historical;
pub use session_historical::*;

pub mod session;
pub use session::*;

pub mod grandpa;
pub use grandpa::*;

pub mod im_online;
pub use im_online::*;

pub mod authority_discovery;
pub use authority_discovery::*;

pub mod democracy;
pub use democracy::*;

pub mod collective;
pub use collective::*;

pub mod elections_phragmen;
pub use elections_phragmen::*;

pub mod membership;
pub use membership::*;

pub mod treasury;
pub use treasury::*;

pub mod sudo;
pub use sudo::*;

pub mod claims;
pub use claims::*;

pub mod vesting;
pub use vesting::*;

pub mod utility;
pub use utility::*;

pub mod identity;
pub use identity::*;

pub mod society;
pub use society::*;

pub mod recovery;
pub use recovery::*;

pub mod scheduler;
pub use scheduler::*;

pub mod proxy;
pub use proxy::*;

pub mod multisig;
pub use multisig::*;

pub mod header_mmr;
pub use header_mmr::*;

pub mod oldetp_issuing;
pub use oldetp_issuing::*;

pub mod oldetp_backing;
pub use oldetp_backing::*;

pub mod ethereum_relay;
pub use ethereum_relay::*;

pub mod ethereum_backing;
pub use ethereum_backing::*;

pub mod relayer_game;
pub use relayer_game::*;

pub mod relay_authorities;
pub use relay_authorities::*;

pub mod oldna_backing;
pub use oldna_backing::*;

pub mod evm;
pub use evm::*;

pub mod dvm;
pub use dvm::*;

// --- hyperspace ---
pub use constants::*;
use hyperspace_evm::{Account as EVMAccount, FeeCalculator};
pub use hyperspace_staking::StakerStatus;
pub use hyperspace_primitives::*;
pub use impls::*;
pub use wasm::*;

// --- crates ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::{
	debug,
	traits::{KeyOwnerProofSystem, Randomness},
	weights::constants::ExtrinsicBaseWeight,
};
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_transaction_payment::FeeDetails;
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo as TransactionPaymentRuntimeDispatchInfo;
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
	create_runtime_str, generic,
	traits::{Block as BlockT, NumberFor, SaturatedConversion, StaticLookup},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiAddress, OpaqueExtrinsic, Perbill, RuntimeDebug,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
// --- hyperspace ---
use hyperspace_balances_rpc_runtime_api::RuntimeDispatchInfo as BalancesRuntimeDispatchInfo;
use hyperspace_evm::Runner;
use hyperspace_header_mmr_rpc_runtime_api::RuntimeDispatchInfo as HeaderMMRRuntimeDispatchInfo;
use hyperspace_staking_rpc_runtime_api::RuntimeDispatchInfo as StakingRuntimeDispatchInfo;
use dvm_rpc_runtime_api::TransactionStatus;

/// The address format for describing accounts.
type Address = MultiAddress<AccountId, ()>;
/// Block type as expected by this runtime.
type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	hyperspace_ethereum_relay::CheckEthereumRelayHeaderParcel<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Executive: handles dispatch to the various modules.
type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllModules,
	// CustomOnRuntimeUpgrade,
	PhragmenElectionDepositRuntimeUpgrade,
>;
/// The payload being signed in transactions.
type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

type Etp = Balances;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("Hyperspace"),
	impl_name: create_runtime_str!("Hyperspace"),
	authoring_version: 1,
	spec_version: 20,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

frame_support::construct_runtime! {
	pub enum Runtime
	where
		Block = Block,
		NodeBlock = OpaqueBlock,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Basic stuff; balances is uncallable initially.
		System: frame_system::{Module, Call, Storage, Config, Event<T>} = 0,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage} = 1,

		// Must be before session.
		Babe: pallet_babe::{Module, Call, Storage, Config, Inherent, ValidateUnsigned} = 2,

		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent} = 3,
		Balances: hyperspace_balances::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>} = 4,
		Dna: hyperspace_balances::<Instance1>::{Module, Call, Storage, Config<T>, Event<T>} = 5,
		TransactionPayment: pallet_transaction_payment::{Module, Storage} = 6,

		// Consensus support.
		Authorship: pallet_authorship::{Module, Call, Storage, Inherent} = 7,
		Staking: hyperspace_staking::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned} = 8,
		Offences: pallet_offences::{Module, Call, Storage, Event} = 9,
		Historical: pallet_session_historical::{Module} = 10,
		Session: pallet_session::{Module, Call, Storage, Config<T>, Event} = 11,
		Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event, ValidateUnsigned} = 12,
		ImOnline: pallet_im_online::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned} = 13,
		AuthorityDiscovery: pallet_authority_discovery::{Module, Call, Config} = 14,

		// Governance stuff; uncallable initially.
		Democracy: hyperspace_democracy::{Module, Call, Storage, Config, Event<T>} = 15,
		Council: pallet_collective::<Instance0>::{Module, Call, Storage, Origin<T>, Config<T>, Event<T>} = 16,
		TechnicalCommittee: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Config<T>, Event<T>} = 17,
		ElectionsPhragmen: hyperspace_elections_phragmen::{Module, Call, Storage, Config<T>, Event<T>} = 18,
		TechnicalMembership: pallet_membership::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>} = 19,
		Treasury: hyperspace_treasury::{Module, Call, Storage, Event<T>} = 20,

		Sudo: pallet_sudo::{Module, Call, Storage, Config<T>, Event<T>} = 21,

		// Claims. Usable initially.
		Claims: hyperspace_claims::{Module, Call, Storage, Config, Event<T>, ValidateUnsigned} = 22,

		// Vesting. Usable initially, but removed once all vesting is finished.
		Vesting: hyperspace_vesting::{Module, Call, Storage, Event<T>, Config<T>} = 23,

		// Utility module.
		Utility: pallet_utility::{Module, Call, Event} = 24,

		// Less simple identity module.
		Identity: pallet_identity::{Module, Call, Storage, Event<T>} = 25,

		// Society module.
		Society: pallet_society::{Module, Call, Storage, Event<T>} = 26,

		// Social recovery module.
		Recovery: pallet_recovery::{Module, Call, Storage, Event<T>} = 27,

		// System scheduler.
		Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>} = 28,

		// Proxy module. Late addition.
		Proxy: pallet_proxy::{Module, Call, Storage, Event<T>} = 29,

		// Multisig module. Late addition.
		Multisig: pallet_multisig::{Module, Call, Storage, Event<T>} = 30,

		HeaderMMR: hyperspace_header_mmr::{Module, Call, Storage} = 31,

		OldetpIssuing: hyperspace_oldetp_issuing::{Module, Call, Storage, Config, Event<T>} = 32,
		OldetpBacking: hyperspace_oldetp_backing::{Module, Storage, Config<T>} = 33,

		EthereumRelay: hyperspace_ethereum_relay::{Module, Call, Storage, Config<T>, Event<T>} = 34,
		EthereumBacking: hyperspace_ethereum_backing::{Module, Call, Storage, Config<T>, Event<T>} = 35,
		EthereumRelayerGame: hyperspace_relayer_game::<Instance0>::{Module, Storage} = 36,
		EthereumRelayAuthorities: hyperspace_relay_authorities::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>} = 37,

		OldnaBacking: hyperspace_oldna_backing::{Module, Storage, Config<T>} = 38,

		EVM: hyperspace_evm::{Module, Call, Storage, Config, Event<T>} = 39,
		Ethereum: dvm_ethereum::{Module, Call, Storage, Config, Event, ValidateUnsigned} = 40,
	}
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		public: <Signature as sp_runtime::traits::Verify>::Signer,
		account: AccountId,
		nonce: Nonce,
	) -> Option<(
		Call,
		<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		// take the biggest period possible.
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let tip = 0;
		let extra: SignedExtra = (
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			hyperspace_ethereum_relay::CheckEthereumRelayHeaderParcel::<Runtime>::new(),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				debug::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let (call, extra, _) = raw_payload.deconstruct();
		let address = <Runtime as frame_system::Config>::Lookup::unlookup(account);
		Some((call, (address, signature, extra)))
	}
}
impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
}
impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	Call: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = Call;
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(
			data: sp_inherents::InherentData
		) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			Historical::prove((fg_primitives::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(fg_primitives::OpaqueKeyOwnershipProof::new)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
			// The choice of `c` parameter (where `1 - c` represents the
			// probability of a slot being empty), is done in accordance to the
			// slot duration and expected target block time, for safely
			// resisting network delays of maximum two seconds.
			// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
			sp_consensus_babe::BabeGenesisConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDuration::get(),
				c: PRIMARY_PROBABILITY,
				genesis_authorities: Babe::authorities(),
				randomness: Babe::randomness(),
				allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
			}
		}

		fn current_epoch_start() -> sp_consensus_babe::SlotNumber {
			Babe::current_epoch_start()
		}

		fn current_epoch() -> sp_consensus_babe::Epoch {
			Babe::current_epoch()
		}

		fn next_epoch() -> sp_consensus_babe::Epoch {
			Babe::next_epoch()
		}

		fn generate_key_ownership_proof(
			_slot_number: sp_consensus_babe::SlotNumber,
			authority_id: sp_consensus_babe::AuthorityId,
		) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Babe::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}
	}

	impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityDiscoveryId> {
			AuthorityDiscovery::authorities()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> TransactionPaymentRuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl hyperspace_balances_rpc_runtime_api::BalancesApi<Block, AccountId, Balance> for Runtime {
		fn usable_balance(instance: u8, account: AccountId) -> BalancesRuntimeDispatchInfo<Balance> {
			match instance {
				0 => Etp::usable_balance_rpc(account),
				1 => Dna::usable_balance_rpc(account),
				_ => Default::default()
			}
		}
	}

	impl hyperspace_header_mmr_rpc_runtime_api::HeaderMMRApi<Block, Hash> for Runtime {
		fn gen_proof(
			block_number_of_member_leaf: u64,
			block_number_of_last_leaf: u64
		) -> HeaderMMRRuntimeDispatchInfo<Hash> {
			HeaderMMR::gen_proof_rpc(block_number_of_member_leaf, block_number_of_last_leaf )
		}
	}

	impl hyperspace_staking_rpc_runtime_api::StakingApi<Block, AccountId, Power> for Runtime {
		fn power_of(account: AccountId) -> StakingRuntimeDispatchInfo<Power> {
			Staking::power_of_rpc(account)
		}
	}

	impl dvm_rpc_runtime_api::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			<Runtime as hyperspace_evm::Config>::ChainId::get()
		}

		fn gas_price() -> U256 {
			<Runtime as hyperspace_evm::Config>::FeeCalculator::min_gas_price()
		}

		fn account_basic(address: H160) -> EVMAccount {
			// --- hyperspace ---
			use hyperspace_evm::AccountBasicMapping;

			<Runtime as hyperspace_evm::Config>::AccountBasicMapping::account_basic(&address)
		}

		fn account_code_at(address: H160) -> Vec<u8> {
			hyperspace_evm::Module::<Runtime>::account_codes(address)
		}

		fn author() -> H160 {
			<dvm_ethereum::Module<Runtime>>::find_author()
		}

		fn storage_at(address: H160, index: U256) -> H256 {
			let mut tmp = [0u8; 32];
			index.to_big_endian(&mut tmp);
			hyperspace_evm::Module::<Runtime>::account_storages(address, H256::from_slice(&tmp[..]))
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			gas_price: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
		) -> Result<hyperspace_evm::CallInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as hyperspace_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			<Runtime as hyperspace_evm::Config>::Runner::call(
				from,
				to,
				data,
				value,
				gas_limit.low_u64(),
				gas_price,
				nonce,
				config.as_ref().unwrap_or(<Runtime as hyperspace_evm::Config>::config()),
			).map_err(|err| err.into())
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			gas_price: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
		) -> Result<hyperspace_evm::CreateInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as hyperspace_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			<Runtime as hyperspace_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.low_u64(),
				gas_price,
				nonce,
				config.as_ref().unwrap_or(<Runtime as hyperspace_evm::Config>::config()),
			).map_err(|err| err.into())
		}


		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
			Ethereum::current_transaction_statuses()
		}

		fn current_block() -> Option<dvm_ethereum::Block> {
			Ethereum::current_block()
		}

		fn current_receipts() -> Option<Vec<dvm_ethereum::Receipt>> {
			Ethereum::current_receipts()
		}

		fn current_all() -> (
			Option<dvm_ethereum::Block>,
			Option<Vec<dvm_ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		) {
			(
				Ethereum::current_block(),
				Ethereum::current_receipts(),
				Ethereum::current_transaction_statuses()
			)
		}
	}
}

pub struct TransactionConverter;
impl dvm_rpc_runtime_api::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(&self, transaction: dvm_ethereum::Transaction) -> UncheckedExtrinsic {
		UncheckedExtrinsic::new_unsigned(
			<dvm_ethereum::Call<Runtime>>::transact(transaction).into(),
		)
	}
}
impl dvm_rpc_runtime_api::ConvertTransaction<OpaqueExtrinsic> for TransactionConverter {
	fn convert_transaction(&self, transaction: dvm_ethereum::Transaction) -> OpaqueExtrinsic {
		let extrinsic = UncheckedExtrinsic::new_unsigned(
			<dvm_ethereum::Call<Runtime>>::transact(transaction).into(),
		);
		let encoded = extrinsic.encode();

		OpaqueExtrinsic::decode(&mut &encoded[..]).expect("Encoded extrinsic is always valid")
	}
}

// pub struct CustomOnRuntimeUpgrade;
// impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
// 	fn on_runtime_upgrade() -> frame_support::weights::Weight {
// 		// --- substrate ---
// 		use frame_support::migration::*;

// 		MAXIMUM_BLOCK_WEIGHT
// 	}
// }

pub struct PhragmenElectionDepositRuntimeUpgrade;
impl hyperspace_elections_phragmen::migrations_2_0_0::ToV2 for PhragmenElectionDepositRuntimeUpgrade {
	type AccountId = AccountId;
	type Balance = Balance;
	type Module = ElectionsPhragmen;
}
impl frame_support::traits::OnRuntimeUpgrade for PhragmenElectionDepositRuntimeUpgrade {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		hyperspace_elections_phragmen::migrations_2_0_0::apply::<Self>(5 * MILLI, COIN)
	}
}
