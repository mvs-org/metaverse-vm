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

//! Mock file for ethereum-linear-relay.

// --- std ---
use std::fs::File;
// --- crates ---
use serde::Deserialize;
// --- substrate ---
use frame_support::{impl_outer_dispatch, impl_outer_origin, parameter_types};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, RuntimeDebug};
// --- hyperspace ---
use crate::*;
use ethereum_primitives::receipt::LogEntry;
use ethereum_types::H512;

type AccountId = u64;
type BlockNumber = u64;
type Balance = u128;

pub type System = frame_system::Module<Test>;
pub type EthereumRelay = Module<Test>;

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
		frame_system::System,
		hyperspace_ethereum_relay::EthereumRelay,
	}
}

hyperspace_support::impl_test_account_data! { deprecated }

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;
parameter_types! {
	pub const EthereumRelayModuleId: ModuleId = ModuleId(*b"da/ethli");
	pub static EthereumNetwork: EthereumNetworkType = EthereumNetworkType::Ropsten;
}
impl Config for Test {
	type ModuleId = EthereumRelayModuleId;
	type Event = ();
	type EthereumNetwork = EthereumNetwork;
	type Call = Call;
	type Currency = Etp;
	type WeightInfo = ();
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = ();
	type Version = ();
	type PalletInfo = ();
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl hyperspace_balances::Config<EtpInstance> for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ();
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type MaxLocks = ();
	type OtherCurrencies = ();
	type WeightInfo = ();
}

#[derive(Debug)]
pub struct BlockWithProof {
	pub proof_length: u64,
	pub merkle_root: H128,
	pub header_rlp: Vec<u8>,
	pub merkle_proofs: Vec<H128>,
	pub elements: Vec<H256>,
}
impl BlockWithProof {
	pub fn from_file(path: &str) -> Self {
		#[derive(Deserialize)]
		struct RawBlockWithProof {
			proof_length: u64,
			merkle_root: String,
			header_rlp: String,
			merkle_proofs: Vec<String>,
			elements: Vec<String>,
		}

		fn zero_padding(mut s: String, hex_len: usize) -> String {
			let len = hex_len << 1;
			if s.starts_with("0x") {
				let missing_zeros = len + 2 - s.len();
				if missing_zeros != 0 {
					for _ in 0..missing_zeros {
						s.insert(2, '0');
					}
				}
			} else {
				let missing_zeros = len - s.len();
				if missing_zeros != 0 {
					for _ in 0..missing_zeros {
						s.insert(0, '0');
					}
				}
			}

			s
		}

		let raw_block_with_proof: RawBlockWithProof =
			serde_json::from_reader(File::open(path).unwrap()).unwrap();

		BlockWithProof {
			proof_length: raw_block_with_proof.proof_length,
			merkle_root: array_bytes::hex2array_unchecked!(&raw_block_with_proof.merkle_root, 16).into(),
			header_rlp: array_bytes::hex2bytes_unchecked(&raw_block_with_proof.header_rlp),
			merkle_proofs: raw_block_with_proof
				.merkle_proofs
				.iter()
				.cloned()
				.map(|raw_merkle_proof| {
					array_bytes::hex2array_unchecked!(&zero_padding(raw_merkle_proof, 16), 16).into()
				})
				.collect(),
			elements: raw_block_with_proof
				.elements
				.iter()
				.cloned()
				.map(|raw_element| {
					array_bytes::hex2array_unchecked!(&zero_padding(raw_element, 32), 32).into()
				})
				.collect(),
		}
	}

	pub fn to_double_node_with_merkle_proof_vec(&self) -> Vec<EthashProof> {
		fn combine_dag_h256_to_h512(elements: Vec<H256>) -> Vec<H512> {
			elements
				.iter()
				.zip(elements.iter().skip(1))
				.enumerate()
				.filter(|(i, _)| i % 2 == 0)
				.map(|(_, (a, b))| {
					let mut buffer = [0u8; 64];
					buffer[..32].copy_from_slice(&(a.0));
					buffer[32..].copy_from_slice(&(b.0));
					H512(buffer.into())
				})
				.collect()
		}

		let h512s = combine_dag_h256_to_h512(self.elements.clone());
		h512s
			.iter()
			.zip(h512s.iter().skip(1))
			.enumerate()
			.filter(|(i, _)| i % 2 == 0)
			.map(|(i, (a, b))| EthashProof {
				dag_nodes: [*a, *b],
				proof: self.merkle_proofs
					[i / 2 * self.proof_length as usize..(i / 2 + 1) * self.proof_length as usize]
					.to_vec(),
			})
			.collect()
	}
}

