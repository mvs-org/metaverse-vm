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

//! Tests for ethereum-linear-relay.

// --- substrate ---
use frame_support::{
	assert_err, assert_ok,
	weights::{DispatchClass, DispatchInfo},
};
use frame_system::RawOrigin;
// --- hyperspace ---
use crate::{mock::*, *};
use ethereum_primitives::receipt::TransactionOutcome;

// --- ropsten test ---

#[test]
fn verify_receipt_proof() {
	ExtBuilder::default().build().execute_with(|| {
		System::inc_account_nonce(&2);
		assert_ok!(EthereumRelay::set_number_of_blocks_safe(
			RawOrigin::Root.into(),
			0
		));

		// mock header and proof
		let [_, header_with_proof, _, _, _] = mock_canonical_relationship();
		let proof_record = mock_canonical_receipt();

		// mock logs
		let mut logs = vec![];
		let mut log_entries = mock_receipt_logs();
		for _ in 0..log_entries.len() {
			logs.push(log_entries.pop().unwrap());
		}

		logs.reverse();

		// mock receipt
		let receipt = EthereumReceipt::new(TransactionOutcome::StatusCode(1), 1371263.into(), logs);

		// verify receipt
		assert_ok!(EthereumRelay::init_genesis_header(
			&header_with_proof.header,
			0x6b2dd4a2c4f47d
		));
		assert_eq!(
			EthereumRelay::verify_receipt(&proof_record).unwrap(),
			receipt
		);
	});
}

#[test]
fn relay_header() {
	ExtBuilder::default().build().execute_with(|| {
		let [origin, grandpa, _, parent, current] = mock_canonical_relationship();
		assert_ok!(EthereumRelay::init_genesis_header(
			&origin.header,
			0x6b2dd4a2c4f47d
		));

		// relay grandpa
		assert_ok!(EthereumRelay::verify_header_basic(&grandpa.header));
		assert_ok!(EthereumRelay::maybe_store_header(&0, &grandpa.header));

		// relay parent
		assert_ok!(EthereumRelay::verify_header_basic(&parent.header));
		assert_ok!(EthereumRelay::maybe_store_header(&0, &parent.header));

		// relay current
		assert_ok!(EthereumRelay::verify_header_basic(&current.header));
		assert_ok!(EthereumRelay::maybe_store_header(&0, &current.header));
	});
}

/// # Check EthereumReceipt Safety
///
/// ## Family Tree
///
/// | pos     | height  | tx                                                                 |
/// |---------|---------|--------------------------------------------------------------------|
/// | origin  | 7575765 |                                                                    |
/// | grandpa | 7575766 | 0xc56be493f656f1c8222006eda5cd3392be5f0c096e8b7fb1c5542088c0f0c889 |
/// | uncle   | 7575766 |                                                                    |
/// | parent  | 7575767 |                                                                    |
/// | current | 7575768 | 0xfc836bf547f1e035e837bf0a8d26e432aa26da9659db5bf6ba69b0341d818778 |
///
/// To help reward miners for when duplicate block solutions are found
/// because of the shorter block times of Ethereum (compared to other cryptocurrency).
/// An uncle is a smaller reward than a full block.
///
/// ## Note:
///
/// check receipt should
/// - succeed when we confirmed the correct header
/// - failed when canonical hash was re-orged by the block which contains our tx's brother block
#[test]
fn check_receipt_safety() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EthereumRelay::add_authority(RawOrigin::Root.into(), 0));
		assert_ok!(EthereumRelay::set_number_of_blocks_safe(
			RawOrigin::Root.into(),
			0
		));

		// family tree
		let [origin, grandpa, uncle, _, _] = mock_canonical_relationship();
		assert_ok!(EthereumRelay::init_genesis_header(
			&origin.header,
			0x6b2dd4a2c4f47d
		));

		let receipt = mock_canonical_receipt();
		assert_ne!(grandpa.header.hash, uncle.header.hash);
		assert_eq!(grandpa.header.number, uncle.header.number);

		// check receipt should succeed when we confirmed the correct header
		assert_ok!(EthereumRelay::relay_header(
			Origin::signed(0),
			grandpa.header.clone(),
			grandpa.proof
		));
		assert_ok!(EthereumRelay::check_receipt(
			Origin::signed(0),
			receipt.clone(),
		));

		// check should fail when canonical hash was re-orged by
		// the block which contains our tx's brother block
		assert_ok!(EthereumRelay::relay_header(
			Origin::signed(0),
			uncle.header,
			uncle.proof
		));
		assert_err!(
			EthereumRelay::check_receipt(Origin::signed(0), receipt.clone()),
			<Error<Test>>::ReceiptProofInv
		);
	});
}

