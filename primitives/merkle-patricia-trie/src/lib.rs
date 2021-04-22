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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod trie;

mod db;
mod error;
mod nibbles;
mod node;
mod proof;
mod tests;

pub use db::MemoryDB;
pub use error::TrieError;
pub use proof::Proof;
pub use trie::{MerklePatriciaTrie, Trie, TrieResult};

use sp_std::rc::Rc;

/// Generates a trie for a vector of key-value tuples
///
/// ```rust
/// extern crate merkle_patricia_trie as trie;
/// extern crate hex;
///
/// use trie::{Trie, build_trie};
/// use hex::FromHex;
///
/// fn main() {
/// 	let v = vec![
/// 		("doe", "reindeer"),
/// 		("dog", "puppy"),
/// 		("dogglesworth", "cat"),
/// 	];
///
/// 	let root:Vec<u8> = array_bytes::hex2bytes("8aad789dff2f538bca5d8ea56e8abe10f4c7ba3a5dea95fea4cd6e7c3a1168d3").unwrap();
/// 	assert_eq!(build_trie(v).unwrap().root().unwrap(), root);
/// }
/// ```
pub fn build_trie<I, A, B>(data: I) -> TrieResult<MerklePatriciaTrie>
where
	I: IntoIterator<Item = (A, B)>,
	A: AsRef<[u8]> + Ord,
	B: AsRef<[u8]>,
{
	let memdb = Rc::new(MemoryDB::new());
	let mut trie = MerklePatriciaTrie::new(memdb.clone());
	for (k, v) in data {
		trie.insert(k.as_ref().to_vec(), v.as_ref().to_vec())?;
	}
	trie.root()?;
	Ok(trie)
}

/// Generates a trie for a vector of values
///
/// ```rust
/// extern crate merkle_patricia_trie as trie;
/// extern crate hex;
///
/// use trie::{Trie, build_order_trie};
/// use hex::FromHex;
///
/// fn main() {
/// 	let v = &["doe", "reindeer"];
/// 	let root:Vec<u8> = array_bytes::hex2bytes("e766d5d51b89dc39d981b41bda63248d7abce4f0225eefd023792a540bcffee3").unwrap();
/// 	assert_eq!(build_order_trie(v).unwrap().root().unwrap(), root);
/// }
/// ```
pub fn build_order_trie<I>(data: I) -> TrieResult<MerklePatriciaTrie>
where
	I: IntoIterator,
	I::Item: AsRef<[u8]>,
{
	build_trie(
		data.into_iter()
			.enumerate()
			.map(|(i, v)| (rlp::encode(&i), v)),
	)
}
