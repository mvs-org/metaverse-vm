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

use codec::{Decode, Encode};

use crate::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode)]
/// Error indicating value found is outside of a valid range.
pub struct OutOfBounds<T> {
	/// Minimum allowed value.
	pub min: Option<T>,
	/// Maximum allowed value.
	pub max: Option<T>,
	/// Value found.
	pub found: T,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Encode, Decode)]
/// Error indicating an expected value was not found.
pub struct Mismatch<T> {
	/// Value expected.
	pub expected: T,
	/// Value found.
	pub found: T,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum EthereumError {
	InvalidProofOfWork(OutOfBounds<U256>),
	DifficultyOutOfBounds(OutOfBounds<U256>),
	InvalidSealArity(Mismatch<usize>),
	SealInvalid,
	MerkleProofMismatch(&'static str),
	Rlp(&'static str),
	InvalidReceiptProof,
}

impl From<EthereumError> for &str {
	fn from(e: EthereumError) -> Self {
		use EthereumError::*;

		match e {
			InvalidProofOfWork(_) => "Proof Of Work - INVALID",
			DifficultyOutOfBounds(_) => "Difficulty - OUT OF BOUNDS",
			InvalidSealArity(_) => "Seal Arity - INVALID",
			SealInvalid => "Seal - INVALID",
			MerkleProofMismatch(msg) => msg,
			Rlp(msg) => msg,
			InvalidReceiptProof => "EthereumReceipt Proof - INVALID",
		}
	}
}
