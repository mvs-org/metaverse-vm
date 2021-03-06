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

//! # Hyperspace-ethereum-linear-relay Module
//!
//! Prototype module for bridging in Ethereum pow blockchain, including Mainnet and Ropsten.
//!
//! ## Overview
//!
//! The hyperspace eth linear relay module itself is a chain relay targeting Ethereum networks to
//! Hyperspace networks. This module follows the basic linear chain relay design which
//! requires relayers to relay the headers one by one.
//!
//! ### Relayer Incentive Model
//!
//! There is a points pool recording contribution of relayers, for each finalized and
//! confirmed block header, the relayer(origin) will get one unit of contribution point.
//! The income of the points pool come from two parts:
//! 	- The first part comes from clients who use chain relay to verify receipts, they
//!       might need to pay for the check_receipt service, although currently the chain
//!       relay didn't charge service fees, but in future, customers module/call should
//!       pay for this.
//!     - The second part comes from the compensate budget/proposal from system or governance,
//!       for example, someone may submit a proposal from treasury module to compensate the
//!       relay module account (points pool).
//!
//! The points owners can claim their incomes any time(block), the income is calculated according
//! to his points proportion of total points, and after paying to him, the points will be destroyed
//! from the points pool.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]

pub mod weights;
// --- hyperspace ---
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod types {
	// --- hyperspace ---
	use crate::*;

	pub type Balance<T> = <CurrencyT<T> as Currency<AccountId<T>>>::Balance;

	type AccountId<T> = <T as frame_system::Config>::AccountId;

	type CurrencyT<T> = <T as Config>::Currency;
}

// --- crates ---
use codec::{Decode, Encode};
// --- github ---
use ethereum_types::H128;
// --- substrate ---
use frame_support::{
	debug::trace,
	decl_error, decl_event, decl_module, decl_storage, ensure,
	traits::Get,
	traits::{Currency, ExistenceRequirement::KeepAlive, IsSubType, ReservableCurrency},
};
use frame_system::{ensure_root, ensure_signed};
use sp_runtime::{
	traits::{AccountIdConversion, DispatchInfoOf, Dispatchable, Saturating, SignedExtension},
	transaction_validity::{
		InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
	},
	DispatchError, DispatchResult, ModuleId, SaturatedConversion,
};
use sp_std::prelude::*;
// --- hyperspace ---
use hyperspace_support::{
	balance::lock::LockableCurrency, traits::EthereumReceipt as EthereumReceiptT,
};
use ethereum_primitives::{
	ethashproof::EthashProof,
	header::EthereumHeader,
	pow::EthashPartial,
	receipt::{EthereumReceipt, EthereumReceiptProof, EthereumTransactionIndex},
	EthereumBlockNumber, EthereumNetworkType, H256, U256,
};
use types::*;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/dags_merkle_roots.rs"));

pub trait Config: frame_system::Config {
	/// The ethereum-linear-relay's module id, used for deriving its sovereign account ID.
	type ModuleId: Get<ModuleId>;

	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	type EthereumNetwork: Get<EthereumNetworkType>;

	type Call: Dispatchable + From<Call<Self>> + IsSubType<Call<Self>> + Clone;

