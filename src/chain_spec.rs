use serde_json::json;
use sp_core::{U256, crypto::UncheckedFrom};
use sp_runtime::Perbill;
use sc_service::ChainType;
use betelgeuse_primitives::DOLLARS;
use betelgeuse_runtime::{
	BalancesConfig, GenesisConfig, GrandpaConfig, Signature, IndicesConfig, SystemConfig,
	DifficultyConfig, ErasConfig, AccountId, RewardsConfig, WASM_BINARY,
};
use sp_finality_grandpa::AuthorityId as GrandpaId;

use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
        TPublic::Pair::from_string(&format!("//{}", seed), None)
                .expect("static values are valid; qed")
                .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
        AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
        AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate session key from seed
pub fn authority_keys_from_seed(seed: &str) -> GrandpaId {
	get_from_seed::<GrandpaId>(seed)
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"betelgeuse",
		"dev",
		ChainType::Live,
		move || testnet_genesis(
			wasm_binary,
			U256::from(1000),
			vec![authority_keys_from_seed("Alice")],
			account_id_from_seed::<sr25519::Public>("Alice"),
			// Endowed Accounts
			vec![
			account_id_from_seed::<sr25519::Public>("Alice"),
			account_id_from_seed::<sr25519::Public>("Bob"),
			account_id_from_seed::<sr25519::Public>("Alice//stash"),
			account_id_from_seed::<sr25519::Public>("Bob//stash"),
			],
		),
		vec![],
		None,
		Some("betelgeuse"),
		Some(json!({
			"ss58Format": 16,
			"tokenDecimals": 12,
			"tokenSymbol": "ETP3"
		}).as_object().expect("Created an object").clone()),
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"Local Testnet",
		"local",
		ChainType::Local,
		 move || testnet_genesis(
                        wasm_binary,
                        U256::from(1000),
                        vec![authority_keys_from_seed("Alice")],
                        account_id_from_seed::<sr25519::Public>("Alice"),
                        // Endowed Accounts
                        vec![
                        account_id_from_seed::<sr25519::Public>("Alice"),
                        account_id_from_seed::<sr25519::Public>("Bob"),
                        account_id_from_seed::<sr25519::Public>("Alice//stash"),
                        account_id_from_seed::<sr25519::Public>("Bob//stash"),
                        ],
                ),
		vec![],
		None,
		Some("betelgeuselocal"),
		Some(json!({
			"ss58Format": 16,
			"tokenDecimals": 12,
			"tokenSymbol": "ETP3"
		}).as_object().expect("Created an object").clone()),
		None,
	))
}

pub fn breaknet4_config() -> ChainSpec {
	ChainSpec::from_json_bytes(&include_bytes!("../res/eras/1/config.json")[..])
		.expect("Breaknet4 config included is valid")
}

pub fn mainnet_config() -> ChainSpec {
	ChainSpec::from_json_bytes(&include_bytes!("../res/eras/1/config.json")[..])
		.expect("Mainnet config included is valid")
}

fn testnet_genesis(wasm_binary: &[u8], initial_difficulty: U256, initial_authorities: Vec<GrandpaId>, root_key: AccountId, endowed_accounts: Vec<AccountId>,) -> GenesisConfig {
	GenesisConfig {
		system: Some(SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		//balances: Some(BalancesConfig {
		//	balances: vec![],
		//}),
		balances: Some(BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		}),
		indices: Some(IndicesConfig {
			indices: vec![],
		}),
		difficulty: Some(DifficultyConfig {
			initial_difficulty,
		}),
		collective_Instance1: Some(Default::default()),
		collective_Instance2: Some(Default::default()),
		democracy: Some(Default::default()),
		treasury: Some(Default::default()),
		elections_phragmen: Some(Default::default()),
		eras: Some(Default::default()),
		membership_Instance1: Some(Default::default()),
		vesting: Some(Default::default()),
		rewards: Some(RewardsConfig {
			reward: 60 * DOLLARS,
			taxation: Perbill::from_percent(0),
			curve: vec![],
			additional_rewards: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
			//authorities: vec![],
			authorities: initial_authorities.iter().map(|x| (x.clone(), 1)).collect(),
		}),
	}
}

/// Swamp bottom genesis config generation.
#[allow(unused)]
pub fn mainnet_genesis() -> GenesisConfig {
	let era_state = crate::eras::era0_state();

	GenesisConfig {
		system: Some(SystemConfig {
			code: include_bytes!("../res/eras/1/betelgeuse_runtime.compact.wasm").to_vec(),
			changes_trie_config: Default::default(),
		}),
		balances: Some(BalancesConfig {
			balances: era_state.balances.into_iter().map(|balance| {
				(AccountId::unchecked_from(balance.address), balance.balance.as_u128())
			}).collect(),
		}),
		indices: Some(IndicesConfig {
			indices: era_state.indices.into_iter().map(|index| {
				(index.index, AccountId::unchecked_from(index.address))
			}).collect(),
		}),
		difficulty: Some(DifficultyConfig {
			initial_difficulty: era_state.difficulty,
		}),
		eras: Some(ErasConfig {
			past_eras: vec![
				pallet_eras::Era {
					genesis_block_hash: era_state.previous_era.genesis_block_hash,
					final_block_hash: era_state.previous_era.final_block_hash,
					final_state_root: era_state.previous_era.final_state_root,
				}
			],
		}),
		collective_Instance1: Some(Default::default()),
		collective_Instance2: Some(Default::default()),
		democracy: Some(Default::default()),
		treasury: Some(Default::default()),
		elections_phragmen: Some(Default::default()),
		membership_Instance1: Some(Default::default()),
		vesting: None,
		rewards: Some(RewardsConfig {
			reward: 60 * DOLLARS,
			taxation: Perbill::from_percent(0),
			curve: vec![],
			additional_rewards: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
                        authorities: vec![],
                }),
	}
}
