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

//! MMR for Ethereum
//! No mater the hash function of chain,
//! the Merge of Ethereum MMR used in shadow service is blake2b

// --- crates ---
pub use ckb_merkle_mountain_range::{
	leaf_index_to_mmr_size, leaf_index_to_pos, Merge, MerkleProof,
};

// ---crates ---
use blake2_rfc::blake2b::blake2b;
// --- substrate ---
use sp_std::vec;

/// BlakeTwo256 hash function
pub fn hash(data: &[u8]) -> [u8; 32] {
	let mut dest = [0; 32];
	dest.copy_from_slice(blake2b(32, &[], data).as_bytes());
	dest
}

/// MMR Merge for MMR Merge trait
pub struct MMRMerge;
impl Merge for MMRMerge {
	type Item = [u8; 32];
	fn merge(lhs: &Self::Item, rhs: &Self::Item) -> Self::Item {
		let mut data = vec![];
		data.append(&mut lhs.to_vec());
		data.append(&mut rhs.to_vec());
		hash(&data.as_slice())
	}
}