pub struct HeaderWithProof {
	pub header: EthereumHeader,
	pub proof: Vec<EthashProof>,
}
impl HeaderWithProof {
	fn from_file(path: &str) -> Self {
		#[derive(Deserialize)]
		struct RawShadowServiceResponse {
			result: RawHeaderWithProof,
		}
		#[derive(Deserialize)]
		struct RawHeaderWithProof {
			eth_header: String,
			proof: String,
		}

		let raw_shadow_service_response: RawShadowServiceResponse =
			serde_json::from_reader(File::open(path).unwrap()).unwrap();
		Self {
			header: Decode::decode::<&[u8]>(
				&mut &array_bytes::hex2bytes_unchecked(raw_shadow_service_response.result.eth_header)[..],
			)
			.unwrap(),
			proof: Decode::decode::<&[u8]>(
				&mut &array_bytes::hex2bytes_unchecked(raw_shadow_service_response.result.proof)[..],
			)
			.unwrap(),
		}
	}
}

pub struct ExtBuilder {
	eth_network: EthereumNetworkType,
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			eth_network: EthereumNetworkType::Ropsten,
		}
	}
}
impl ExtBuilder {
	pub fn eth_network(mut self, eth_network: EthereumNetworkType) -> Self {
		self.eth_network = eth_network;
		self
	}
	pub fn set_associated_constants(&self) {
		ETHEREUM_NETWORK.with(|v| v.replace(self.eth_network.clone()));
	}

