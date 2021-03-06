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

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
pub extern crate rlp_derive;

pub mod encoded;
pub mod error;
pub mod ethashproof;
pub mod header;
pub mod pow;
pub mod receipt;

pub use ethbloom::{Bloom, Input as BloomInput};
pub use ethereum_types::H64;
pub use primitive_types::{H160, H256, U128, U256, U512};

use codec::{Decode, Encode};
use sp_std::prelude::*;

pub type Bytes = Vec<u8>;
pub type EthereumAddress = H160;
pub type EthereumBlockNumber = u64;

#[derive(Clone, PartialEq, Encode, Decode)]
pub enum EthereumNetworkType {
	Mainnet,
	Ropsten,
}
impl Default for EthereumNetworkType {
	fn default() -> EthereumNetworkType {
		EthereumNetworkType::Mainnet
	}
}
