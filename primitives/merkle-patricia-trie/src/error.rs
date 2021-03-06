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

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

use rlp::DecoderError;
#[cfg(not(feature = "std"))]
use sp_std::borrow::ToOwned;
use sp_std::fmt;

#[derive(Debug)]
pub enum TrieError {
	DB(String),
	Decoder(DecoderError),
	InvalidData,
	InvalidStateRoot,
	InvalidProof,
}

impl fmt::Display for TrieError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let printable = match *self {
			TrieError::DB(ref err) => format!("trie error: {:?}", err),
			TrieError::Decoder(ref err) => format!("trie error: {:?}", err),
			TrieError::InvalidData => "trie error: invalid data".to_owned(),
			TrieError::InvalidStateRoot => "trie error: invalid state root".to_owned(),
			TrieError::InvalidProof => "trie error: invalid proof".to_owned(),
		};
		write!(f, "{}", printable)
	}
}

impl From<DecoderError> for TrieError {
	fn from(error: DecoderError) -> Self {
		TrieError::Decoder(error)
	}
}
