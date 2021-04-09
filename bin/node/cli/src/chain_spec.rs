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
use std::collections::BTreeMap;
// --- substrate ---
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public, H160};
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
use hyperspace_runtime::{constants::COIN, BalancesConfig as EtpConfig, *};

pub type HyperspaceChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

type AccountPublic = <Signature as Verify>::Signer;

const PANGOLIN_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

pub fn hyperspace_config() -> Result<HyperspaceChainSpec, String> {
	HyperspaceChainSpec::from_json_bytes(&include_bytes!("../../../res/hyperspace/hyperspace.json")[..])
}

pub fn hyperspace_session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys {
		babe,
		grandpa,
		im_online,
		authority_discovery,
	}
}

pub fn properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(),150.into());
	properties.insert("tokenDecimals".into(), vec![8, 8].into());
	properties.insert("tokenSymbol".into(), vec!["ETP", "DNA"].into());

	properties
}

pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_authority_keys_from_seed(
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

pub fn hyperspace_build_spec_config() -> HyperspaceChainSpec {
	HyperspaceChainSpec::from_genesis(
		"Hyperspace",
		"hyperspace",
		ChainType::Live,
		hyperspace_build_spec_genesis,
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(PANGOLIN_TELEMETRY_URL.to_string(), 0)])
				.expect("Hyperspace telemetry url is valid; qed"),
		),
		None,
		Some(properties()),
		None,
	)
}

fn hyperspace_build_spec_genesis() -> GenesisConfig {
	const ROOT: &'static str = "0x72819fbc1b93196fa230243947c1726cbea7e33044c7eb6f736ff345561f9e4c";
	const GENESIS_VALIDATOR: &'static str = "Alice";
	const GENESIS_VALIDATOR_STASH: &'static str = "Alice//stash";
	const GENESIS_VALIDATOR_BOND: Balance = COIN;
	const GENESIS_EVM_ACCOUNT: &'static str = "0x68898db1012808808c903f390909c52d9f706749";
	const GENESIS_ETHEREUM_RELAY_AUTHORITY_SIGNER: &'static str =
		"0x6aA70f55E5D770898Dd45aa1b7078b8A80AAbD6C";

	const TOKEN_REDEEM_ADDRESS: &'static str = "0x49262B932E439271d05634c32978294C7Ea15d0C";
	const DEPOSIT_REDEEM_ADDRESS: &'static str = "0x6EF538314829EfA8386Fc43386cB13B4e0A67D1e";
	const SET_AUTHORITIES_ADDRESS: &'static str = "0xE4A2892599Ad9527D76Ce6E26F93620FA7396D85";
	const ETP_TOKEN_ADDRESS: &'static str = "0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0";
	const DNA_TOKEN_ADDRESS: &'static str = "0x1994100c58753793D52c6f457f189aa3ce9cEe94";

	let root = AccountId::from(array_bytes::hex2array_unchecked!(ROOT, 32));
	let evm = array_bytes::hex2array_unchecked!(GENESIS_EVM_ACCOUNT, 20).into();
	let initial_authorities = vec![get_authority_keys_from_seed(GENESIS_VALIDATOR)];
	let endowed_accounts = vec![
		(root.clone(), 1 << 56),
		(
			get_account_id_from_seed::<sr25519::Public>(GENESIS_VALIDATOR_STASH),
			GENESIS_VALIDATOR_BOND,
		),
	];
	let mut evm_accounts = BTreeMap::new();

	evm_accounts.insert(
		evm,
		GenesisAccount {
			nonce: 0.into(),
			balance: 20_000_000_000_000_000_000_000_000u128.into(),
			storage: BTreeMap::new(),
			code: vec![],
		},
	);

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
		hyperspace_balances_Instance0: Some(EtpConfig { balances: endowed_accounts }),
		hyperspace_balances_Instance1: Some(Default::default()),
		hyperspace_staking: Some(StakingConfig {
			minimum_validator_count: 2,
			validator_count: 7,
			stakers: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0, x.1, GENESIS_VALIDATOR_BOND, StakerStatus::Validator))
				.collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perbill::from_percent(50),
			..Default::default()
		}),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0.clone(), x.0, hyperspace_session_keys(x.2, x.3, x.4, x.5)))
				.collect(),
		}),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(Default::default()),
		hyperspace_democracy: Some(Default::default()),
		pallet_collective_Instance0: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		hyperspace_elections_phragmen: Some(Default::default()),
		pallet_membership_Instance0: Some(Default::default()),
		hyperspace_claims: Some(Default::default()),
		hyperspace_vesting: Some(Default::default()),
		pallet_sudo: Some(SudoConfig { key: root.clone() }),
		hyperspace_oldna_issuing: Some(OldnaIssuingConfig {
			total_mapped_etp: 1 << 56
		}),
		hyperspace_oldna_backing: Some(OldnaBackingConfig {
			backed_etp: 1 << 56
		}),
		hyperspace_ethereum_backing: Some(EthereumBackingConfig {
			token_redeem_address: array_bytes::hex2array_unchecked!(TOKEN_REDEEM_ADDRESS, 20).into(),
			deposit_redeem_address: array_bytes::hex2array_unchecked!(DEPOSIT_REDEEM_ADDRESS, 20).into(),
			set_authorities_address: array_bytes::hex2array_unchecked!(SET_AUTHORITIES_ADDRESS, 20).into(),
			etp_token_address: array_bytes::hex2array_unchecked!(ETP_TOKEN_ADDRESS, 20).into(),
			dna_token_address: array_bytes::hex2array_unchecked!(DNA_TOKEN_ADDRESS, 20).into(),
			etp_locked: 1 << 56,
			dna_locked: 1 << 56,
		}),
		hyperspace_ethereum_relay: Some(EthereumRelayConfig {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 33, 123, 11, 188, 251, 114, 226, 213, 126, 40, 243, 60, 179, 97, 185, 152, 53, 19, 23, 119, 85, 220, 63, 51, 206, 62, 112, 34, 237, 98, 183, 123, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 65, 148, 16, 35, 104, 9, 35, 224, 254, 77, 116, 163, 75, 218, 200, 20, 31, 37, 64, 227, 174, 144, 98, 55, 24, 228, 125, 102, 209, 202, 74, 45],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoaderR::from_file(
				"bin/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		}),
		hyperspace_oldetp_backing: Some(OldetpBackingConfig {
			backed_etp: 1 << 56,
			backed_dna: 1 << 56,
		}),
		hyperspace_evm: Some(EVMConfig {
			accounts: evm_accounts,
		}),
		dvm_ethereum: Some(Default::default()),
		hyperspace_relay_authorities_Instance0: Some(EthereumRelayAuthoritiesConfig {
			authorities: vec![(root, array_bytes::hex2array_unchecked!(GENESIS_ETHEREUM_RELAY_AUTHORITY_SIGNER, 20).into(), 1)]
		}),
	}
}