	pub fn build(self) -> sp_io::TestExternalities {
		self.set_associated_constants();

		let mut storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		GenesisConfig::<Test> {
			number_of_blocks_finality: 30,
			number_of_blocks_safe: 10,
			dags_merkle_roots_loader: DagsMerkleRootsLoader::from_file(
				"../../../../bin/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		storage.into()
	}
}

/// To help reward miners for when duplicate block solutions are found
/// because of the shorter block times of Ethereum (compared to other crypto currency).
/// An uncle is a smaller reward than a full block.
///
/// stackoverflow: https://ethereum.stackexchange.com/questions/34/what-is-an-uncle-ommer-block
///
/// returns: [origin, grandpa, uncle, parent, current]
pub fn mock_canonical_relationship() -> [HeaderWithProof; 5] {
	// The block we loads
	// | pos     | height  | tx                                                                 |
	// |---------|---------|--------------------------------------------------------------------|
	// | origin  | 7575765 |                                                                    |
	// | grandpa | 7575766 | 0xc56be493f656f1c8222006eda5cd3392be5f0c096e8b7fb1c5542088c0f0c889 |
	// | uncle   | 7575766 |                                                                    |
	// | parent  | 7575767 |                                                                    |
	// | current | 7575768 | 0xfc836bf547f1e035e837bf0a8d26e432aa26da9659db5bf6ba69b0341d818778 |
	[
		HeaderWithProof::from_file("./src/test-data/ropsten_origin_7575765_scale.json"),
		HeaderWithProof::from_file("./src/test-data/ropsten_grandpa_7575766_scale.json"),
		HeaderWithProof::from_file("./src/test-data/ropsten_uncle_7575766_scale.json"),
		HeaderWithProof::from_file("./src/test-data/ropsten_parent_7575767_scale.json"),
		HeaderWithProof::from_file("./src/test-data/ropsten_current_7575768_scale.json"),
	]
}

/// mock canonical receipt
pub fn mock_canonical_receipt() -> EthereumReceiptProof {
	// fn mock_receipt_from_source(o: &mut Object) -> Option<EthereumReceiptProof> {
	// 	Some(EthereumReceiptProof {
	// 		index: o.get("index")?.as_str()?[2..].parse::<u64>().unwrap(),
	// 		proof: hex(&o.get("proof")?.as_str()?)?,
	// 		header_hash: H256::from(bytes!(&o.get("header_hash")?, 32)),
	// 	})
	// }

	let receipt: serde_json::Value = serde_json::from_str(RECEIPT).unwrap();
	EthereumReceiptProof {
		index: receipt["index"]
			.as_str()
			.unwrap()
			.trim_start_matches("0x")
			.parse()
			.unwrap(),
		proof: array_bytes::hex2bytes_unchecked(receipt["proof"].as_str().unwrap()),
		header_hash: array_bytes::hex2array_unchecked!(receipt["header_hash"].as_str().unwrap(), 32)
			.into(),
	}
}

/// mock log events
pub fn mock_receipt_logs() -> Vec<LogEntry> {
	let logs: serde_json::Value = serde_json::from_str(EVENT_LOGS).unwrap();
	logs["logs"]
		.as_array()
		.unwrap()
		.iter()
		.map(|log| LogEntry {
			address: array_bytes::hex2array_unchecked!(log["address"].as_str().unwrap(), 20).into(),
			topics: log["topics"]
				.as_array()
				.unwrap()
				.iter()
				.map(|topic| array_bytes::hex2array_unchecked!(topic.as_str().unwrap(), 32).into())
				.collect(),
			data: array_bytes::hex2bytes_unchecked(log["data"].as_str().unwrap()),
		})
		.collect()
}

// TODO: make this correct
pub const MAINNET_GENESIS_HEADER: &'static str = r#"
{
	"difficulty": "0x400000000",
	"extraData": "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa",
	"gasLimit": "0x1388",
	"gasUsed": "0x0",
	"hash": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3",
	"logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
	"miner": "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa",
	"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
	"nonce": "0x0000000000000042",
	"number": "0x0",
	"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
	"receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
	"sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
	"size": "0x21c",
	"stateRoot": "0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544",
	"timestamp": "0x0",
	"totalDifficulty": "0x400000000",
	"transactions": [omitted],
	"transactionsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
	"uncles": [omitted]
}
"#;

/// common receipt
pub const RECEIPT: &'static str = r#"{
	"index": "0x3",
	"proof": "0xf90639f90636b853f851a0e2dde80962d77a47a1eab063cc8a378f739d23df6e29593b9a213416656c68c180808080808080a02afdbac54a1d63d1329fd2ce2cac4041e26a23aee0509d76b23b0dbedf44a5f38080808080808080b8d3f8d180a068902a3cc3e2192a6ccc54eef093708d56e13136dd90b08efb0f1dc3305df2d7a085a0404ca58ca11c6e2d7dc0bdc7eacbfa284097940cd23f1a4200476c4ecd0fa0d569ad3746049c498094c5e3e1a28a498e5525732684eba723ff6539d3cac009a0385871245210d867025a2f2ea5143b884b67c3a2ba2f76561b982de230492012a086c057ea140bacf807b4c0d93efabf320ebe30a43d997259241832aea2ac26c6a02e10c5544b0b294153fafe7b444127463c08e9de37b60fc300070287d9b8d5b080808080808080808080b90509f9050620b90502f904ff018314ec7fb9010000000000000000000000002000000000400000000000201008000000000000000040000000000002000000000000000000000000000080400000080000000000000000000000000000000208100000000000000000000000000000000000000000000000020000000000000000000804080000000000000000000010000000000000000000000001000000000000000000000000004000000000000000200000000000000000000000000000000000000000000000000200000000000000000000000002000000000000000000040000000001000000000800200000000060000000000000000000000000000000000000000000000000020000000000000000f903f4f89b94b52fbe2b925ab79a821b261c82c5ba0814aaa5e0f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000002c7536e3605d9c16a7a3d7b1898e529396a65c23a0000000000000000000000000dbc888d701167cbfb86486c516aafbefc3a4de6ea00000000000000000000000000000000000000000000000000de0b6b3a7640000f87a94b52fbe2b925ab79a821b261c82c5ba0814aaa5e0f842a0cc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5a0000000000000000000000000dbc888d701167cbfb86486c516aafbefc3a4de6ea00000000000000000000000000000000000000000000000000de0b6b3a7640000f89b94b52fbe2b925ab79a821b261c82c5ba0814aaa5e0f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa0000000000000000000000000dbc888d701167cbfb86486c516aafbefc3a4de6ea00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000de0b6b3a7640000f9011c94dbc888d701167cbfb86486c516aafbefc3a4de6ef863a038045eaef0a21b74ff176350f18df02d9041a25d6694b5f63e9474b7b6cd6b94a0000000000000000000000000b52fbe2b925ab79a821b261c82c5ba0814aaa5e0a00000000000000000000000002c7536e3605d9c16a7a3d7b1898e529396a65c23b8a00000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000212ad7b504ddbe25a05647312daa8d0bbbafba360686241b7e193ca90f9b01f95faa00000000000000000000000000000000000000000000000000000000000000f9011c94b52fbe2b925ab79a821b261c82c5ba0814aaa5e0f863a09bfafdc2ae8835972d7b64ef3f8f307165ac22ceffde4a742c52da5487f45fd1a00000000000000000000000002c7536e3605d9c16a7a3d7b1898e529396a65c23a0000000000000000000000000dbc888d701167cbfb86486c516aafbefc3a4de6eb8a00000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000212ad7b504ddbe25a05647312daa8d0bbbafba360686241b7e193ca90f9b01f95faa00000000000000000000000000000000000000000000000000000000000000",
	"header_hash": "0xb49cc783d8da7896e5dc50fc2a927b80dcef6ebb36738a3f0aeaf3b4f970e768"
}"#;

/// event logs
pub const EVENT_LOGS: &'static str = r#"
{
	"logs": [
		{
			"address": "0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0",
			"blockHash": "0xb49cc783d8da7896e5dc50fc2a927b80dcef6ebb36738a3f0aeaf3b4f970e768",
			"blockNumber": 7575766,
			"data": "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000",
			"logIndex": 8,
			"removed": false,
			"topics": [
				"0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
				"0x0000000000000000000000002c7536e3605d9c16a7a3d7b1898e529396a65c23",
				"0x000000000000000000000000dbc888d701167cbfb86486c516aafbefc3a4de6e"
			],
			"transactionHash": "0xc56be493f656f1c8222006eda5cd3392be5f0c096e8b7fb1c5542088c0f0c889",
			"transactionIndex": 3,
			"id": "log_d6f43e1c"
		},
		{
			"address": "0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0",
			"blockHash": "0xb49cc783d8da7896e5dc50fc2a927b80dcef6ebb36738a3f0aeaf3b4f970e768",
			"blockNumber": 7575766,
			"data": "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000",
			"logIndex": 9,
			"removed": false,
			"topics": [
				"0xcc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5",
				"0x000000000000000000000000dbc888d701167cbfb86486c516aafbefc3a4de6e"
			],
			"transactionHash": "0xc56be493f656f1c8222006eda5cd3392be5f0c096e8b7fb1c5542088c0f0c889",
			"transactionIndex": 3,
			"id": "log_a2379338"
		},
		{
			"address": "0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0",
			"blockHash": "0xb49cc783d8da7896e5dc50fc2a927b80dcef6ebb36738a3f0aeaf3b4f970e768",
			"blockNumber": 7575766,
			"data": "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000",
			"logIndex": 10,
			"removed": false,
			"topics": [
				"0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
				"0x000000000000000000000000dbc888d701167cbfb86486c516aafbefc3a4de6e",
				"0x0000000000000000000000000000000000000000000000000000000000000000"
			],
			"transactionHash": "0xc56be493f656f1c8222006eda5cd3392be5f0c096e8b7fb1c5542088c0f0c889",
			"transactionIndex": 3,
			"id": "log_acf4e896"
		},
		{
			"address": "0xdBC888D701167Cbfb86486C516AafBeFC3A4de6e",
			"blockHash": "0xb49cc783d8da7896e5dc50fc2a927b80dcef6ebb36738a3f0aeaf3b4f970e768",
			"blockNumber": 7575766,
			"data": "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000212ad7b504ddbe25a05647312daa8d0bbbafba360686241b7e193ca90f9b01f95faa00000000000000000000000000000000000000000000000000000000000000",
			"logIndex": 11,
			"removed": false,
			"topics": [
				"0x38045eaef0a21b74ff176350f18df02d9041a25d6694b5f63e9474b7b6cd6b94",
				"0x000000000000000000000000b52fbe2b925ab79a821b261c82c5ba0814aaa5e0",
				"0x0000000000000000000000002c7536e3605d9c16a7a3d7b1898e529396a65c23"
			],
			"transactionHash": "0xc56be493f656f1c8222006eda5cd3392be5f0c096e8b7fb1c5542088c0f0c889",
			"transactionIndex": 3,
			"id": "log_44dceebb"
		},
		{
			"address": "0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0",
			"blockHash": "0xb49cc783d8da7896e5dc50fc2a927b80dcef6ebb36738a3f0aeaf3b4f970e768",
			"blockNumber": 7575766,
			"data": "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000212ad7b504ddbe25a05647312daa8d0bbbafba360686241b7e193ca90f9b01f95faa00000000000000000000000000000000000000000000000000000000000000",
			"logIndex": 12,
			"removed": false,
			"topics": [
				"0x9bfafdc2ae8835972d7b64ef3f8f307165ac22ceffde4a742c52da5487f45fd1",
				"0x0000000000000000000000002c7536e3605d9c16a7a3d7b1898e529396a65c23",
				"0x000000000000000000000000dbc888d701167cbfb86486c516aafbefc3a4de6e"
			],
			"transactionHash": "0xc56be493f656f1c8222006eda5cd3392be5f0c096e8b7fb1c5542088c0f0c889",
			"transactionIndex": 3,
			"id": "log_840077b9"
		}
	]
}
"#;
