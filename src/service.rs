//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use std::sync::Arc;
use std::str::FromStr;
use std::time::Duration;
use std::thread;
use parking_lot::Mutex;
use codec::Encode;
use sp_runtime::{Perbill, generic::BlockId, traits::Bounded};
use sp_core::{H256, crypto::{UncheckedFrom, Ss58Codec, Ss58AddressFormat}};
use sc_service::{error::{Error as ServiceError}, Configuration, TaskManager};
use sc_executor::native_executor_instance;
use sc_client_api::backend::RemoteBackend;
use betelgeuse_runtime::{self, opaque::Block, RuntimeApi};
use log::*;
//GRANDPA
//use sc_finality_grandpa::{FinalityProofProvider as GrandpaFinalityProofProvider, StorageAndProofProvider};
use sc_finality_grandpa::{
	self, FinalityProofProvider as GrandpaFinalityProofProvider, GrandpaBlockImport,StorageAndProofProvider
};

pub use sc_executor::NativeExecutor;

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	betelgeuse_runtime::api::dispatch,
	betelgeuse_runtime::native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);

pub fn decode_author(
	author: Option<&str>,
) -> Option<betelgeuse_pow::app::Public> {
	author.map(|author| {
		if author.starts_with("0x") {
			betelgeuse_pow::app::Public::unchecked_from(
				H256::from_str(&author[2..]).expect("Invalid author account")
			).into()
		} else {
			let (address, version) = betelgeuse_pow::app::Public::from_ss58check_with_version(author)
				.expect("Invalid author address");
			assert!(version == Ss58AddressFormat::KulupuAccount, "Invalid author version");
			address
		}
	})
}

type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// Inherent data provider for Betelgeuse.
pub fn betelgeuse_inherent_data_providers(
	author: Option<betelgeuse_pow::app::Public>, donate: bool,
) -> Result<sp_inherents::InherentDataProviders, ServiceError> {
	let inherent_data_providers = sp_inherents::InherentDataProviders::new();

	if !inherent_data_providers.has_provider(&sp_timestamp::INHERENT_IDENTIFIER) {
		inherent_data_providers
			.register_provider(sp_timestamp::InherentDataProvider)
			.map_err(Into::into)
			.map_err(sp_consensus::Error::InherentData)?;
	}

	if let Some(author) = author {
		let encoded_author = author.encode();

		if !inherent_data_providers.has_provider(&pallet_rewards::INHERENT_IDENTIFIER_V0) {
			inherent_data_providers
				.register_provider(pallet_rewards::InherentDataProviderV0(
					encoded_author.clone(),
				))
				.map_err(Into::into)
				.map_err(sp_consensus::Error::InherentData)?;
		}

		if !inherent_data_providers.has_provider(&pallet_rewards::INHERENT_IDENTIFIER) {
			inherent_data_providers
				.register_provider(pallet_rewards::InherentDataProvider(
					(encoded_author, if donate { Perbill::max_value() } else { Perbill::zero() })
				))
				.map_err(Into::into)
				.map_err(sp_consensus::Error::InherentData)?;
		}
	}

	Ok(inherent_data_providers)
}

pub fn new_partial(
	config: &Configuration,
	author: Option<&str>,
	check_inherents_after: u32,
	donate: bool,
	enable_weak_subjectivity: bool,
) -> Result<sc_service::PartialComponents<
	FullClient, FullBackend, FullSelectChain,
	sp_consensus::DefaultImportQueue<Block, FullClient>,
	sc_transaction_pool::FullPool<Block, FullClient>,
	//sc_consensus_pow::PowBlockImport<Block, betelgeuse_pow::weak_sub::WeakSubjectiveBlockImport<Block, Arc<FullClient>, FullClient, FullSelectChain, betelgeuse_pow::RandomXAlgorithm<FullClient>, betelgeuse_pow::weak_sub::ExponentialWeakSubjectiveAlgorithm>, FullClient, FullSelectChain, betelgeuse_pow::RandomXAlgorithm<FullClient>, sp_consensus::AlwaysCanAuthor>,
	(
		//sc_consensus_pow::PowBlockImport<Block, betelgeuse_pow::weak_sub::WeakSubjectiveBlockImport<Block, Arc<FullClient>, FullClient, FullSelectChain, betelgeuse_pow::RandomXAlgorithm<FullClient>, betelgeuse_pow::weak_sub::ExponentialWeakSubjectiveAlgorithm>, FullClient, FullSelectChain, betelgeuse_pow::RandomXAlgorithm<FullClient>, sp_consensus::AlwaysCanAuthor>,
		sc_consensus_pow::PowBlockImport<Block, GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>, FullClient, FullSelectChain, betelgeuse_pow::RandomXAlgorithm<FullClient>, sp_consensus::AlwaysCanAuthor>,
		sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>
	)
