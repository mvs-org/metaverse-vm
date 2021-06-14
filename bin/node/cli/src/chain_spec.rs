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
use std::{collections::BTreeMap, marker::PhantomData};
// --- substrate ---
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};
// --- hyperspace ---
use hyperspace_claims::ClaimsList;
use hyperspace_ethereum_relay::DagsMerkleRootsLoader as DagsMerkleRootsLoaderR;
use hyperspace_evm::GenesisAccount;
use hyperspace_primitives::*;

pub type HyperspaceChainSpec = sc_service::GenericChainSpec<hyperspace_runtime::GenesisConfig>;

type AccountPublic = <Signature as Verify>::Signer;

const HYPERSPACE_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

const TEAM_MEMBERS: &[&'static str] = &[
	// Huiyi
	"0x281b7ec1e05feb46457caa9c54cef0ebdaf7f65d31fd6ed740a34dbc9875304c",
	// Ron
	"0x9cf0c0ea7488a17e348f0abba9c229032f3240a793ffcfbedc4b46db0aeb306c",
	// Cheng
	"0x922b6854052ba1084c74dd323ee70047d58ae4eb068f20bc251831f1ec109030",
	// Jane
	"0xb26268877f72c4dcd9c2459a99dde0d2caf5a816c6b4cd3bd1721252b26f4909",
	// Cai
	"0xf41d3260d736f5b3db8a6351766e97619ea35972546a5f850bbf0b27764abe03",
	// Tiny
	"0xf29638cb649d469c317a4c64381e179d5f64ef4d30207b4c52f2725c9d2ec533",
	// Eve
	"0x1a7008a33fa595398b509ef56841db3340931c28a42881e36c9f34b1f15f9271",
	// Yuqi
	"0x500e3197e075610c1925ddcd86d66836bf93ae0a476c64f56f611afc7d64d16f",
	// Aki
	"0x129f002b1c0787ea72c31b2dc986e66911fe1b4d6dc16f83a1127f33e5a74c7d",
	// Alex
	"0x26fe37ba5d35ac650ba37c5cc84525ed135e772063941ae221a1caca192fff49",
	// Shell
	"0x187c272f576b1999d6cf3dd529b59b832db12125b43e57fb088677eb0c570a6b",
	// Xavier
	"0xb4f7f03bebc56ebe96bc52ea5ed3159d45a0ce3a8d7f082983c33ef133274747",
	// Xuelei
	"0x88d388115bd0df43e805b029207cfa4925cecfb29026e345979d9b0004466c49",
];
const EVM_ACCOUNTS: &[&'static str] = &[
	"0x68898db1012808808c903f390909c52d9f706749",
	"0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
	"0xB90168C8CBcd351D069ffFdA7B71cd846924d551",
	// Echo
	"0x0f14341A7f464320319025540E8Fe48Ad0fe5aec",
	// for External Project
	"0x7682Ba569E3823Ca1B7317017F5769F8Aa8842D4",
	// Subswap
	"0xbB3E51d20CA651fBE19b1a1C2a6C8B1A4d950437",
];
const A_FEW_COINS: Balance = 1 << 44;
const MANY_COINS: Balance = A_FEW_COINS << 6;
const BUNCH_OF_COINS: Balance = MANY_COINS << 6;

const TOKEN_REDEEM_ADDRESS: &'static str = "0x49262B932E439271d05634c32978294C7Ea15d0C";
const DEPOSIT_REDEEM_ADDRESS: &'static str = "0x6EF538314829EfA8386Fc43386cB13B4e0A67D1e";
const SET_AUTHORITIES_ADDRESS: &'static str = "0xD35Bb6F1bc1C84b53E0995c1830454AB7C4147f1";
const ETP_TOKEN_ADDRESS: &'static str = "0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0";
const DNA_TOKEN_ADDRESS: &'static str = "0x1994100c58753793D52c6f457f189aa3ce9cEe94";
const ETHEREUM_RELAY_AUTHORITY_SIGNER: &'static str = "0x68898db1012808808c903f390909c52d9f706749";
const MAPPING_FACTORY_ADDRESS: &'static str = "0x6b58D3903Ae8997A5dA02FAAd51333D4Bf6958cC";
const ETHEREUM_BACKING_ADDRESS: &'static str = "0xbF6E8B2A6387952C39634f4cCF6Acf4FA2b99FA4";

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> hyperspace_runtime::SessionKeys {
	hyperspace_runtime::SessionKeys {
		babe,
		grandpa,
		im_online,
		authority_discovery,
	}
}