#[test]
fn canonical_reorg_uncle_should_succeed() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EthereumRelay::add_authority(RawOrigin::Root.into(), 0));
		assert_ok!(EthereumRelay::set_number_of_blocks_safe(
			RawOrigin::Root.into(),
			0
		));

		let [origin, grandpa, uncle, _, _] = mock_canonical_relationship();
		assert_ok!(EthereumRelay::init_genesis_header(
			&origin.header,
			0x6b2dd4a2c4f47d
		));

		// check relationship
		assert_ne!(grandpa.header.hash, uncle.header.hash);
		assert_eq!(grandpa.header.number, uncle.header.number);

		let (gh, uh) = (grandpa.header.hash, uncle.header.hash);
		let number = grandpa.header.number;

		// relay uncle header
		assert_ok!(EthereumRelay::relay_header(
			Origin::signed(0),
			uncle.header,
			uncle.proof
		));
		assert_eq!(EthereumRelay::canonical_header_hash(number), uh.unwrap());

		// relay grandpa and re-org uncle
		assert_ok!(EthereumRelay::relay_header(
			Origin::signed(0),
			grandpa.header,
			grandpa.proof
		));
		assert_eq!(EthereumRelay::canonical_header_hash(number), gh.unwrap());
	});
}

#[test]
fn test_safety_block() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EthereumRelay::add_authority(RawOrigin::Root.into(), 0));
		assert_ok!(EthereumRelay::set_number_of_blocks_safe(
			RawOrigin::Root.into(),
			2
		));

		// family tree
		let [origin, grandpa, uncle, parent, current] = mock_canonical_relationship();

		let receipt = mock_canonical_receipt();

		// not safety after 0 block
		assert_ok!(EthereumRelay::init_genesis_header(
			&origin.header,
			0x6b2dd4a2c4f47d
		));
		assert_ok!(EthereumRelay::relay_header(
			Origin::signed(0),
			grandpa.header,
			grandpa.proof
		));
		assert_err!(
			EthereumRelay::check_receipt(Origin::signed(0), receipt.clone()),
			<Error<Test>>::ReceiptProofInv
		);

		// not safety after 2 blocks
		assert_ok!(EthereumRelay::relay_header(
			Origin::signed(0),
			parent.header,
			parent.proof
		));
		assert_ok!(EthereumRelay::relay_header(
			Origin::signed(0),
			uncle.header,
			uncle.proof
		));
		assert_err!(
			EthereumRelay::check_receipt(Origin::signed(0), receipt.clone()),
			<Error<Test>>::ReceiptProofInv
		);

		// safety after 3 blocks
		assert_ok!(EthereumRelay::relay_header(
			Origin::signed(0),
			current.header,
			current.proof
		));
		assert_ok!(EthereumRelay::check_receipt(Origin::signed(0), receipt));
	});
}

// --- mainnet test ---

#[test]
fn build_genesis_header() {
	let genesis_header = EthereumHeader::from_str_unchecked(MAINNET_GENESIS_HEADER);
	assert_eq!(genesis_header.hash(), genesis_header.re_compute_hash());
	// println!("{:?}", rlp::encode(&genesis_header));
}

