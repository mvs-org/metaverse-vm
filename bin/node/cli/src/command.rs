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
use std::path::PathBuf;
// --- substrate ---
use sc_cli::{Role, RunCmd, RuntimeVersion, SubstrateCli};
// --- hyperspace ---
use crate::{
	chain_spec,
	cli::{Cli, Subcommand},
	service,
};
use hyperspace_cli::{Configuration, HyperspaceCli};

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Hyperspace Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/mvs/Hyperspace/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2018
	}

	fn executable_name() -> String {
		"hyperspace".into()
	}

	fn native_runtime_version(_spec: &Box<dyn sc_service::ChainSpec>) -> &'static RuntimeVersion {
		&service::hyperspace_runtime::VERSION
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		let id = if id == "" {
			let n = get_exec_name().unwrap_or_default();
			["hyperspace"]
				.iter()
				.cloned()
				.find(|&chain| n.starts_with(chain))
				.unwrap_or("hyperspace")
		} else {
			id
		};

		Ok(match id {
			"hyperspace" => Box::new(chain_spec::hyperspace_config()?),
			"hyperspace-dev" | "dev" => Box::new(chain_spec::hyperspace_development_config()),
			"hyperspace-genesis" => Box::new(chain_spec::hyperspace_build_spec_config()),
			path => Box::new(chain_spec::HyperspaceChainSpec::from_json_file(
				PathBuf::from(path),
			)?),
		})
	}
}
impl HyperspaceCli for Cli {
	fn conf(&self) -> &Option<PathBuf> {
		&self.conf
	}

	fn base(&self) -> &RunCmd {
		&self.run.base
	}

	fn mut_base(&mut self) -> &mut RunCmd {
		&mut self.run.base
	}
}

fn get_exec_name() -> Option<String> {
	std::env::current_exe()
		.ok()
		.and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
		.and_then(|s| s.into_string().ok())
}

/// Parse command line arguments into service configuration.
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		None => {
			let authority_discovery_disabled = cli.run.authority_discovery_disabled;
			let runner = Configuration::create_runner(cli)?;

			runner.run_node_until_exit(|config| async move {
				match config.role {
					Role::Light => {
						service::hyperspace_new_light(config).map(|(task_manager, _, _)| task_manager)
					}
					_ => service::hyperspace_new_full(config, authority_discovery_disabled)
						.map(|(task_manager, _, _)| task_manager),
				}
				.map_err(sc_cli::Error::Service)
			})
		}
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops::<
					service::hyperspace_runtime::RuntimeApi,
					service::HyperspaceExecutor,
				>(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops::<
					service::hyperspace_runtime::RuntimeApi,
					service::HyperspaceExecutor,
				>(&mut config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops::<
					service::hyperspace_runtime::RuntimeApi,
					service::HyperspaceExecutor,
				>(&mut config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops::<
					service::hyperspace_runtime::RuntimeApi,
					service::HyperspaceExecutor,
				>(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		}
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.async_run(|mut config| {
				let (client, backend, _, task_manager) = service::new_chain_ops::<
					service::hyperspace_runtime::RuntimeApi,
					service::HyperspaceExecutor,
				>(&mut config)?;
				Ok((cmd.run(client, backend), task_manager))
			})
		}
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::Sign(cmd)) => cmd.run(),
		Some(Subcommand::Verify(cmd)) => cmd.run(),
		Some(Subcommand::Vanity(cmd)) => cmd.run(),
	}
}