	type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
		+ ReservableCurrency<Self::AccountId>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

decl_event! {
	pub enum Event<T>
	where
		<T as frame_system::Config>::AccountId,
		Balance = Balance<T>,
	{
		SetGenesisHeader(EthereumHeader, u64),
		RelayHeader(AccountId, EthereumHeader),
		VerifyProof(AccountId, EthereumReceipt, EthereumReceiptProof),
		AddAuthority(AccountId),
		RemoveAuthority(AccountId),
		ToggleCheckAuthorities(bool),
		ClaimReward(AccountId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Account - NO PRIVILEGES
		AccountNP,

		/// Block Number - OVERFLOW
		BlockNumberOF,
		/// Block Number - UNDERFLOW
		BlockNumberUF,

		/// Block Number - MISMATCHED
		BlockNumberMis,
		/// Header Hash - MISMATCHED
		HeaderHashMis,
		/// Mixhash - MISMATCHED
		MixHashMis,

		/// Begin Header - NOT EXISTED
		BeginHeaderNE,
		/// Header - NOT EXISTED
		HeaderNE,
		/// Header Brief - NOT EXISTED
		HeaderBriefNE,

		/// Header - ALREADY EXISTED
		HeaderAE,
		/// Header - TOO EARLY
		HeaderTE,
		/// Header - TOO OLD,
		HeaderTO,

		/// Rlp - DECODE FAILED
		RlpDcF,
		/// Ethereum Receipt Proof - INVALID
		ReceiptProofInv,
		/// Block Basic - VERIFICATION FAILED
		BlockBasicVF,
		/// Difficulty - VERIFICATION FAILED
		DifficultyVF,

		/// Payout - NO POINTS OR FUNDS
		PayoutNPF,
	}
}

#[cfg(feature = "std")]
hyperspace_support::impl_genesis! {
	struct DagsMerkleRootsLoader {
		dags_merkle_roots: Vec<H128>
	}
}
decl_storage! {
	trait Store for Module<T: Config> as HyperspaceEthereumLinearRelay {
		/// Anchor block that works as genesis block
		pub GenesisHeader get(fn begin_header): Option<EthereumHeader>;

		/// Dags merkle roots of ethereum epoch (each epoch is 30000)
		pub DagsMerkleRoots get(fn dag_merkle_root): map hasher(identity) u64 => H128;

		/// Hash of best block header
		pub BestHeaderHash get(fn best_header_hash): H256;

		pub CanonicalHeaderHashes get(fn canonical_header_hash): map hasher(identity) u64 => H256;

		pub Headers get(fn header): map hasher(identity) H256 => Option<EthereumHeader>;
		pub HeaderBriefs get(fn header_brief): map hasher(identity) H256 => Option<EthereumHeaderBrief<T::AccountId>>;

		/// Number of blocks finality
		pub NumberOfBlocksFinality get(fn number_of_blocks_finality) config(): u64;
		pub NumberOfBlocksSafe get(fn number_of_blocks_safe) config(): u64;

		pub CheckAuthority get(fn check_authority) config(): bool = true;
		pub Authorities get(fn authorities) config(): Vec<T::AccountId>;

		pub ReceiptVerifyFee get(fn receipt_verify_fee) config(): Balance<T>;

		pub RelayerPoints get(fn relayer_points): map hasher(blake2_128_concat) T::AccountId => u64;
		pub TotalRelayerPoints get(fn total_points): u64 = 0;
	}
	add_extra_genesis {
		// genesis: Option<Header, Difficulty>
		config(genesis_header): Option<(u64, Vec<u8>)>;
		config(dags_merkle_roots_loader): DagsMerkleRootsLoader;
		build(|config| {
			let GenesisConfig {
				genesis_header,
				dags_merkle_roots_loader,
				..
			} = config;

			if let Some((total_difficulty, header)) = genesis_header {
				if let Ok(header) = rlp::decode(&header) {
					<Module<T>>::init_genesis_header(&header, *total_difficulty).unwrap();
				} else {
					panic!("{}", <&str>::from(<Error<T>>::RlpDcF));
				}
			}

			let dags_merkle_roots = if dags_merkle_roots_loader.dags_merkle_roots.is_empty() {
				DagsMerkleRootsLoader::from_str(DAGS_MERKLE_ROOTS_STR).dags_merkle_roots.clone()
			} else {
				dags_merkle_roots_loader.dags_merkle_roots.clone()
			};
			for (i, dag_merkle_root) in dags_merkle_roots.into_iter().enumerate() {
				DagsMerkleRoots::insert(i as u64, dag_merkle_root);
			}

			let _ = T::Currency::make_free_balance_be(
				&<Module<T>>::account_id(),
				T::Currency::minimum_balance(),
			);
		});
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call
	where
		origin: T::Origin
	{
		type Error = Error<T>;

		const ModuleId: ModuleId = T::ModuleId::get();

		const EthereumNetwork: EthereumNetworkType = T::EthereumNetwork::get();

		fn deposit_event() = default;

		/// Relay header of eth block, store the passing header
		/// if it is verified.
		///
		/// # <weight>
		/// - `O(1)`, but takes a lot of computation works
		/// - Limited Storage reads
		/// - One storage read
		/// - One storage write
		/// - Up to one event
		/// # </weight>
		#[weight = 200_000_000]
		pub fn relay_header(origin, header: EthereumHeader, ethash_proof: Vec<EthashProof>) {
			trace!(target: "ethereum-linear-relay", "{:?}", header);
			let relayer = ensure_signed(origin)?;

			if Self::check_authority() {
				ensure!(Self::authorities().contains(&relayer), <Error<T>>::AccountNP);
			}

			let header_hash = header.hash();

			ensure!(<HeaderBriefs<T>>::get(&header_hash).is_none(), <Error<T>>::HeaderAE);

			// 1. proof of difficulty
			// 2. proof of pow (mixhash)
			// 3. challenge
			{
				Self::verify_header_basic(&header)?;
				Self::verify_header_pow(&header, &ethash_proof)?;
			}

			Self::maybe_store_header(&relayer, &header)?;

			<Module<T>>::deposit_event(RawEvent::RelayHeader(relayer, header));
		}

		/// Check receipt
		///
		/// # <weight>
		/// - `O(1)`.
		/// - Limited Storage reads
		/// - Up to one event
		/// # </weight>
		#[weight = 100_000_000]
		pub fn check_receipt(origin, proof_record: EthereumReceiptProof) {
			let worker = ensure_signed(origin)?;

			let verified_receipt =
				Self::verify_receipt(&proof_record).map_err(|_| <Error<T>>::ReceiptProofInv)?;
			let fee = Self::receipt_verify_fee();

			let module_account = Self::account_id();

			T::Currency::transfer(&worker, &module_account, fee, KeepAlive)?;

			<Module<T>>::deposit_event(RawEvent::VerifyProof(worker, verified_receipt, proof_record));
		}

		/// Claim Reward for Relayers
		///
		/// # <weight>
		/// - `O(1)`.
		/// - Limited Storage reads
		/// - Up to one event
		/// # </weight>
		#[weight = 10_000_000]
		pub fn claim_reward(origin) {
			let relayer = ensure_signed(origin)?;

			let points = Self::relayer_points(&relayer);
			let total_points = Self::total_points();

			let max_payout = Self::pot().saturated_into::<u128>();

			ensure!(total_points > 0 && points > 0 && max_payout > 0 && total_points >= points, <Error<T>>::PayoutNPF);

			let payout = <Balance<T>>::saturated_from(points as u128 * max_payout / (total_points  as u128));
			let module_account = Self::account_id();

			T::Currency::transfer(&module_account, &relayer, payout, KeepAlive)?;

			<RelayerPoints<T>>::remove(&relayer);

			TotalRelayerPoints::mutate(|p| *p -= points);

			<Module<T>>::deposit_event(RawEvent::ClaimReward(relayer, payout));
		}

		// --- root call ---

		#[weight = 100_000_000]
		pub fn reset_genesis_header(origin, header: EthereumHeader, genesis_difficulty: u64) {
			let _ = ensure_root(origin)?;

			Self::init_genesis_header(&header, genesis_difficulty)?;

			<Module<T>>::deposit_event(RawEvent::SetGenesisHeader(header, genesis_difficulty));
		}

		/// Add authority
		///
		/// # <weight>
		/// - `O(A)` where `A` length of `authorities`
		/// - One storage mutation (codec `O(A)`).
		/// - Up to one event
		/// # </weight>
		#[weight = 50_000_000]
		pub fn add_authority(origin, who: T::AccountId) {
			ensure_root(origin)?;

			if !Self::authorities().contains(&who) {
				<Authorities<T>>::mutate(|l| l.push(who.clone()));

				<Module<T>>::deposit_event(RawEvent::AddAuthority(who));
			}
		}

		/// Remove authority
		///
		/// # <weight>
		/// - `O(A)` where `A` length of `authorities`
		/// - One storage mutation (codec `O(A)`).
		/// - Up to one event
		/// # </weight>
		#[weight = 50_000]
		pub fn remove_authority(origin, who: T::AccountId) {
			ensure_root(origin)?;

			if let Some(i) = Self::authorities()
				.into_iter()
				.position(|who_| who_ == who) {
				<Authorities<T>>::mutate(|l| l.remove(i));

				<Module<T>>::deposit_event(RawEvent::RemoveAuthority(who));
			}
		}

		/// Check authorities
		///
		/// # <weight>
		/// - `O(1)`.
		/// - One storage write
		/// - Up to one event
		/// # </weight>
		#[weight = 10_000_000]
		pub fn toggle_check_authorities(origin) {
			ensure_root(origin)?;

			CheckAuthority::put(!Self::check_authority());

			<Module<T>>::deposit_event(RawEvent::ToggleCheckAuthorities(Self::check_authority()));
		}

		/// Set number of blocks finality
		///
		/// # <weight>
		/// - `O(1)`.
		/// - One storage write
		/// # </weight>
		#[weight = 10_000_000]
		pub fn set_number_of_blocks_finality(origin, #[compact] new: u64) {
			ensure_root(origin)?;

			let old_number = NumberOfBlocksFinality::get();
			let best_header_info = Self::header_brief(Self::best_header_hash()).ok_or(<Error<T>>::HeaderBriefNE);
			if new < old_number && best_header_info.is_ok() {
				let best_header_info_number = best_header_info.unwrap().number;

				for i in 0..(old_number - new) {
					// Adding reward points to the relayer of finalized block
					if best_header_info_number > Self::number_of_blocks_finality() + i + 1 {
						let finalized_block_number = best_header_info_number - Self::number_of_blocks_finality() - i - 1;
						let finalized_block_hash = CanonicalHeaderHashes::get(finalized_block_number);
						if let Some(info) = <HeaderBriefs<T>>::get(finalized_block_hash) {
							let points: u64 = Self::relayer_points(&info.relayer);

							<RelayerPoints<T>>::insert(info.relayer, points + 1);

							TotalRelayerPoints::put(Self::total_points() + 1);
						}
					}
				}
			} else {
				// Finality interval becomes larger, some points might already been claimed.
				// But we just ignore possible double claimed in future here.
			}

			NumberOfBlocksFinality::put(new);
		}

		/// Set number of blocks finality
		///
		/// # <weight>
		/// - `O(1)`.
		/// - One storage write
		/// # </weight>
		#[weight = 10_000_000]
		pub fn set_number_of_blocks_safe(origin, #[compact] new: u64) {
			ensure_root(origin)?;
			NumberOfBlocksSafe::put(new);
		}

		/// Set verify receipt fee
		///
		/// # <weight>
		/// - `O(1)`.
		/// - One storage write
		/// # </weight>
		#[weight = 10_000_000]
		pub fn set_receipt_verify_fee(origin, #[compact] new: Balance<T>) {
			ensure_root(origin)?;
			<ReceiptVerifyFee<T>>::put(new);
		}
	}
}

impl<T: Config> Module<T> {
	/// The account ID of the ethereum linear relay pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}

	/// Return the amount of money in the pot.
	// The existential deposit is not part of the pot so ethereum-linear-relay account never gets deleted.
	fn pot() -> Balance<T> {
		T::Currency::usable_balance(&Self::account_id())
			// Must never be less than 0 but better be safe.
			.saturating_sub(T::Currency::minimum_balance())
	}

	pub fn init_genesis_header(
		header: &EthereumHeader,
		genesis_total_difficulty: u64,
	) -> DispatchResult {
		let header_hash = header.hash();

		ensure!(
			header_hash == header.re_compute_hash(),
			<Error<T>>::HeaderHashMis
		);

		let block_number = header.number;

		Headers::insert(&header_hash, header);

		// initialize header info, including total difficulty.
		<HeaderBriefs<T>>::insert(
			&header_hash,
			EthereumHeaderBrief::<T::AccountId> {
				parent_hash: header.parent_hash,
				total_difficulty: genesis_total_difficulty.into(),
				number: block_number,
				relayer: Default::default(),
			},
		);

		// Initialize the the best hash.
		BestHeaderHash::put(header_hash);

		CanonicalHeaderHashes::insert(block_number, header_hash);

		// Removing header with larger numbers, if there are.
		for number in block_number
			.checked_add(1)
			.ok_or(<Error<T>>::BlockNumberOF)?..u64::max_value()
		{
			// If the current block hash is 0 (unlikely), or the previous hash matches the
			// current hash, then we chains converged and can stop now.
			if !CanonicalHeaderHashes::contains_key(&number) {
				break;
			}

			CanonicalHeaderHashes::remove(&number);
		}

		GenesisHeader::put(header.clone());

		Ok(())
	}

	fn verify_header_basic(header: &EthereumHeader) -> DispatchResult {
		ensure!(
			header.hash() == header.re_compute_hash(),
			<Error<T>>::HeaderHashMis
		);
		trace!(target: "ethereum-linear-relay", "Hash OK");

		let begin_header_number = Self::begin_header()
			.ok_or(<Error<T>>::BeginHeaderNE)?
			.number;
		ensure!(header.number >= begin_header_number, <Error<T>>::HeaderTE);
		trace!(target: "ethereum-linear-relay", "Number1 OK");

		// There must be a corresponding parent hash
		let prev_header = Self::header(header.parent_hash).ok_or(<Error<T>>::HeaderNE)?;
		// block number was verified in `re_compute_hash`,`u64` is enough; qed
		ensure!(
			header.number == prev_header.number + 1,
			<Error<T>>::BlockNumberMis
		);
		trace!(target: "ethereum-linear-relay", "Number2 OK");

		// check difficulty
		let ethash_params = match T::EthereumNetwork::get() {
			EthereumNetworkType::Mainnet => EthashPartial::production(),
			EthereumNetworkType::Ropsten => EthashPartial::ropsten_testnet(),
		};
		ethash_params
			.verify_block_basic(header)
			.map_err(|_| <Error<T>>::BlockBasicVF)?;
		trace!(target: "ethereum-linear-relay", "Basic OK");

		// verify difficulty
		let difficulty = ethash_params.calculate_difficulty(header, &prev_header);
		ensure!(difficulty == *header.difficulty(), <Error<T>>::DifficultyVF);
		trace!(target: "ethereum-linear-relay", "Difficulty OK");

		Ok(())
	}

	fn verify_header_pow(header: &EthereumHeader, ethash_proof: &[EthashProof]) -> DispatchResult {
		Self::verify_header_basic(&header)?;

		let ethash_params = match T::EthereumNetwork::get() {
			EthereumNetworkType::Mainnet => EthashPartial::production(),
			EthereumNetworkType::Ropsten => EthashPartial::ropsten_testnet(),
		};

		let merkle_root = Self::dag_merkle_root((header.number as usize / 30000) as u64);
		if ethash_params
			.verify_seal_with_proof(&header, &ethash_proof, &merkle_root)
			.is_err()
		{
			Err(<Error<T>>::MixHashMis)?;
		};
		trace!(target: "ethereum-linear-relay", "MixHash OK");

		// TODO: Check other verification condition
		// See YellowPaper formula (50) in section 4.3.4
		// 1. Simplified difficulty check to conform adjusting difficulty bomb
		// 2. Added condition: header.parent_hash() == prev.hash()
		//
		//			ethereum_types::U256::from((result.0).0) < ethash::cross_boundary(header.difficulty.0)
		//				&& (
		//				!self.validate_ethash
		//					|| (
		//					header.difficulty < header.difficulty * 101 / 100
		//						&& header.difficulty > header.difficulty * 99 / 100
		//				)
		//			)
		//				&& header.gas_used <= header.gas_limit
		//				&& header.gas_limit < prev.gas_limit * 1025 / 1024
		//				&& header.gas_limit > prev.gas_limit * 1023 / 1024
		//				&& header.gas_limit >= U256(5000.into())
		//				&& header.timestamp > prev.timestamp
		//				&& header.number == prev.number + 1
		//				&& header.parent_hash == prev.hash.unwrap()

		Ok(())
	}

	fn maybe_store_header(relayer: &T::AccountId, header: &EthereumHeader) -> DispatchResult {
		let best_header_info =
			Self::header_brief(Self::best_header_hash()).ok_or(<Error<T>>::HeaderBriefNE)?;

		ensure!(
			best_header_info.number
				<= header
					.number
					.checked_add(Self::number_of_blocks_finality())
					.ok_or(<Error<T>>::BlockNumberOF)?,
			<Error<T>>::HeaderTO,
		);

		let parent_total_difficulty = Self::header_brief(header.parent_hash)
			.ok_or(<Error<T>>::HeaderBriefNE)?
			.total_difficulty;

		let header_hash = header.hash();
		let header_brief = EthereumHeaderBrief::<T::AccountId> {
			number: header.number,
			parent_hash: header.parent_hash,
			total_difficulty: parent_total_difficulty
				.checked_add(header.difficulty)
				.ok_or(<Error<T>>::BlockNumberOF)?,
			relayer: relayer.clone(),
		};

		// Check total difficulty and re-org if necessary.
		if header_brief.total_difficulty > best_header_info.total_difficulty
			|| (header_brief.total_difficulty == best_header_info.total_difficulty
				&& header.difficulty % 2 == U256::zero())
		{
			// The new header is the tip of the new canonical chain.
			// We need to update hashes of the canonical chain to match the new header.

			// If the new header has a lower number than the previous header, we need to cleaning
			// it going forward.
			if best_header_info.number > header_brief.number {
				for number in header_brief
					.number
					.checked_add(1)
					.ok_or(<Error<T>>::BlockNumberOF)?..=best_header_info.number
				{
					CanonicalHeaderHashes::remove(&number);
				}
			}
			// Replacing the global best header hash.
			BestHeaderHash::put(header_hash);

			CanonicalHeaderHashes::insert(header_brief.number, header_hash);

			// Replacing past hashes until we converge into the same parent.
			// Starting from the parent hash.
			let mut current_hash = header_brief.parent_hash;
			for number in (0..=header
				.number
				.checked_sub(1)
				.ok_or(<Error<T>>::BlockNumberUF)?)
				.rev()
			{
				let prev_value = CanonicalHeaderHashes::get(number);
				// If the current block hash is 0 (unlikely), or the previous hash matches the
				// current hash, then we chains converged and can stop now.
				if number == 0 || prev_value == current_hash {
					break;
				}

				CanonicalHeaderHashes::insert(number, current_hash);

				// Check if there is an info to get the parent hash
				if let Some(info) = <HeaderBriefs<T>>::get(current_hash) {
					current_hash = info.parent_hash;
				} else {
					break;
				}
			}
		}

		// Adding reward points to the relayer of finalized block
		if header.number > Self::number_of_blocks_finality() {
			let finalized_block_number = header.number - Self::number_of_blocks_finality() - 1;
			let finalized_block_hash = CanonicalHeaderHashes::get(finalized_block_number);
			if let Some(info) = <HeaderBriefs<T>>::get(finalized_block_hash) {
				let points: u64 = Self::relayer_points(&info.relayer);

				<RelayerPoints<T>>::insert(info.relayer, points + 1);

				TotalRelayerPoints::put(Self::total_points() + 1);
			}
		}

		Headers::insert(header_hash, header);
		<HeaderBriefs<T>>::insert(header_hash, header_brief.clone());

		Ok(())
	}
}

impl<T: Config> EthereumReceiptT<T::AccountId, Balance<T>> for Module<T> {
	type EthereumReceiptProofThing = EthereumReceiptProof;

	fn account_id() -> T::AccountId {
		Self::account_id()
	}

	fn receipt_verify_fee() -> Balance<T> {
		Self::receipt_verify_fee()
	}

	/// confirm that the block hash is right
	/// get the receipt MPT trie root from the block header
	/// Using receipt MPT trie root to verify the proof and index etc.
	fn verify_receipt(
		proof: &Self::EthereumReceiptProofThing,
	) -> Result<EthereumReceipt, DispatchError> {
		let info = Self::header_brief(&proof.header_hash).ok_or(<Error<T>>::HeaderBriefNE)?;

		let canonical_hash = Self::canonical_header_hash(info.number);
		ensure!(
			canonical_hash == proof.header_hash,
			<Error<T>>::HeaderHashMis
		);

		let best_info =
			Self::header_brief(Self::best_header_hash()).ok_or(<Error<T>>::HeaderBriefNE)?;

		ensure!(
			best_info.number
				>= info
					.number
					.checked_add(Self::number_of_blocks_safe())
					.ok_or(<Error<T>>::BlockNumberOF)?,
			<Error<T>>::BlockNumberMis
		);

		let header = Self::header(&proof.header_hash).ok_or(<Error<T>>::HeaderNE)?;

		// Verify receipt proof
		let receipt = EthereumReceipt::verify_proof_and_generate(header.receipts_root(), &proof)
			.map_err(|_| <Error<T>>::ReceiptProofInv)?;

		Ok(receipt)
	}

	fn gen_receipt_index(proof: &Self::EthereumReceiptProofThing) -> EthereumTransactionIndex {
		(proof.header_hash, proof.index)
	}
}

/// `SignedExtension` that checks if a transaction has duplicate header hash to avoid coincidence
/// header between several relayers
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct CheckEthereumRelayHeaderParcel<T: Config + Send + Sync>(sp_std::marker::PhantomData<T>);
impl<T: Config + Send + Sync> Default for CheckEthereumRelayHeaderParcel<T> {
	fn default() -> Self {
		Self(sp_std::marker::PhantomData)
	}
}
impl<T: Config + Send + Sync> sp_std::fmt::Debug for CheckEthereumRelayHeaderParcel<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "CheckEthereumRelayHeaderParcel")
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}
impl<T: Config + Send + Sync> SignedExtension for CheckEthereumRelayHeaderParcel<T> {
	const IDENTIFIER: &'static str = "CheckEthereumRelayHeaderParcel";
	type AccountId = T::AccountId;
	type Call = <T as Config>::Call;
	type AdditionalSigned = ();
	type Pre = ();

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn validate(
		&self,
		_who: &Self::AccountId,
		call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		let call = match call.is_sub_type() {
			Some(call) => call,
			None => return Ok(ValidTransaction::default()),
		};

		match call {
			Call::relay_header(ref header, _) => {
				sp_runtime::print("check ethereum-linear-relay header hash was received.");
				let header_hash = header.hash();

				if <HeaderBriefs<T>>::get(&header_hash).is_none() {
					Ok(ValidTransaction::default())
				} else {
					InvalidTransaction::Custom(<Error<T>>::HeaderAE.as_u8()).into()
				}
			}
			_ => Ok(Default::default()),
		}
	}
}

/// Familial details concerning a block
#[derive(Clone, Default, PartialEq, Encode, Decode)]
pub struct EthereumHeaderBrief<AccountId> {
	/// Total difficulty of the block and all its parents
	pub total_difficulty: U256,
	/// Parent hash of the header
	pub parent_hash: H256,
	/// Block number
	pub number: EthereumBlockNumber,
	/// Relayer of the block header
	pub relayer: AccountId,
}