fn properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 18.into());
	properties.insert("tokenDecimals".into(), vec![9, 9].into());
	properties.insert("tokenSymbol".into(), vec!["PETP", "PDNA"].into());

	properties
}

fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn get_authority_keys_from_seed(
	s: &str,
) -> (
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<BabeId>(s),
		get_from_seed::<GrandpaId>(s),
		get_from_seed::<ImOnlineId>(s),
		get_from_seed::<AuthorityDiscoveryId>(s),
	)
}

pub fn hyperspace_config() -> Result<HyperspaceChainSpec, String> {
	HyperspaceChainSpec::from_json_bytes(&include_bytes!("../../../res/hyperspace/hyperspace.json")[..])
}

pub fn hyperspace_build_spec_config() -> HyperspaceChainSpec {
	HyperspaceChainSpec::from_genesis(
		"Hyperspace",
		"hyperspace",
		ChainType::Live,
		hyperspace_build_spec_genesis,
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(HYPERSPACE_TELEMETRY_URL.to_string(), 0)])
				.expect("Hyperspace telemetry url is valid; qed"),
		),
		None,
		Some(properties()),
		None,
	)
}

fn hyperspace_build_spec_genesis() -> hyperspace_runtime::GenesisConfig {
	struct Keys {
		stash: AccountId,
		session: hyperspace_runtime::SessionKeys,
	}
	impl Keys {
		fn new(sr25519: &str, ed25519: &str) -> Self {
			let sr25519 = array_bytes::hex2array_unchecked!(sr25519, 32);
			let ed25519 = array_bytes::hex2array_unchecked!(ed25519, 32);

			Self {
				stash: sr25519.into(),
				session: session_keys(
					sr25519.unchecked_into(),
					ed25519.unchecked_into(),
					sr25519.unchecked_into(),
					sr25519.unchecked_into(),
				),
			}
		}
	}

	let root = AccountId::from(array_bytes::hex2array_unchecked!(
		"0x72819fbc1b93196fa230243947c1726cbea7e33044c7eb6f736ff345561f9e4c",
		32
	));
	let initial_authorities = vec![
		Keys::new(
			"0x9c43c00407c0a51e0d88ede9d531f165e370013b648e6b62f4b3bcff4689df02",
			"0x63e122d962a835020bef656ad5a80dbcc994bb48a659f1af955552f4b3c27b09",
		),
		Keys::new(
			"0x741a9f507722713ec0a5df1558ac375f62469b61d1f60fa60f5dedfc85425b2e",
			"0x8a50704f41448fca63f608575debb626639ac00ad151a1db08af1368be9ccb1d",
		),
		Keys::new(
			"0x2276a3162f1b63c21b3396c5846d43874c5b8ba69917d756142d460b2d70d036",
			"0xb28fade2d023f08c0d5a131eac7d64a107a2660f22a0aca09b37a3f321259ef6",
		),
		Keys::new(
			"0x7a8b265c416eab5fdf8e5a1b3c7635131ca7164fbe6f66d8a70feeeba7c4dd7f",
			"0x305bafd512366e7fd535fdc144c7034b8683e1814d229c84a116f3cb27a97643",
		),
		Keys::new(
			"0xe446c1f1f419cc0927ad3319e141501b02844dee6252d905aae406f0c7097d1a",
			"0xc3c9880f6821b6e906c4396e54137297b1ee6c4c448b6a98abc5e29ffcdcec81",
		),
		Keys::new(
			"0xae05263d9508581f657ce584184721884ee2886eb66765db0c4f5195aa1d4e21",
			"0x1ed7de3855ffcce134d718b570febb49bbbbeb32ebbc8c319f44fb9f5690643a",
		),
	];
	let collective_members = vec![get_account_id_from_seed::<sr25519::Public>("Alice")];
	let evm_accounts = {
		let mut map = BTreeMap::new();

		for account in EVM_ACCOUNTS.iter() {
			map.insert(
				array_bytes::hex2array_unchecked!(account, 20).into(),
				GenesisAccount {
					nonce: 0.into(),
					balance: (MANY_COINS * (10 as Balance).pow(9)).into(),
					storage: BTreeMap::new(),
					code: vec![],
				},
			);
		}

		map
	};

	hyperspace_runtime::GenesisConfig {
		frame_system: hyperspace_runtime::SystemConfig {
			code: hyperspace_runtime::wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_babe: hyperspace_runtime::BabeConfig {
			authorities: vec![],
			epoch_config: Some(hyperspace_runtime::BABE_GENESIS_EPOCH_CONFIG)
		},
		hyperspace_balances_Instance0: hyperspace_runtime::BalancesConfig {
			balances: vec![
				(root.clone(), BUNCH_OF_COINS),
				(get_account_id_from_seed::<sr25519::Public>("Alice"), A_FEW_COINS),
			]
			.into_iter()
			.chain(
				initial_authorities
					.iter()
					.map(|Keys { stash, .. }| (stash.to_owned(), A_FEW_COINS)),
			)
			.chain(
				TEAM_MEMBERS
					.iter()
					.map(|m| (array_bytes::hex2array_unchecked!(m, 32).into(), MANY_COINS)),
			)
			.collect()
		},
		hyperspace_balances_Instance1: hyperspace_runtime::DnaConfig {
			balances: vec![(root.clone(), BUNCH_OF_COINS)]
				.into_iter()
				.chain(
					initial_authorities
						.iter()
						.map(|Keys { stash, .. }| (stash.to_owned(), A_FEW_COINS)),
				)
				.chain(
					TEAM_MEMBERS
						.iter()
						.map(|m| (array_bytes::hex2array_unchecked!(m, 32).into(), A_FEW_COINS)),
				)
				.collect()
		},
		hyperspace_staking: hyperspace_runtime::StakingConfig {
			minimum_validator_count: 6,
			validator_count: 6,
			stakers: initial_authorities
				.iter()
				.map(|Keys { stash, .. }| (
					stash.to_owned(),
					stash.to_owned(),
					A_FEW_COINS,
					hyperspace_runtime::StakerStatus::Validator
				))
				.collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perbill::from_percent(50),
			..Default::default()
		},
		pallet_session: hyperspace_runtime::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|Keys { stash, session }| (
					stash.to_owned(),
					stash.to_owned(),
					session.to_owned()
				))
				.collect(),
		},
		pallet_grandpa: Default::default(),
		pallet_im_online: Default::default(),
		pallet_authority_discovery: Default::default(),
		hyperspace_democracy: Default::default(),
		pallet_collective_Instance0: hyperspace_runtime::CouncilConfig {
			phantom: PhantomData::<hyperspace_runtime::CouncilCollective>,
			members: collective_members.clone(),
		},
		pallet_collective_Instance1: hyperspace_runtime::TechnicalCommitteeConfig {
			phantom: PhantomData::<hyperspace_runtime::TechnicalCollective>,
			members: collective_members
		},
		hyperspace_elections_phragmen: Default::default(),
		pallet_membership_Instance0: Default::default(),
		hyperspace_claims: Default::default(),
		hyperspace_vesting: Default::default(),
		pallet_sudo: hyperspace_runtime::SudoConfig { key: root.clone() },
		hyperspace_oldetp_issuing: hyperspace_runtime::HyperspaceOldetpIssuingConfig { total_mapped_etp: BUNCH_OF_COINS },
		hyperspace_oldetp_backing: hyperspace_runtime::HyperspaceOldetpBackingConfig { backed_etp: BUNCH_OF_COINS },
		hyperspace_ethereum_relay: hyperspace_runtime::EthereumRelayConfig {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 33, 123, 11, 188, 251, 114, 226, 213, 126, 40, 243, 60, 179, 97, 185, 152, 53, 19, 23, 119, 85, 220, 63, 51, 206, 62, 112, 34, 237, 98, 183, 123, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 65, 148, 16, 35, 104, 9, 35, 224, 254, 77, 116, 163, 75, 218, 200, 20, 31, 37, 64, 227, 174, 144, 98, 55, 24, 228, 125, 102, 209, 202, 74, 45],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoaderR::from_file(
				"bin/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		},
		hyperspace_ethereum_backing: hyperspace_runtime::EthereumBackingConfig {
			token_redeem_address: array_bytes::hex2array_unchecked!(TOKEN_REDEEM_ADDRESS, 20).into(),
			deposit_redeem_address: array_bytes::hex2array_unchecked!(DEPOSIT_REDEEM_ADDRESS, 20).into(),
			set_authorities_address: array_bytes::hex2array_unchecked!(SET_AUTHORITIES_ADDRESS, 20).into(),
			etp_token_address: array_bytes::hex2array_unchecked!(ETP_TOKEN_ADDRESS, 20).into(),
			dna_token_address: array_bytes::hex2array_unchecked!(DNA_TOKEN_ADDRESS, 20).into(),
			etp_locked: BUNCH_OF_COINS,
			dna_locked: BUNCH_OF_COINS,
		},
		hyperspace_ethereum_issuing: hyperspace_runtime::EthereumIssuingConfig {
			mapping_factory_address: array_bytes::hex2array_unchecked!(MAPPING_FACTORY_ADDRESS, 20).into(),
			ethereum_backing_address: array_bytes::hex2array_unchecked!(ETHEREUM_BACKING_ADDRESS, 20).into(),
		},
		hyperspace_relay_authorities_Instance0: hyperspace_runtime::EthereumRelayAuthoritiesConfig {
			authorities: vec![(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				array_bytes::hex2array_unchecked!(ETHEREUM_RELAY_AUTHORITY_SIGNER, 20).into(),
				1
			)]
		},
		hyperspace_oldna_backing: hyperspace_runtime::OldnaBackingConfig {
			backed_etp: BUNCH_OF_COINS,
			backed_dna: BUNCH_OF_COINS,
		},
		hyperspace_evm: hyperspace_runtime::EVMConfig { accounts: evm_accounts },
		dvm_ethereum: Default::default(),
	}
}