#[test]
fn relay_mainet_header() {
	ExtBuilder::default()
		.eth_network(EthereumNetworkType::Mainnet)
		.build()
		.execute_with(|| {
			assert_ok!(EthereumRelay::add_authority(RawOrigin::Root.into(), 0));
			assert_ok!(EthereumRelay::set_number_of_blocks_safe(
				RawOrigin::Root.into(),
				0
			));

			// block 8996776
			{
				let blocks_with_proof = BlockWithProof::from_file("./src/test-data/8996776.json");
				// println!("{:?}", blocks_with_proof);
				let header: EthereumHeader =
					rlp::decode(&blocks_with_proof.header_rlp.to_vec()).unwrap();
				assert_ok!(EthereumRelay::init_genesis_header(
					&header,
					0x6b2dd4a2c4f47d
				));
				// println!("{:?}", &header);
			}

			// block 8996777
			{
				let blocks_with_proof = BlockWithProof::from_file("./src/test-data/8996777.json");
				// println!("{:#?}", blocks_with_proof);
				let header: EthereumHeader =
					rlp::decode(&blocks_with_proof.header_rlp.to_vec()).unwrap();
				// println!("{:?}", &header);

				assert_ok!(EthereumRelay::verify_header_pow(
					&header,
					&blocks_with_proof.to_double_node_with_merkle_proof_vec()
				));
				assert_ok!(EthereumRelay::maybe_store_header(&0, &header));
			}

			// block 8996778
			{
				let blocks_with_proof = BlockWithProof::from_file("./src/test-data/8996778.json");
				// println!("{:?}", blocks_with_proof);
				let header: EthereumHeader =
					rlp::decode(&blocks_with_proof.header_rlp.to_vec()).unwrap();
				// println!("{:?}", &header);

				assert_ok!(EthereumRelay::verify_header_pow(
					&header,
					&blocks_with_proof.to_double_node_with_merkle_proof_vec()
				));
				assert_ok!(EthereumRelay::maybe_store_header(&0, &header));
			}
		});
}

#[test]
fn receipt_verify_fees_and_relayer_claim_reward() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EthereumRelay::add_authority(RawOrigin::Root.into(), 0));
		assert_ok!(EthereumRelay::set_number_of_blocks_safe(
			RawOrigin::Root.into(),
			0
		));

		assert_ok!(EthereumRelay::set_number_of_blocks_finality(
			RawOrigin::Root.into(),
			0
		));

		assert_ok!(EthereumRelay::set_receipt_verify_fee(
			RawOrigin::Root.into(),
			0
		));

		// family tree
		let [origin, grandpa, _, parent, _] = mock_canonical_relationship();

		let receipt = mock_canonical_receipt();

		// not safety after 0 block
		assert_ok!(EthereumRelay::init_genesis_header(
			&origin.header,
			0x6b2dd4a2c4f47d
		));
		assert_ok!(EthereumRelay::relay_header(
			Origin::signed(0),
			grandpa.header,
			grandpa.proof
		));

		// not safety after 2 blocks
		assert_ok!(EthereumRelay::relay_header(
			Origin::signed(0),
			parent.header,
			parent.proof
		));

		assert_ok!(EthereumRelay::check_receipt(
			Origin::signed(1),
			receipt.clone()
		));

		assert_ok!(EthereumRelay::set_receipt_verify_fee(
			RawOrigin::Root.into(),
			10
		));

		assert_err!(
			EthereumRelay::check_receipt(Origin::signed(1), receipt.clone()),
			EtpError::InsufficientBalance,
		);

		let _ = Etp::deposit_creating(&1, 1000);

		assert_ok!(EthereumRelay::check_receipt(
			Origin::signed(1),
			receipt.clone()
		));

		assert_eq!(EthereumRelay::pot(), 10);
		assert_eq!(Etp::free_balance(&1), 990);

		assert_ok!(EthereumRelay::claim_reward(Origin::signed(0)));

		assert_eq!(EthereumRelay::pot(), 0);
		assert_eq!(Etp::free_balance(&0), 10);
	});
}

