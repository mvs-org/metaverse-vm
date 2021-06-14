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

//! # Chain MMR Pallet
//!
//! ## Overview
//! This is the pallet to maintain accumulate headers Merkle Mountain Range
//! and push the mmr root in to the digest of block headers on finalize.
//! MMR can be used for light client to implement super light clients,
//! and can also be used in other chains to implement chain relay for
//! cross-chain verification purpose.
//!
//! ## Terminology
//!
//! ### Merkle Mountain Range
//! For more details about the MMR struct, refer https://github.com/mimblewimble/grin/blob/master/doc/mmr.md#structure
//!
//! ### MMR Proof
//! Using the MMR Store Storage, MMR Proof can be generated for specific
//! block header hash. Proofs can be used to verify block inclusion together with
//! the mmr root in the header digest.
//!
//! ### Digest Item
//! The is a ```MerkleMountainRangeRoot(Hash)``` digest item pre-subscribed in Digest.
//! This is implemented in Hyperspace's fork of substrate: https://github.com/new-mvs/substrate
//! The Pull request link is https://github.com/new-mvs/substrate/pull/1
//!
//! ## Implementation
//! We are using the MMR library from https://github.com/nervosnetwork/merkle-mountain-range
//! Pull request: https://github.com/new-mvs/hyperspace/pull/358
//!
//! ## References
//! Hyperspace Relay's Technical Paper:
//! https://github.com/new-mvs/rfcs/blob/master/paper/Hyperspace_Relay_Sublinear_Optimistic_Relay_for_Interoperable_Blockchains_v0.7.pdf
//!
//! https://github.com/mimblewimble/grin/blob/master/doc/mmr.md#structure
//! https://github.com/mimblewimble/grin/blob/0ff6763ee64e5a14e70ddd4642b99789a1648a32/core/src/core/pmmr.rs#L606
//! https://github.com/nervosnetwork/merkle-mountain-range/blob/master/src/tests/test_accumulate_headers.rs
//! https://eprint.iacr.org/2019/226.pdf
//!

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

#[cfg(feature = "std")]
use serde::Serialize;

// --- github ---
use merkle_mountain_range::{
	leaf_index_to_mmr_size, leaf_index_to_pos, MMRStore, Result as MMRResult, MMR,
};
// --- substrate ---
use codec::{Decode, Encode};
use frame_support::{debug::error, decl_module, decl_storage};
use sp_runtime::{
	generic::{DigestItem, OpaqueDigestItemId},
	traits::{Hash, Header},
	RuntimeDebug, SaturatedConversion,
};
use sp_std::{marker::PhantomData, prelude::*};
// --- hyperspace ---
use hyperspace_header_mmr_rpc_runtime_api::{Proof, RuntimeDispatchInfo};
use hyperspace_relay_primitives::MMR as MMRT;
use hyperspace_support::impl_rpc;

pub const PARENT_MMR_ROOT_LOG_ID: [u8; 4] = *b"MMRR";

#[cfg_attr(feature = "std", derive(Serialize))]
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct MerkleMountainRangeRootLog<Hash> {
	/// Specific prefix to identify the mmr root log in the digest items with Other type.
	pub prefix: [u8; 4],
	/// The merkle mountain range root hash.
	pub parent_mmr_root: Hash,
}

pub trait Config: frame_system::Config {}

