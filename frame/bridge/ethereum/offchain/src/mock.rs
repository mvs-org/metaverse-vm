// This file is part of Hyperspace.
//
// Copyright (C) 2018-2021 Metaverse
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

// --- crates ---
use codec::Encode;
// --- substrate ---
use frame_support::{impl_outer_dispatch, impl_outer_origin, parameter_types, weights::Weight};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
	ModuleId, MultiSignature, Perbill, RuntimeDebug,
};
// --- hyperspace ---
use crate::*;
use hyperspace_ethereum_linear_relay::EthereumNetworkType;

type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
type Balance = u128;
type Extrinsic = TestXt<Call, ()>;

pub type System = frame_system::Module<Test>;
pub type Etp = hyperspace_balances::Module<Test, EtpInstance>;
pub type EthereumRelay = hyperspace_ethereum_linear_relay::Module<Test>;

pub type EthOffchain = Module<Test>;

impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
		hyperspace_ethereum_linear_relay::EthereumRelay,
		hyperspace_ethereum_offchain::EthOffchain,
	}
}

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

hyperspace_support::impl_test_account_data! {}

static mut SHADOW_SERVICE: Option<ShadowService> = None;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const FetchInterval: u64 = 3;
}
impl Trait for Test {
	type AuthorityId = crypto::AuthorityId;
	type FetchInterval = FetchInterval;
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = sp_core::H256;
	type Hashing = BlakeTwo256;
	type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl hyperspace_balances::Trait<EtpInstance> for Test {
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

parameter_types! {
	pub const EthereumRelayModuleId: ModuleId = ModuleId(*b"da/ethli");
	pub const EthereumNetwork: EthereumNetworkType = EthereumNetworkType::Ropsten;
}
impl hyperspace_ethereum_linear_relay::Trait for Test {
	type ModuleId = EthereumRelayModuleId;
	type Event = ();
	type EthereumNetwork = EthereumNetwork;
	type Call = Call;
	type Currency = Etp;
	type WeightInfo = ();
}

impl frame_system::offchain::SigningTypes for Test {
	type Public = <MultiSignature as Verify>::Signer;
	type Signature = MultiSignature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	type Extrinsic = Extrinsic;
	type OverarchingCall = Call;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		_public: <MultiSignature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

pub struct ExtBuilder {
	genesis_header: Option<(u64, Vec<u8>)>,
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			genesis_header: None,
		}
	}
}
impl ExtBuilder {
	pub fn set_genesis_header(mut self) -> Self {
		let genesis_header = EthereumHeader::from_str_unchecked(SUPPOSED_ETH_HEADER);
		self.genesis_header = Some((1, rlp::encode(&genesis_header)));
		self
	}
	pub fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		hyperspace_ethereum_linear_relay::GenesisConfig::<Test> {
			genesis_header: self.genesis_header,
			..Default::default()
		}
		.assimilate_storage(&mut storage)
		.unwrap();
		storage.into()
	}
}

impl OffchainRequestTrait for OffchainRequest {
	fn send(&mut self) -> Option<Vec<u8>> {
		let _ = self;
		unsafe {
			match SHADOW_SERVICE {
				Some(ShadowService::Scale) => Some(SUPPOSED_SHADOW_SCALE_RESPONSE.to_vec()),
				Some(ShadowService::Json) => Some(SUPPOSED_SHADOW_JSON_RESPONSE.to_vec()),
				_ => None,
			}
		}
	}
}

pub enum ShadowService {
	Scale,
	Json,
}

pub(crate) fn set_shadow_service(s: Option<ShadowService>) {
	unsafe {
		SHADOW_SERVICE = s;
	}
}

pub const SUPPOSED_SHADOW_SCALE_RESPONSE: &'static [u8] = br#"{"jsonrpc":"2.0","id":1,"result":{"eth_header":"0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa32442ba5500000000010000000000000005a56e2d52c817161883f50c441c3228cfe54d9f56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4211dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934764476574682f76312e302e302f6c696e75782f676f312e342e32d67e4d450343046425ae4271474353857ab860dbc0a1dde64b41b5cd3a532bf356e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4210000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008813000000000000000000000000000000000000000000000000000000000000000080ff030000000000000000000000000000000000000000000000000000000884a0969b900de27b6ac6a67742365dd65f55a0526c41fd18e1b16f1a1215c2e66f592488539bd4979fef1ec40188e96d4537bea4d9c05d12549907b32561d3bf31f45aae734cdc119f13406cb6","proof":"0x04000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}}"#;
pub const SUPPOSED_ETH_HEADER: &'static str = r#"
			{
				"difficulty": "0x3ff800000",
				"extraData": "0x476574682f76312e302e302f6c696e75782f676f312e342e32",
				"gasLimit": "0x1388",
				"gasUsed": "0x0",
				"hash": "0x88e96d4537bea4d9c05d12549907b32561d3bf31f45aae734cdc119f13406cb6",
				"logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
				"miner": "0x05a56e2d52c817161883f50c441c3228cfe54d9f",
				"mixHash": "0x969b900de27b6ac6a67742365dd65f55a0526c41fd18e1b16f1a1215c2e66f59",
				"nonce": "0x539bd4979fef1ec4",
				"number": "0x1",
				"parentHash": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3",
				"receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
				"sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
				"size": "0x219",
				"stateRoot": "0xd67e4d450343046425ae4271474353857ab860dbc0a1dde64b41b5cd3a532bf3",
				"timestamp": "0x55ba4224",
				"totalDifficulty": "0x7ff800000",
				"transactions": [],
				"transactionsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
				"uncles": []
			}
			"#;

pub const SUPPOSED_SHADOW_JSON_RESPONSE: &'static [u8] = br#"{"jsonrpc":"2.0","id":1,"result":{"eth_header":{"difficulty":"0x9fa52dbdada","extraData":"0xd783010302844765746887676f312e352e31856c696e7578","gasLimit":"0x2fefd8","gasUsed":"0x37881","hash":"0x26f10bfb3c09f1e1eadf856a8d75f5dbd2f88bd8eb4da8488f131835fa4a6ae3","logsBloom":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000c00000000000000000000020000000000000004000000000000000000000000000000020000000000000000000000000001000000000000004000000200000000000000000008020000020000000000000000001000000000000000000000004000040000000000000000000000000000000000000000000000000000000004001000000000000000000000000004080008000000000120000000000000000000000400000000000800000000000000000000000000200000000000001000000000000a0008000040000000000000000000000000000000","miner":"0x738db714c08b8a32a29e0e68af00215079aa9c5c","mixHash":"0xcb63ce95a3043c0f846ad6e1c3c25ec7a8cd8e09dccf02c7078669f2496f02c2","nonce":"0xfc2c4055195dac95","number":"0xeb770","parentHash":"0x28e9cc57847a0a1efd2920115ba94530ba7d29d7a7ffb15fc933302a97c73e49","receiptsRoot":"0xba124ff4744d7f59fd4f829be59f727fe17f468b34344759d4dd2ed10d6260d2","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","size":"0x792","stateRoot":"0x46f9f3d17b9bba9d551ab85a6aa6686a51590a184f5d42b98b6d8518303da470","timestamp":"0x56b66a81","totalDifficulty":"0x5d4fe4695aed3d42","transactions":[],"transactionsRoot":"0x5e7f4d048b09e832ccdb062c655def06f532ebdf02b3c0c423a65c6566220523","uncles":[]},"proof":[{"dag_nodes":["0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"],"proof":["0x00000000000000000000000000000000","0x00000000000000000000000000000000"]}]}}"#;
