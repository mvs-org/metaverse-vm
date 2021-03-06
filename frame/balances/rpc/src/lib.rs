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

//! Node-specific RPC methods for interaction with balances.

// --- hyperspace ---
pub use hyperspace_balances_rpc_runtime_api::BalancesApi as BalancesRuntimeApi;

// --- core ---
use core::fmt::Debug;
// --- std ---
use std::sync::Arc;
// --- crates ---
use codec::Codec;
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
// --- substrate ---
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, MaybeDisplay, MaybeFromStr},
};
// --- hyperspace ---
use hyperspace_balances_rpc_runtime_api::RuntimeDispatchInfo;

const RUNTIME_ERROR: i64 = -1;

#[rpc]
pub trait BalancesApi<AccountId, Response> {
	#[rpc(name = "balances_usableBalance")]
	fn usable_balance(&self, instance: u8, who: AccountId) -> Result<Response>;
}

pub struct Balances<Client, Block> {
	client: Arc<Client>,
	_marker: std::marker::PhantomData<Block>,
}

impl<Client, Block> Balances<Client, Block> {
	pub fn new(client: Arc<Client>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

impl<Client, Block, AccountId, Balance> BalancesApi<AccountId, RuntimeDispatchInfo<Balance>>
	for Balances<Client, Block>
where
	Client: 'static + Send + Sync + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	Client::Api: BalancesRuntimeApi<Block, AccountId, Balance>,
	Block: BlockT,
	AccountId: Codec,
	Balance: Debug + Codec + MaybeDisplay + MaybeFromStr,
{
	fn usable_balance(&self, instance: u8, who: AccountId) -> Result<RuntimeDispatchInfo<Balance>> {
		let api = self.client.runtime_api();
		let best = self.client.info().best_hash;
		let at = BlockId::hash(best);

		api.usable_balance(&at, instance, who).map_err(|e| Error {
			code: ErrorCode::ServerError(RUNTIME_ERROR),
			message: "Unable to query usable balance.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
