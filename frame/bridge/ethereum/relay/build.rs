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

// --- std ---
use std::{env, fs, io::Read, path::Path};

fn main() {
	let mut dags_merkle_roots_file =
		fs::File::open("../../../../bin/res/ethereum/dags-merkle-roots.json").unwrap();
	let mut dags_merkle_roots_str = String::new();
	dags_merkle_roots_file
		.read_to_string(&mut dags_merkle_roots_str)
		.unwrap();

	fs::write(
		&Path::new(&env::var_os("OUT_DIR").unwrap()).join("dags_merkle_roots.rs"),
		&format!(
			"pub const DAGS_MERKLE_ROOTS_STR: &'static str = r#\"{}\"#;",
			dags_merkle_roots_str,
		),
	)
	.unwrap();
}
