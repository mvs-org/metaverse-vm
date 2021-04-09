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

//! Tests for the module.

#![cfg(test)]

// --- substrate ---
use frame_support::traits::OnFinalize;
use sp_runtime::testing::{Digest, H256};
// --- hyperspace ---
use crate::{mock::*, *};

use merkle_mountain_range::{leaf_index_to_pos, Merge};

#[test]
fn first_header_mmr() {
	new_test_ext().execute_with(|| {
		let parent_hash: H256 = Default::default();
		initialize_block(1, parent_hash);

		System::note_finished_extrinsics();
		HeaderMMR::on_finalize(1);

		let header = System::finalize();
		assert_eq!(
			header.digest,
			Digest {
				logs: vec![header_mmr_log(parent_hash)]
			}
		);
	});
}

#[test]
fn test_insert_header() {
	new_test_ext().execute_with(|| {
		initialize_block(1, Default::default());

		HeaderMMR::on_finalize(1);

		let mut headers = vec![];

		let mut header = System::finalize();
		headers.push(header.clone());

		for i in 2..30 {
			initialize_block(i, header.hash());

			HeaderMMR::on_finalize(i);
			header = System::finalize();
			headers.push(header.clone());
		}

		let h1 = 11 as u64;
		let h2 = 19 as u64;

		let prove_elem = headers[h1 as usize - 1].hash();

		let pos = 19;
		assert_eq!(pos, leaf_index_to_pos(h1));
		assert_eq!(prove_elem, HeaderMMR::mmr_node_list(pos).unwrap());

		let parent_mmr_root = HeaderMMR::_find_parent_mmr_root(headers[h2 as usize - 1].clone())
			.expect("Header mmr get failed");

		let store = <ModuleMMRStore<Test>>::default();
		let mmr = MMR::<_, MMRMerge<Test>, _>::new(leaf_index_to_mmr_size(h2 - 1), store);

		assert_eq!(mmr.get_root().expect("Get Root Failed"), parent_mmr_root);

		let proof = mmr.gen_proof(vec![pos]).expect("gen proof");

		let result = proof
			.verify(parent_mmr_root, vec![(pos, prove_elem)])
			.expect("verify");
		assert!(result);
	});
}

#[test]
fn should_serialize_mmr_digest() {
	let digest = Digest {
		logs: vec![header_mmr_log(Default::default())],
	};

	assert_eq!(
		serde_json::to_string(&digest).unwrap(),
		// 0x90 is compact codec of the length 36, 0x4d4d5252 is prefix "MMRR"
		r#"{"logs":["0x00904d4d52520000000000000000000000000000000000000000000000000000000000000000"]}"#
	);
}

#[test]
fn non_system_mmr_digest_item_encoding() {
	let item = header_mmr_log(Default::default());
	let encoded = item.encode();
	assert_eq!(
		encoded,
		vec![
			0,    // type = DigestItemType::Other
			0x90, // vec length
			77, 77, 82, 82, // Prefix, *b"MMRR"
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, // mmr root
		]
	);

	let decoded: DigestItem<H256> = Decode::decode(&mut &encoded[..]).unwrap();
	assert_eq!(item, decoded);
}

#[test]
fn test_mmr_root() {
	let store = <ModuleMMRStore<Test>>::default();
	let mut mmr = <MMR<_, MMRMerge<Test>, _>>::new(0, store);
	(0..10).for_each(|i| {
		let cur = HEADERS_N_ROOTS[i];
		mmr.push(array_bytes::hex2array_unchecked!(cur.0, 32).into())
			.unwrap();
		assert_eq!(
			&format!("{:?}", mmr.get_root().expect("get root failed"))[2..],
			cur.1
		);
	});
}

#[test]
fn test_mmr_merge() {
	let res = MMRMerge::<Test>::merge(
		&array_bytes::hex2array_unchecked!(HEADERS_N_ROOTS[0].0, 32).into(),
		&array_bytes::hex2array_unchecked!(HEADERS_N_ROOTS[1].0, 32).into(),
	);
	assert_eq!(
		format!("{:?}", res),
		"0x3aafcc7fe12cb8fad62c261458f1c19dba0a3756647fa4e8bff6e248883938be"
	);
}