>, ServiceError> {
	let inherent_data_providers = crate::service::betelgeuse_inherent_data_providers(
		decode_author(author),
		donate,
	)?;

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);
//GRANDPA
	//let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
	//	client.clone(),
	//	&(client.clone() as Arc<_>),
	//	select_chain.clone(),
	//)?;
	let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
		client.clone(),
		&(client.clone() as std::sync::Arc<_>),
		select_chain.clone(),
	)?;
//end GRANDPA
	let algorithm = betelgeuse_pow::RandomXAlgorithm::new(client.clone());

	let weak_sub_block_import = betelgeuse_pow::weak_sub::WeakSubjectiveBlockImport::new(
		client.clone(),
		client.clone(),
		algorithm.clone(),
		betelgeuse_pow::weak_sub::ExponentialWeakSubjectiveAlgorithm(30, 1.1),
		select_chain.clone(),
		enable_weak_subjectivity,
	);

	let pow_block_import = sc_consensus_pow::PowBlockImport::new(
		//GRANDPA
		//weak_sub_block_import,
		grandpa_block_import,
		client.clone(),
		algorithm.clone(),
		check_inherents_after,
		select_chain.clone(),
		inherent_data_providers.clone(),
		sp_consensus::AlwaysCanAuthor,
	);


	let import_queue = sc_consensus_pow::import_queue(
		Box::new(pow_block_import.clone()),
		None,
		algorithm.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	)?;

	Ok(sc_service::PartialComponents {
		client, backend, task_manager, import_queue, keystore_container,
		select_chain, transaction_pool, inherent_data_providers,
		other: (pow_block_import, grandpa_link),
	})
}

/// Builds a new service for a full client.
pub fn new_full(
	config: Configuration,
	author: Option<&str>,
	threads: usize,
	round: u32,
	check_inherents_after: u32,
	donate: bool,
	enable_weak_subjectivity: bool,
) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client, backend, mut task_manager, import_queue, keystore_container,
		select_chain, transaction_pool, inherent_data_providers,
		other: (pow_block_import, grandpa_link),
	} = new_partial(&config, author, check_inherents_after, donate, enable_weak_subjectivity)?;

	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config, backend.clone(), task_manager.spawn_handle(), client.clone(), network.clone(),
		);
	}

	let role = config.role.clone();
	let prometheus_registry = config.prometheus_registry().cloned();
	let telemetry_connection_sinks = sc_service::TelemetryConnectionSinks::default();
	//GRANDPA
	let is_authority = config.role.is_authority();
	let provider = client.clone() as Arc<dyn StorageAndProofProvider<_, _>>;
	let finality_proof_provider =
		GrandpaFinalityProofProvider::new_for_service(backend.clone(), client.clone());


	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				deny_unsafe,
			};

			crate::rpc::create_full(deps)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		telemetry_connection_sinks: telemetry_connection_sinks.clone(),
		rpc_extensions_builder: rpc_extensions_builder,
		on_demand: None,
		remote_blockchain: None,
		backend, network_status_sinks, system_rpc_tx, config,
	})?;

	


	if role.is_authority() {
		let author = decode_author(author);
		let algorithm = betelgeuse_pow::RandomXAlgorithm::new(
			client.clone(),
		);

		let proposer = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
		);

		let (worker, worker_task) = sc_consensus_pow::start_mining_worker(
			Box::new(pow_block_import.clone()),
			client.clone(),
			select_chain.clone(),
			algorithm,
			proposer,
			network.clone(),
			author.clone().map(|a| a.encode()),
			inherent_data_providers.clone(),
			Duration::new(10, 0),
			Duration::new(10, 0),
			sp_consensus::AlwaysCanAuthor,
		);
		task_manager.spawn_essential_handle().spawn_blocking("pow", worker_task);

		let stats = Arc::new(Mutex::new(betelgeuse_pow::Stats::new()));

		for _ in 0..threads {
			if let Some(keystore) = keystore_container.local_keystore() {
				let worker = worker.clone();
				let client = client.clone();
				let stats = stats.clone();

				thread::spawn(move || {
					loop {
						let metadata = worker.lock().metadata();
						if let Some(metadata) = metadata {
							match betelgeuse_pow::mine(
								client.as_ref(),
								&keystore,
								&BlockId::Hash(metadata.best_hash),
								&metadata.pre_hash,
								metadata.pre_runtime.as_ref().map(|v| &v[..]),
								metadata.difficulty,
								round,
								&stats
							) {
								Ok(Some(seal)) => {
									let mut worker = worker.lock();
									let current_metadata = worker.metadata();
									if current_metadata == Some(metadata) {
										let _ = worker.submit(seal);
									}
								},
								Ok(None) => (),
								Err(err) => {
									warn!("Mining failed: {:?}", err);
								},
							}
						} else {
							thread::sleep(Duration::new(1, 0));
						}
					}
				});
			} else {
				warn!("Local keystore is not available");
			}
		}
	}

	//GRANDPA
	// if the node isn't actively participating in consensus then it doesn't
	// need a keystore, regardless of which protocol we use below.
	//let keystore_container = if is_authority {
	//	Some(keystore_container as sp_core::traits::BareCryptoStorePtr)
	//} else {
	//	None
	//};
	let keystore = if role.is_authority() {
		Some(keystore_container.sync_keystore())
	} else {
		None
	};

	let grandpa_config = sc_finality_grandpa::Config {
		gossip_duration: Duration::from_millis(333),
		justification_period: 512,
		name: None,
		observer_enabled: false,
		keystore,
		is_authority,
	};
	//enable GRANDPA
	let grandpa_config = sc_finality_grandpa::GrandpaParams {
		config: grandpa_config,
		link: grandpa_link,
		network,
		telemetry_on_connect: Some(telemetry_connection_sinks.on_connect_stream()),
		voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
		prometheus_registry,
		shared_voter_state: sc_finality_grandpa::SharedVoterState::empty(),
	};

	// the GRANDPA voter task is considered infallible, i.e.
	// if it fails we take down the service with it.
	task_manager.spawn_essential_handle().spawn_blocking(
		"grandpa-voter",
		sc_finality_grandpa::run_grandpa_voter(grandpa_config)?
	);

	//GRANDPA - END

	network_starter.start_network();
	Ok(task_manager)
}

