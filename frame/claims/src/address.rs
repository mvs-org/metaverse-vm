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

// --- crates ---
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// --- hyperspace ---
use crate::AddressT;


macro_rules! impl_address {
	($name:ident, $sname:expr, $prefix:expr) => {
		#[doc = "An "]
		#[doc = $sname]
		#[doc = " address (i.e. 20 bytes, used to represent an "]
		#[doc = $sname]
		#[doc = ".\n\nThis gets serialized to the "]
		#[doc = $prefix]
		#[doc = "-prefixed hex representation."]
		#[derive(Debug, Default)]
		pub struct $name(pub AddressT);
		impl Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: Serializer,
			{
				serializer.serialize_str(&array_bytes::bytes2hex($prefix, &self.0))
			}
		}
		impl<'de> Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: Deserializer<'de>,
			{
				let base_string = String::deserialize(deserializer)?;
				let offset = if base_string.starts_with($prefix) {
					2
				} else {
					0
				};
				let s = &base_string[offset..];
				if s.len() != 40 {
					Err(serde::de::Error::custom(concat!(
						"Bad length of ",
						$sname,
						" address (should be 42 including '",
						$prefix,
						"')"
					)))?;
				}

				Ok($name(array_bytes::hex2array_unchecked!(s, 20)))
			}
		}
	};
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Account<Address> {
	pub address: Address,
	pub backed_etp: u128,
}

hyperspace_support::impl_genesis! {
	struct ClaimsList {
		dot: Vec<Account<EthereumAddress>>,
		eth: Vec<Account<EthereumAddress>>,
		oldetp: Vec<Account<OldetpAddress>>
	}
}

impl_address!(EthereumAddress, "Ethereum", "0x");
impl_address!(OldetpAddress, "Oldetp", "41");