pub fn hyperspace_development_config() -> HyperspaceChainSpec {
	HyperspaceChainSpec::from_genesis(
		"Hyperspace",
		"hyperspace",
		ChainType::Development,
		hyperspace_development_genesis,
		vec![],
		None,
		None,
		Some(properties()),
		None,
	)
}

fn hyperspace_development_genesis() -> hyperspace_runtime::GenesisConfig {
	let root = get_account_id_from_seed::<sr25519::Public>("Alice");
	let initial_authorities = vec![get_authority_keys_from_seed("Alice")];
	let endowed_accounts = vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
	]
	.into_iter()
	.chain(
		TEAM_MEMBERS
			.iter()
			.map(|m| array_bytes::hex2array_unchecked!(m, 32).into()),
	)
	.collect::<Vec<_>>();
	let collective_members = vec![get_account_id_from_seed::<sr25519::Public>("Alice")];
	let evm_accounts = {
		let mut map = BTreeMap::new();

		for account in EVM_ACCOUNTS.iter() {
			map.insert(
				array_bytes::hex2array_unchecked!(account, 20).into(),
				GenesisAccount {
					nonce: 0.into(),
					balance: (123_456_789_000_000_000_000_090 as Balance).into(),
					storage: BTreeMap::new(),
					code: vec![],
				},
			);
		}

		map
	};

	hyperspace_runtime::GenesisConfig {
		frame_system: hyperspace_runtime::SystemConfig {
			code: hyperspace_runtime::wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_babe: hyperspace_runtime::BabeConfig {
			authorities: vec![],
			epoch_config: Some(hyperspace_runtime::BABE_GENESIS_EPOCH_CONFIG)
		},
		hyperspace_balances_Instance0: hyperspace_runtime::BalancesConfig {
			balances: endowed_accounts
				.clone()
				.into_iter()
				.map(|a| (a, MANY_COINS))
				.collect()
		},
		hyperspace_balances_Instance1: hyperspace_runtime::DnaConfig {
			balances: endowed_accounts
				.clone()
				.into_iter()
				.map(|a| (a, A_FEW_COINS))
				.collect()
		},
		hyperspace_staking: hyperspace_runtime::StakingConfig {
			minimum_validator_count: 1,
			validator_count: 2,
			stakers: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0, x.1, A_FEW_COINS, hyperspace_runtime::StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().cloned().map(|x| x.0).collect(),
			force_era: hyperspace_staking::Forcing::ForceAlways,
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perbill::from_percent(50),
			..Default::default()
		},
		pallet_session: hyperspace_runtime::SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0.clone(), x.0, session_keys(x.2, x.3, x.4, x.5)))
				.collect(),
		},
		pallet_grandpa: Default::default(),
		pallet_im_online: Default::default(),
		pallet_authority_discovery: Default::default(),
		hyperspace_democracy: Default::default(),
		pallet_collective_Instance0: hyperspace_runtime::CouncilConfig {
			phantom: PhantomData::<hyperspace_runtime::CouncilCollective>,
			members: collective_members.clone(),
		},
		pallet_collective_Instance1: hyperspace_runtime::TechnicalCommitteeConfig {
			phantom: PhantomData::<hyperspace_runtime::TechnicalCollective>,
			members: collective_members
		},
		hyperspace_elections_phragmen: Default::default(),
		pallet_membership_Instance0: Default::default(),
		hyperspace_claims: hyperspace_runtime::ClaimsConfig {
			claims_list: ClaimsList::from_file(
				"bin/res/claims-list.json",
				"CLAIMS_LIST_PATH",
			),
		},
		hyperspace_vesting: Default::default(),
		pallet_sudo: hyperspace_runtime::SudoConfig { key: root.clone() },
		hyperspace_oldetp_issuing: hyperspace_runtime::HyperspaceOldetpIssuingConfig { total_mapped_etp: BUNCH_OF_COINS },
		hyperspace_oldetp_backing: hyperspace_runtime::HyperspaceOldetpBackingConfig { backed_etp: BUNCH_OF_COINS },
		hyperspace_ethereum_relay: hyperspace_runtime::EthereumRelayConfig {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 33, 123, 11, 188, 251, 114, 226, 213, 126, 40, 243, 60, 179, 97, 185, 152, 53, 19, 23, 119, 85, 220, 63, 51, 206, 62, 112, 34, 237, 98, 183, 123, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 65, 148, 16, 35, 104, 9, 35, 224, 254, 77, 116, 163, 75, 218, 200, 20, 31, 37, 64, 227, 174, 144, 98, 55, 24, 228, 125, 102, 209, 202, 74, 45],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoaderR::from_file(
				"bin/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		},
		hyperspace_ethereum_backing: hyperspace_runtime::EthereumBackingConfig {
			token_redeem_address: array_bytes::hex2array_unchecked!(TOKEN_REDEEM_ADDRESS, 20).into(),
			deposit_redeem_address: array_bytes::hex2array_unchecked!(DEPOSIT_REDEEM_ADDRESS, 20).into(),
			set_authorities_address: array_bytes::hex2array_unchecked!(SET_AUTHORITIES_ADDRESS, 20).into(),
			etp_token_address: array_bytes::hex2array_unchecked!(ETP_TOKEN_ADDRESS, 20).into(),
			dna_token_address: array_bytes::hex2array_unchecked!(DNA_TOKEN_ADDRESS, 20).into(),
			etp_locked: BUNCH_OF_COINS,
			dna_locked: BUNCH_OF_COINS,
		},
		hyperspace_ethereum_issuing: hyperspace_runtime::EthereumIssuingConfig {
			mapping_factory_address: array_bytes::hex2array_unchecked!(MAPPING_FACTORY_ADDRESS, 20).into(),
			ethereum_backing_address: array_bytes::hex2array_unchecked!(ETHEREUM_BACKING_ADDRESS, 20).into(),
		},
		hyperspace_relay_authorities_Instance0: hyperspace_runtime::EthereumRelayAuthoritiesConfig {
			authorities: vec![(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				array_bytes::hex2array_unchecked!(ETHEREUM_RELAY_AUTHORITY_SIGNER, 20).into(),
				1
			)]
		},
		hyperspace_oldna_backing: hyperspace_runtime::OldnaBackingConfig {
			backed_etp: BUNCH_OF_COINS,
			backed_dna: BUNCH_OF_COINS,
		},
		hyperspace_evm: hyperspace_runtime::EVMConfig { accounts: evm_accounts },
		dvm_ethereum: Default::default(),
	}
}
