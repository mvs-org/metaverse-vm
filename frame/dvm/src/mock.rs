// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Frontier.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.	 See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate. If not, see <http://www.gnu.org/licenses/>.

//! Test utilities

use crate::{
	self as dvm_ethereum,
	account_basic::{DvmAccountBasic, DnaRemainBalance, EtpRemainBalance},
	*,
};
use codec::{Decode, Encode};
use hyperspace_evm::{AddressMapping, EnsureAddressTruncated, FeeCalculator, IssuingHandler};
use ethereum::{TransactionAction, TransactionSignature};
use frame_support::{traits::GenesisBuild, ConsensusEngineId};
use frame_system::mocking::*;
use rlp::*;
use sp_core::{H160, H256, U256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, DispatchResult, ModuleId, Perbill, RuntimeDebug,
};

hyperspace_support::impl_test_account_data! {}

type Block = MockBlock<Test>;
type UncheckedExtrinsic = MockUncheckedExtrinsic<Test>;

type Balance = u64;

frame_support::parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

frame_support::parameter_types! {
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 10;
	pub const ExistentialDeposit: u64 = 500;
}

impl hyperspace_balances::Config<EtpInstance> for Test {
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type OtherCurrencies = ();
	type WeightInfo = ();
	type Balance = Balance;
	type Event = ();
	type BalanceInfo = AccountData<Balance>;
}

impl hyperspace_balances::Config<DnaInstance> for Test {
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type OtherCurrencies = ();
	type WeightInfo = ();
	type Balance = Balance;
	type Event = ();
	type BalanceInfo = AccountData<Balance>;
}

frame_support::parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> U256 {
		1.into()
	}
}

/// EmptyIssuingHandler
pub struct EmptyIssuingHandler;
impl IssuingHandler for EmptyIssuingHandler {
	fn handle(_address: H160, _caller: H160, _input: &[u8]) -> DispatchResult {
		Ok(())
	}
}

pub struct EthereumFindAuthor;
impl FindAuthor<H160> for EthereumFindAuthor {
	fn find_author<'a, I>(_digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		Some(address_build(0).address)
	}
}

frame_support::parameter_types! {
	pub const TransactionByteFee: u64 = 1;
	pub const ChainId: u64 = 23;
	pub const EVMModuleId: ModuleId = ModuleId(*b"py/evmpa");
	pub const BlockGasLimit: U256 = U256::MAX;
}

pub struct HashedAddressMapping;

impl AddressMapping<AccountId32> for HashedAddressMapping {
	fn into_account_id(address: H160) -> AccountId32 {
		let mut data = [0u8; 32];
		data[0..20].copy_from_slice(&address[..]);
		AccountId32::from(Into::<[u8; 32]>::into(data))
	}
}

impl hyperspace_evm::Config for Test {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressTruncated;
	type AddressMapping = HashedAddressMapping;
	type EtpCurrency = Etp;
	type DnaCurrency = Dna;
	type Event = ();
	type Precompiles = (
		hyperspace_evm_precompile_simple::ECRecover,
		hyperspace_evm_precompile_simple::Sha256,
		hyperspace_evm_precompile_simple::Ripemd160,
		hyperspace_evm_precompile_simple::Identity,
		hyperspace_evm_precompile_withdraw::WithDraw<Self>,
	);
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type Runner = hyperspace_evm::runner::stack::Runner<Self>;
	type EtpAccountBasic = DvmAccountBasic<Self, Etp, EtpRemainBalance>;
	type DnaAccountBasic = DvmAccountBasic<Self, Dna, DnaRemainBalance>;
	type IssuingHandler = EmptyIssuingHandler;
}

impl Config for Test {
	type Event = ();
	type FindAuthor = EthereumFindAuthor;
	type StateRoot = IntermediateStateRoot;
	type EtpCurrency = Etp;
	type DnaCurrency = Dna;
}