/// Builds a new service for a light client.
pub fn new_light(
	config: Configuration,
	author: Option<&str>,
	check_inherents_after: u32,
	donate: bool,
	enable_weak_subjectivity: bool,
) -> Result<TaskManager, ServiceError> {
	let (client, backend, keystore_container, mut task_manager, on_demand) =
		sc_service::new_light_parts::<Block, RuntimeApi, Executor>(&config)?;

	let transaction_pool = Arc::new(sc_transaction_pool::BasicPool::new_light(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
		on_demand.clone(),
	));

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let inherent_data_providers = betelgeuse_inherent_data_providers(decode_author(author), donate)?;

	let algorithm = betelgeuse_pow::RandomXAlgorithm::new(client.clone());

	let weak_sub_block_import = betelgeuse_pow::weak_sub::WeakSubjectiveBlockImport::new(
		client.clone(),
		client.clone(),
		algorithm.clone(),
		betelgeuse_pow::weak_sub::ExponentialWeakSubjectiveAlgorithm(30, 1.1),
		select_chain.clone(),
		enable_weak_subjectivity,
	);

	let pow_block_import = sc_consensus_pow::PowBlockImport::new(
		weak_sub_block_import,
		client.clone(),
		algorithm.clone(),
		check_inherents_after,
		select_chain.clone(),
		inherent_data_providers.clone(),
		sp_consensus::AlwaysCanAuthor,
	);

	let import_queue = sc_consensus_pow::import_queue(
		Box::new(pow_block_import.clone()),
		None,
		algorithm.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	)?;

	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: Some(on_demand.clone()),
			block_announce_validator_builder: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config, backend.clone(), task_manager.spawn_handle(), client.clone(), network.clone(),
		);
	}

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		remote_blockchain: Some(backend.remote_blockchain()),
		transaction_pool,
		task_manager: &mut task_manager,
		on_demand: Some(on_demand),
		rpc_extensions_builder: Box::new(|_, _| ()),
		telemetry_connection_sinks: sc_service::TelemetryConnectionSinks::default(),
		config,
		client,
		keystore: keystore_container.sync_keystore(),
		backend,
		network,
		network_status_sinks,
		system_rpc_tx,
	 })?;

	 network_starter.start_network();

	 Ok(task_manager)
}