pub fn hyperspace_development_config() -> HyperspaceChainSpec {
	HyperspaceChainSpec::from_genesis(
		"Development",
		"hyperspace_dev",
		ChainType::Development,
		|| {
			let initial_evm_account = vec![
				array_bytes::hex2array_unchecked!("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b", 20)
					.into(),
				array_bytes::hex2array_unchecked!("0xB90168C8CBcd351D069ffFdA7B71cd846924d551", 20)
					.into(),
			];
			let mut evm_accounts = BTreeMap::new();

			for account_id in initial_evm_account.iter() {
				evm_accounts.insert(
					*account_id,
					GenesisAccount {
						nonce: 0.into(),
						balance: 123_456_789_000_000_000_090u128.into(),
						storage: BTreeMap::new(),
						code: vec![],
					},
				);
			}

			hyperspace_development_genesis(
				vec![get_authority_keys_from_seed("Alice")],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				evm_accounts,
			)
		},
		vec![],
		None,
		None,
		Some(properties()),
		None,
	)
}

fn hyperspace_development_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	mut endowed_accounts: Vec<AccountId>,
	evm_accounts: BTreeMap<H160, GenesisAccount>,
) -> GenesisConfig {
	const GENESIS_ETHEREUM_RELAY_AUTHORITY_SIGNER: &'static str =
		"0x6aA70f55E5D770898Dd45aa1b7078b8A80AAbD6C";

	const TOKEN_REDEEM_ADDRESS: &'static str = "0x49262B932E439271d05634c32978294C7Ea15d0C";
	const DEPOSIT_REDEEM_ADDRESS: &'static str = "0x6EF538314829EfA8386Fc43386cB13B4e0A67D1e";
	const SET_AUTHORITIES_ADDRESS: &'static str = "0xE4A2892599Ad9527D76Ce6E26F93620FA7396D85";
	const ETP_TOKEN_ADDRESS: &'static str = "0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0";
	const DNA_TOKEN_ADDRESS: &'static str = "0x1994100c58753793D52c6f457f189aa3ce9cEe94";

	initial_authorities.iter().for_each(|x| {
		if !endowed_accounts.contains(&x.0) {
			endowed_accounts.push(x.0.clone())
		}
	});

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_babe: Some(Default::default()),
		hyperspace_balances_Instance0: Some(EtpConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 56))
				.collect(),
		}),
		hyperspace_balances_Instance1: Some(DnaConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 56))
				.collect(),
		}),
		hyperspace_staking: Some(StakingConfig {
			minimum_validator_count: 1,
			validator_count: 2,
			stakers: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0, x.1, 1 << 56, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().cloned().map(|x| x.0).collect(),
			force_era: hyperspace_staking::Forcing::ForceAlways,
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perbill::from_percent(50),
			..Default::default()
		}),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|x| (x.0.clone(), x.0, hyperspace_session_keys(x.2, x.3, x.4, x.5)))
				.collect(),
		}),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(Default::default()),
		hyperspace_democracy: Some(Default::default()),
		pallet_collective_Instance0: Some(Default::default()),
		pallet_collective_Instance1: Some(Default::default()),
		hyperspace_elections_phragmen: Some(Default::default()),
		pallet_membership_Instance0: Some(Default::default()),
		hyperspace_claims: Some(ClaimsConfig {
			claims_list: ClaimsList::from_file(
				"bin/res/claims-list.json",
				"CLAIMS_LIST_PATH",
			),
		}),
		hyperspace_vesting: Some(Default::default()),
		pallet_sudo: Some(SudoConfig { key: root_key.clone() }),
		hyperspace_oldna_issuing: Some(OldnaIssuingConfig {
			total_mapped_etp: 1 << 56
		}),
		hyperspace_oldna_backing: Some(OldnaBackingConfig {
			backed_etp: 1 << 56
		}),
		hyperspace_ethereum_backing: Some(EthereumBackingConfig {
			token_redeem_address: array_bytes::hex2array_unchecked!(TOKEN_REDEEM_ADDRESS, 20).into(),
			deposit_redeem_address: array_bytes::hex2array_unchecked!(DEPOSIT_REDEEM_ADDRESS, 20).into(),
			set_authorities_address: array_bytes::hex2array_unchecked!(SET_AUTHORITIES_ADDRESS, 20).into(),
			etp_token_address: array_bytes::hex2array_unchecked!(ETP_TOKEN_ADDRESS, 20).into(),
			dna_token_address: array_bytes::hex2array_unchecked!(DNA_TOKEN_ADDRESS, 20).into(),
			etp_locked: 1 << 56,
			dna_locked: 1 << 56,
		}),
		hyperspace_ethereum_relay: Some(EthereumRelayConfig {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 33, 123, 11, 188, 251, 114, 226, 213, 126, 40, 243, 60, 179, 97, 185, 152, 53, 19, 23, 119, 85, 220, 63, 51, 206, 62, 112, 34, 237, 98, 183, 123, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 65, 148, 16, 35, 104, 9, 35, 224, 254, 77, 116, 163, 75, 218, 200, 20, 31, 37, 64, 227, 174, 144, 98, 55, 24, 228, 125, 102, 209, 202, 74, 45],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoaderR::from_file(
				"bin/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		}),
		hyperspace_oldetp_backing: Some(OldetpBackingConfig {
			backed_etp: 1 << 56,
			backed_dna: 1 << 56,
		}),
		hyperspace_evm: Some(EVMConfig {
			accounts: evm_accounts,
		}),
		dvm_ethereum: Some(Default::default()),
		hyperspace_relay_authorities_Instance0: Some(EthereumRelayAuthoritiesConfig {
			authorities: vec![(root_key, array_bytes::hex2array_unchecked!(GENESIS_ETHEREUM_RELAY_AUTHORITY_SIGNER, 20).into(), 1)]
		}),
	}
}
