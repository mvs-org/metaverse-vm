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

// --- substrate ---
use frame_support::assert_ok;
// --- hyperspace ---
use crate::{mock::*, test_data::*, *};

#[test]
fn store_relay_header_parcel_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let ethereum_relay_header_parcel: EthereumRelayHeaderParcel = serde_json::from_str(r#"{"header":{"parent_hash":"0x3dd4dc843801af12c0a6dd687642467a3ce835dca09159734dec03109a1c1f1f","timestamp":1479653850,"number":100,"author":"0xc2fa6dcef5a1fbf70028c5636e7f64cd46e7cfd4","transactions_root":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","uncles_hash":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","extra_data":"0xd783010502846765746887676f312e362e33856c696e7578","state_root":"0xf5f18c33ddff06efa928d22a2432fb34a11e6f62cce825cdad1c78e1068e6b7b","receipts_root":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","log_bloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","gas_used":0,"gas_limit":15217318,"difficulty":827755,"seal":["0xa03172866e675b057a294d3f474e9141b588d5a0c622b4d8049e272c6a001e9c4e","0x886d88b33209e0a320"],"hash":"0xb40a0dfde1b270d7c58c3cb505c7e773c50198b28cce3e442c4e2f33ff764582"},"mmr_root":"0x33d834e1e65b96f470374134cf173f359a5b37c910a7e07c7d6148866c1805d7"}"#).unwrap();

		assert!(EthereumRelay::confirmed_header_parcel_of(100).is_none());
		assert!(!EthereumRelay::confirmed_block_numbers().contains(&100));
		assert!(EthereumRelay::best_confirmed_block_number() != 100);

		assert_eq!(ethereum_relay_header_parcel.header.number, 100);
		EthereumRelay::confirm_relay_header_parcel_with_reason(
			ethereum_relay_header_parcel.clone(),
			vec![]
		);

		assert_eq!(EthereumRelay::confirmed_header_parcel_of(100).unwrap(), ethereum_relay_header_parcel);
		assert!(EthereumRelay::confirmed_block_numbers().contains(&100));
		assert_eq!(EthereumRelay::best_confirmed_block_number(), 100);
	});
}

#[test]
fn verify_relay_proofs_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let ethereum_relay_header_parcel_100: EthereumRelayHeaderParcel =
			serde_json::from_str(LAST_CONFIRM).unwrap();

		EthereumRelay::confirm_relay_header_parcel_with_reason(
			ethereum_relay_header_parcel_100.clone(),
			vec![],
		);

		let ethereum_relay_header_parcel_103: EthereumRelayHeaderParcel =
			serde_json::from_str(HEADER_103).unwrap();
		let ethereum_relay_proofs_103: EthereumRelayProofs =
			serde_json::from_str(PROOFS_103).unwrap();

		assert_ok!(EthereumRelay::verify_relay_proofs(
			&103,
			&ethereum_relay_header_parcel_103,
			&ethereum_relay_proofs_103,
			Some(&100)
		));

		let ethereum_relay_header_parcel_102: EthereumRelayHeaderParcel =
			serde_json::from_str(HEADER_102).unwrap();
		let ethereum_relay_proofs_102: EthereumRelayProofs =
			serde_json::from_str(PROOFS_102).unwrap();

		assert_ok!(EthereumRelay::verify_relay_proofs(
			&103,
			&ethereum_relay_header_parcel_102,
			&ethereum_relay_proofs_102,
			None
		));

		let ethereum_relay_header_parcel_101: EthereumRelayHeaderParcel =
			serde_json::from_str(HEADER_101).unwrap();
		let ethereum_relay_proofs_101: EthereumRelayProofs =
			serde_json::from_str(PROOFS_101).unwrap();

		assert_ok!(EthereumRelay::verify_relay_proofs(
			&103,
			&ethereum_relay_header_parcel_101,
			&ethereum_relay_proofs_101,
			None
		));
	});
}