frame_support::construct_runtime! {
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage},
		Etp: hyperspace_balances::<Instance0>::{Pallet, Call, Storage, Config<T>},
		Dna: hyperspace_balances::<Instance1>::{Pallet, Call, Storage},
		EVM: hyperspace_evm::{Pallet, Call, Storage},
		Ethereum: dvm_ethereum::{Pallet, Call, Storage},
	}
}

pub struct AccountInfo {
	pub address: H160,
	pub account_id: AccountId32,
	pub private_key: H256,
}

fn address_build(seed: u8) -> AccountInfo {
	let private_key = H256::from_slice(&[(seed + 1) as u8; 32]); //H256::from_low + 1) as u64);
	let secret_key = secp256k1::SecretKey::parse_slice(&private_key[..]).unwrap();
	let public_key = &secp256k1::PublicKey::from_secret_key(&secret_key).serialize()[1..65];
	let address = H160::from(H256::from_slice(&Keccak256::digest(public_key)[..]));

	let mut data = [0u8; 32];
	data[0..20].copy_from_slice(&address[..]);

	AccountInfo {
		private_key,
		account_id: AccountId32::from(Into::<[u8; 32]>::into(data)),
		address,
	}
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext(accounts_len: usize) -> (Vec<AccountInfo>, sp_io::TestExternalities) {
	// sc_cli::init_logger("");
	let mut ext = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let pairs = (0..accounts_len)
		.map(|i| address_build(i as u8))
		.collect::<Vec<_>>();

	let balances: Vec<_> = (0..accounts_len)
		.map(|i| (pairs[i].account_id.clone(), 100_000_000_000))
		.collect();

	hyperspace_balances::GenesisConfig::<Test, EtpInstance> { balances }
		.assimilate_storage(&mut ext)
		.unwrap();

	(pairs, ext.into())
}

pub fn contract_address(sender: H160, nonce: u64) -> H160 {
	let mut rlp = RlpStream::new_list(2);
	rlp.append(&sender);
	rlp.append(&nonce);

	H160::from_slice(&Keccak256::digest(&rlp.out())[12..])
}

pub fn storage_address(sender: H160, slot: H256) -> H256 {
	H256::from_slice(&Keccak256::digest(
		[&H256::from(sender)[..], &slot[..]].concat().as_slice(),
	))
}

pub struct UnsignedTransaction {
	pub nonce: U256,
	pub gas_price: U256,
	pub gas_limit: U256,
	pub action: TransactionAction,
	pub value: U256,
	pub input: Vec<u8>,
}

impl UnsignedTransaction {
	fn signing_rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(9);
		s.append(&self.nonce);
		s.append(&self.gas_price);
		s.append(&self.gas_limit);
		s.append(&self.action);
		s.append(&self.value);
		s.append(&self.input);
		s.append(&ChainId::get());
		s.append(&0u8);
		s.append(&0u8);
	}

	fn signing_hash(&self) -> H256 {
		let mut stream = RlpStream::new();
		self.signing_rlp_append(&mut stream);
		H256::from_slice(&Keccak256::digest(&stream.out()).as_slice())
	}

	pub fn sign(&self, key: &H256) -> Transaction {
		let hash = self.signing_hash();
		let msg = secp256k1::Message::parse(hash.as_fixed_bytes());
		let s = secp256k1::sign(&msg, &secp256k1::SecretKey::parse_slice(&key[..]).unwrap());
		let sig = s.0.serialize();

		let sig = TransactionSignature::new(
			s.1.serialize() as u64 % 2 + ChainId::get() * 2 + 35,
			H256::from_slice(&sig[0..32]),
			H256::from_slice(&sig[32..64]),
		)
		.unwrap();

		Transaction {
			nonce: self.nonce,
			gas_price: self.gas_price,
			gas_limit: self.gas_limit,
			action: self.action,
			value: self.value,
			input: self.input.clone(),
			signature: sig,
		}
	}
}
