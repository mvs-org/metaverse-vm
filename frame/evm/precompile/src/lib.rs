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
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Hyperspace. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

pub type HyperspacePrecompiles<Runtime> = (
	hyperspace_evm_precompile_simple::ECRecover, // 0x0000000000000000000000000000000000000001
	hyperspace_evm_precompile_simple::Sha256,    // 0x0000000000000000000000000000000000000002
	hyperspace_evm_precompile_simple::Ripemd160, // 0x0000000000000000000000000000000000000003
	hyperspace_evm_precompile_simple::Identity,  // 0x0000000000000000000000000000000000000004
	hyperspace_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000005
	hyperspace_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000006
	hyperspace_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000007
	hyperspace_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000008
	hyperspace_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000009
	hyperspace_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000a
	hyperspace_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000b
	hyperspace_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000c
	hyperspace_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000d
	hyperspace_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000e
	hyperspace_evm_precompile_empty::Empty,      // 0x000000000000000000000000000000000000000f
	hyperspace_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000010
	hyperspace_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000011
	hyperspace_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000012
	hyperspace_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000013
	hyperspace_evm_precompile_empty::Empty,      // 0x0000000000000000000000000000000000000014
	hyperspace_evm_precompile_withdraw::WithDraw<Runtime>, // 0x0000000000000000000000000000000000000015
	hyperspace_evm_precompile_dna::Dna<Runtime>, // 0x0000000000000000000000000000000000000016
);