#[test]
fn verify_relay_chain_should_work() {
	ExtBuilder::default()
		.best_confirmed_block_number(100)
		.build()
		.execute_with(|| {
			EthereumRelay::confirm_relay_header_parcel_with_reason(
				serde_json::from_str(LAST_CONFIRM).unwrap(),
				vec![],
			);

			// Should work for random sample points order

			let relay_chain = vec![
				serde_json::from_str(HEADER_101).unwrap(),
				serde_json::from_str(HEADER_102).unwrap(),
				serde_json::from_str(HEADER_103).unwrap(),
			];

			assert_ok!(EthereumRelay::verify_relay_chain(
				relay_chain.iter().collect()
			));

			let relay_chain = vec![
				serde_json::from_str(HEADER_101).unwrap(),
				serde_json::from_str(HEADER_103).unwrap(),
				serde_json::from_str(HEADER_102).unwrap(),
			];

			assert_ok!(EthereumRelay::verify_relay_chain(
				relay_chain.iter().collect()
			));

			let relay_chain = vec![
				serde_json::from_str(HEADER_102).unwrap(),
				serde_json::from_str(HEADER_103).unwrap(),
				serde_json::from_str(HEADER_101).unwrap(),
			];

			assert_ok!(EthereumRelay::verify_relay_chain(
				relay_chain.iter().collect()
			));
		});
}

#[test]
fn try_confirm_relay_header_parcel_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(EthereumRelay::pending_relay_header_parcels().is_empty());
		assert_eq!(EthereumRelay::best_confirmed_block_number(), 0);

		assert_ok!(EthereumRelay::try_confirm_relay_header_parcel(
			serde_json::from_str(LAST_CONFIRM).unwrap()
		));

		assert!(EthereumRelay::pending_relay_header_parcels().is_empty());
		assert_eq!(EthereumRelay::best_confirmed_block_number(), 100);
	});

	ExtBuilder::default()
		.confirm_period(3)
		.build()
		.execute_with(|| {
			assert!(EthereumRelay::pending_relay_header_parcels().is_empty());
			assert_eq!(EthereumRelay::best_confirmed_block_number(), 0);

			assert_ok!(EthereumRelay::try_confirm_relay_header_parcel(
				serde_json::from_str(LAST_CONFIRM).unwrap()
			));

			assert!(!EthereumRelay::pending_relay_header_parcels().is_empty());
			assert_eq!(EthereumRelay::best_confirmed_block_number(), 0);

			run_to_block(3);

			assert!(EthereumRelay::pending_relay_header_parcels().is_empty());
			assert_eq!(EthereumRelay::best_confirmed_block_number(), 100);
		});
}

// #[test]
// fn mmr() {
// 	// 102 header hash
// 	let header_hash: H256 = array_bytes::array_bytes::hex2array_unchecked!(
// 		"0x16110f3aa1895de2ec22cfd746751f724d112a953c71b62858a1523b50f3dc64",
// 		32
// 	)
// 	.into();
// 	// 103 mmr root
// 	let mmr_root: H256 = array_bytes::array_bytes::hex2array_unchecked!(
// 		"0x34a80a8e0b6bfe253d1c960647cb4de34607a9caf86e99f7611304dbdf7fbde0",
// 		32
// 	)
// 	.into();
// 	// 102 mmr proof
// 	let mmr_proof: Vec<H256> = serde_json::from_str(r#"["0xd7ab806f1ea871d7c7ff0f2bd5c5fdc4a7f19fab776110b755b8c937ead62e5e","0xaa3b24b678c1146b1eea83cbca1b0f13058c19776e3769e36d0ff381502cebab","0xaa466c636dd7eac4230baefd943612ae0fc0a57aa47757a7f1f68bd246ee6119","0x0f2fd65d1be0509c89ef54749b2897243c283c584e29cfb51b9cbec9f086f600"]"#).unwrap();

// 	assert!(EthereumRelay::verify_mmr(
// 		102,
// 		mmr_root,
// 		mmr_proof,
// 		vec![(102, array_unchecked!(header_hash, 0, 32).into())]
// 	));
// }