decl_storage! {
	trait Store for Module<T: Config> as HyperspaceHeaderMMR {
		/// MMR struct of the previous blocks, from first(genesis) to parent hash.
		pub MMRNodeList get(fn mmr_node_list): map hasher(identity) u64 => Option<T::Hash>;

		/// The MMR size and length of the mmr node list
		pub MMRCounter get(fn mmr_counter): u64;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call
	where
		origin: T::Origin
	{
		fn on_finalize(_block_number: T::BlockNumber) {
			let store = <ModuleMMRStore<T>>::default();
			let parent_hash = <frame_system::Module<T>>::parent_hash();
			let mut mmr = <MMR<_, MMRMerge<T>, _>>::new(MMRCounter::get(), store);

			// Update MMR and add mmr root to digest of block header
			let _ = mmr.push(parent_hash);

			if let Ok(parent_mmr_root) = mmr.get_root() {
				if mmr.commit().is_ok() {
					let mmr_root_log = MerkleMountainRangeRootLog::<T::Hash> {
						prefix: PARENT_MMR_ROOT_LOG_ID,
						parent_mmr_root: parent_mmr_root.into()
					};
					let mmr_item = DigestItem::Other(mmr_root_log.encode());

					<frame_system::Module<T>>::deposit_log(mmr_item.into());
				} else {
					error!("[hyperspace-header-mmr] FAILED to Commit MMR");
				}
			} else {
				error!("[hyperspace-header-mmr] FAILED to Calculate MMR");
			}
		}
	}
}

impl<T: Config> Module<T> {
	impl_rpc! {
		pub fn gen_proof_rpc(
			block_number_of_member_leaf: u64,
			block_number_of_last_leaf: u64,
		) -> RuntimeDispatchInfo<T::Hash> {
			if block_number_of_member_leaf <= block_number_of_last_leaf {
				let store = <ModuleMMRStore<T>>::default();
				let mmr_size = leaf_index_to_mmr_size(block_number_of_last_leaf);
				if mmr_size <= MMRCounter::get() {
					let mmr = <MMR<_, MMRMerge<T>, _>>::new(mmr_size, store);
					let pos = leaf_index_to_pos(block_number_of_member_leaf);

					if let Ok(merkle_proof) = mmr.gen_proof(vec![pos]) {
						return RuntimeDispatchInfo {
							mmr_size,
							proof: Proof(merkle_proof.proof_items().to_vec()),
						};
					}
				}
			}

			RuntimeDispatchInfo {
				mmr_size: 0,
				proof: Proof(vec![]),
			}
		}
	}

	// TODO: For future rpc calls
	fn _find_parent_mmr_root(header: T::Header) -> Option<T::Hash> {
		let id = OpaqueDigestItemId::Other;

		let filter_log = |MerkleMountainRangeRootLog {
		                      prefix,
		                      parent_mmr_root,
		                  }: MerkleMountainRangeRootLog<T::Hash>| match prefix {
			PARENT_MMR_ROOT_LOG_ID => Some(parent_mmr_root),
			_ => None,
		};

		// find the first other digest with the right prefix which converts to
		// the right kind of mmr root log.
		header
			.digest()
			.convert_first(|l| l.try_to(id).and_then(filter_log))
	}
}

pub struct MMRMerge<T>(PhantomData<T>);
impl<T: Config> merkle_mountain_range::Merge for MMRMerge<T> {
	type Item = <T as frame_system::Config>::Hash;

	fn merge(lhs: &Self::Item, rhs: &Self::Item) -> Self::Item {
		let encodable = (lhs, rhs);
		<T as frame_system::Config>::Hashing::hash_of(&encodable)
	}
}

pub struct ModuleMMRStore<T>(PhantomData<T>);
impl<T> Default for ModuleMMRStore<T> {
	fn default() -> Self {
		ModuleMMRStore(sp_std::marker::PhantomData)
	}
}
impl<T: Config> MMRStore<T::Hash> for ModuleMMRStore<T> {
	fn get_elem(&self, pos: u64) -> MMRResult<Option<T::Hash>> {
		Ok(<Module<T>>::mmr_node_list(pos))
	}

	fn append(&mut self, pos: u64, elems: Vec<T::Hash>) -> MMRResult<()> {
		let mmr_count = MMRCounter::get();
		if pos != mmr_count {
			// Must be append only.
			Err(merkle_mountain_range::Error::InconsistentStore)?;
		}
		let elems_len = elems.len() as u64;

		for (i, elem) in elems.into_iter().enumerate() {
			<MMRNodeList<T>>::insert(mmr_count + i as u64, elem);
		}

		// increment counter
		MMRCounter::put(mmr_count + elems_len);

		Ok(())
	}
}

impl<T: Config> MMRT<T::BlockNumber, T::Hash> for Module<T> {
	fn get_root(block_number: T::BlockNumber) -> Option<T::Hash> {
		let store = <ModuleMMRStore<T>>::default();
		let mmr_size = leaf_index_to_mmr_size(block_number.saturated_into::<u64>() as _);
		let mmr = <MMR<_, MMRMerge<T>, _>>::new(mmr_size, store);

		if let Ok(mmr_root) = mmr.get_root() {
			Some(mmr_root)
		} else {
			None
		}
	}
}
