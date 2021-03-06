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

use sp_std::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header(Vec<u8>);
impl Header {
	/// Create a new owning header view.
	/// Expects the data to be an RLP-encoded header -- any other case will likely lead to
	/// panics further down the line.
	pub fn new(encoded: Vec<u8>) -> Self {
		Header(encoded)
	}
}