#[test]
fn check_eth_relay_header_hash_works() {
	ExtBuilder::default()
		.eth_network(EthereumNetworkType::Mainnet)
		.build()
		.execute_with(|| {
			// block 8996776
			{
				let blocks_with_proof = BlockWithProof::from_file("./src/test-data/8996776.json");
				// println!("{:?}", blocks_with_proof);
				let header: EthereumHeader =
					rlp::decode(&blocks_with_proof.header_rlp.to_vec()).unwrap();
				assert_ok!(EthereumRelay::init_genesis_header(
					&header,
					0x6b2dd4a2c4f47d
				));

				let blocks_with_proof = BlockWithProof::from_file("./src/test-data/8996776.json");
				let header: EthereumHeader =
					rlp::decode(&blocks_with_proof.header_rlp.to_vec()).unwrap();

				let info = DispatchInfo {
					weight: 100,
					class: DispatchClass::Normal,
					..Default::default()
				};
				let check = CheckEthereumRelayHeaderParcel::<Test>(Default::default());
				let call: mock::Call = crate::Call::relay_header(
					header,
					blocks_with_proof.to_double_node_with_merkle_proof_vec(),
				)
				.into();

				assert_eq!(
					check.validate(&0, &call, &info, 0),
					InvalidTransaction::Custom(<Error<Test>>::HeaderAE.as_u8()).into(),
				);
			}

			// block 8996777
			{
				let blocks_with_proof = BlockWithProof::from_file("./src/test-data/8996777.json");
				let header: EthereumHeader =
					rlp::decode(&blocks_with_proof.header_rlp.to_vec()).unwrap();

				let info = DispatchInfo {
					weight: 100,
					class: DispatchClass::Normal,
					..Default::default()
				};
				let check = CheckEthereumRelayHeaderParcel::<Test>(Default::default());
				let call: mock::Call = crate::Call::relay_header(
					header,
					blocks_with_proof.to_double_node_with_merkle_proof_vec(),
				)
				.into();

				assert_eq!(check.validate(&0, &call, &info, 0), Ok(Default::default()));
			}
		});
}

#[test]
fn test_scale_coding_of_default_double_node_with_proof() {
	let default_double_node_with_proof = EthashProof::default();
	let mut scale_encode_str: &[u8] = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"; // len 129
	let decoded_double_node_with_proof: EthashProof =
		Decode::decode::<&[u8]>(&mut scale_encode_str).unwrap();
	assert_eq!(
		default_double_node_with_proof,
		decoded_double_node_with_proof
	);
}
#[test]
fn test_scale_coding_of_double_node_with_two_proof() {
	let mut scale_encode_str: &[u8] = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"; // len 129 + 16 + 16
	let decoded_double_node_with_proof: EthashProof =
		Decode::decode::<&[u8]>(&mut scale_encode_str).unwrap();
	assert_eq!(2, decoded_double_node_with_proof.proof.len());
}
#[test]
fn test_scale_coding_of_default_double_node_with_proof_vector() {
	let default_double_node_with_proof = EthashProof::default();
	let vector = vec![default_double_node_with_proof];
	let mut scale_encode_str: &[u8] = b"\x04\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"; // len 1 + 129
	let decoded_double_node_with_proof: Vec<EthashProof> =
		Decode::decode::<&[u8]>(&mut scale_encode_str).ok().unwrap();
	assert_eq!(vector, decoded_double_node_with_proof);
}

#[test]
fn test_build_double_node_with_proof_from_str() {
	let s = r#"{"dag_nodes":["0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000""],"proof":["0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000","0x00000000000000000000000000000000"]}"#;

	let double_node_with_merkle_proof = EthashProof::from_str_unchecked(s);
	assert_eq!(
		double_node_with_merkle_proof.dag_nodes,
		EthashProof::default().dag_nodes
	);
	assert_eq!(double_node_with_merkle_proof.proof.len(), 25);
}
