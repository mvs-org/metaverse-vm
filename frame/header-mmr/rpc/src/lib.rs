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

//! Node-specific RPC methods for interaction with header-mmr.

// --- hyperspace ---
pub use hyperspace_header_mmr_rpc_runtime_api::HeaderMMRApi as HeaderMMRRuntimeApi;

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
use hyperspace_header_mmr_rpc_runtime_api::RuntimeDispatchInfo;

const RUNTIME_ERROR: i64 = -1;

#[rpc]
pub trait HeaderMMRApi<Hash, Response> {
	#[rpc(name = "headerMMR_genProof")]
	fn gen_proof(
		&self,
		block_number_of_member_leaf: u64,
		block_number_of_last_leaf: u64,
	) -> Result<Response>;
}

pub struct HeaderMMR<Client, Block> {
	client: Arc<Client>,
	_marker: std::marker::PhantomData<Block>,
}

impl<Client, Block> HeaderMMR<Client, Block> {
	pub fn new(client: Arc<Client>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

impl<Client, Block, Hash> HeaderMMRApi<Hash, RuntimeDispatchInfo<Hash>> for HeaderMMR<Client, Block>
where
	Client: 'static + Send + Sync + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	Client::Api: HeaderMMRRuntimeApi<Block, Hash>,
	Block: BlockT,
	Hash: core::fmt::Debug + Codec + MaybeDisplay + MaybeFromStr,
{
	fn gen_proof(
		&self,
		block_number_of_member_leaf: u64,
		block_number_of_last_leaf: u64,
	) -> Result<RuntimeDispatchInfo<Hash>> {
		let api = self.client.runtime_api();
		let best = self.client.info().best_hash;
		let at = BlockId::hash(best);

		api.gen_proof(&at, block_number_of_member_leaf, block_number_of_last_leaf)
			.map_err(|e| Error {
				code: ErrorCode::ServerError(RUNTIME_ERROR),
				message: "Unable to query power.".into(),
				data: Some(format!("{:?}", e).into()),
			})
	}
}
