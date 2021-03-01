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

//! Module to relay blocks from Ethereum Network
//!
//! In this module,
//! the offchain worker will keep fetch the next block info and relay to Metaverse.
//! The worker will fetch the header and the Merkle proof information for blocks from a nonexistent domain,
//! ie http://eth-resource/, such that it can be connected with shadow service.
//! Now the shadow service is provided by our another project, hyperspace.js.
//! https://github.com/hyperspace-network/hyperspace.js
//!
//!
//! Here is the basic flow.
//! The starting point is the `offchain_worker`
//! - base on block schedule, the `relay_header` will be called
//! - then the `relay_header` will get ethereum blocks and Merkle proof information from from http://eth-resource/
//! - After the http response corrected fetched, we will simple validate the format of http response,
//!   and parse and build Ethereum header and Merkle Proofs.
//! - After all, the corrected Ethereum header with the proofs will be submit and validate on chain of Metaverse by
//!   `submit_header`
//!
//! The protocol of shadow service and offchain worker can be scale encoded format or json format,
//! and the worker will use json format as fail back, such that it may be easier to debug.
//! If you want to build your own shadow service please refer
//! https://github.com/hyperspace-network/hyperspace-common/issues/86
//!
//! More details about offchain workers in following PRs
//! https://github.com/hyperspace-network/hyperspace/pull/335
//! https://github.com/hyperspace-network/hyperspace-common/pull/43
//! https://github.com/hyperspace-network/hyperspace-common/pull/63
#![cfg_attr(not(feature = "std"), no_std)]

pub mod crypto {
	// --- substrate ---
	use frame_system::offchain::AppCrypto;
	use sp_core::sr25519::{Public, Signature};
	use sp_runtime::{traits::Verify, MultiSignature};

	mod app {
		// --- substrate ---
		use sp_runtime::app_crypto::{app_crypto, sr25519};
		// --- hyperspace ---
		use crate::ETH_OFFCHAIN;

		app_crypto!(sr25519, ETH_OFFCHAIN);
	}

	pub struct AuthorityId;
	impl AppCrypto<<MultiSignature as Verify>::Signer, MultiSignature> for AuthorityId {
		type RuntimeAppPublic = app::Public;
		type GenericPublic = Public;
		type GenericSignature = Signature;
	}
}

#[cfg(all(feature = "std", test))]
mod mock;
#[cfg(all(feature = "std", test))]
mod tests;

// --- core ---
use core::str::from_utf8;
// --- crates ---
use codec::Decode;
// --- substrate ---
use frame_support::{debug::trace, decl_error, decl_module, traits::Get};
#[cfg(not(test))]
use frame_system::offchain::SendSignedTransaction;
use frame_system::offchain::{AppCrypto, CreateSignedTransaction, ForAll, Signer};

use sp_runtime::{traits::Zero, DispatchError, KeyTypeId};
use sp_std::prelude::*;
// --- hyperspace ---
use array_bytes::{base_n_bytes_unchecked, hex_bytes_unchecked};
use hyperspace_support::literal_procesor::extract_from_json_str;
use ethereum_primitives::{ethashproof::EthashProof, header::EthereumHeader};

type EthereumRelay<T> = hyperspace_ethereum_linear_relay::Module<T>;
type EthereumRelayCall<T> = hyperspace_ethereum_linear_relay::Call<T>;

pub const ETH_OFFCHAIN: KeyTypeId = KeyTypeId(*b"etho");

/// A dummy endpoint, point this to shadow service
const ETH_RESOURCE: &'static [u8] = b"http://shadow.mvs.org/";

pub trait Trait:
	CreateSignedTransaction<EthereumRelayCall<Self>> + hyperspace_ethereum_linear_relay::Trait
{
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

	type FetchInterval: Get<Self::BlockNumber>;
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// API Response - UNEXPECTED
		APIRespUnexp,

		/// Best Header - NOT EXISTED
		BestHeaderNE,
		/// Block Number - OVERFLOW
		BlockNumberOF,

		/// Proof - SCALE DECODE ERROR
		ProofSE,
		/// Proof - JSON DECODE ERROR
		ProofJE,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		type Error = Error<T>;

		const FetchInterval: T::BlockNumber = T::FetchInterval::get();

		/// The offchain worker which will be called in a regular block schedule
		/// The relay_header is called when the block meet the schedule timing
		fn offchain_worker(block: T::BlockNumber) {
			let fetch_interval = T::FetchInterval::get().max(1.into());
			if (block % fetch_interval).is_zero() {
				let signer = <Signer<T, T::AuthorityId>>::all_accounts();
				if signer.can_sign() {
					if let Err(e) = Self::relay_header(&signer){
						trace!(target: "ethereum-offchain", "[ethereum-offchain] Error: {:?}", e);
					}
				} else {
					trace!(target: "ethereum-offchain", "[ethereum-offchain] use `author_insertKey` rpc to inscert key to enable worker");
				}
			}
		}
	}
}

impl<T: Trait> Module<T> {
	/// The `relay_header` is try to get Ethereum blocks with merkle proofs from shadow service
	/// The default communication will transfer data with scale encoding,
	/// if there are issue to communicate with scale encoding, the failback communication will
	/// be performed with json format(use option: `true`)
	fn relay_header(signer: &Signer<T, T::AuthorityId, ForAll>) -> Result<(), DispatchError> {
		let target_number = Self::get_target_number()?;
		let header_without_option = Self::fetch_header(ETH_RESOURCE.to_vec(), target_number, false);
		let (header, proof_list) = match header_without_option {
			Ok(r) => r,
			Err(e) => {
				trace!(target: "ethereum-offchain", "[ethereum-offchain] request without option fail: {:?}", e);
				trace!(target: "ethereum-offchain", "[ethereum-offchain] request fail back wth option");
				Self::fetch_header(ETH_RESOURCE.to_vec(), target_number, true)?
			}
		};

		Self::submit_header(signer, header, proof_list);

		Ok(())
	}

	/// Get the last confirmed block number, and return the blocknumber of next one as target
	fn get_target_number() -> Result<u64, DispatchError> {
		let target_number = <EthereumRelay<T>>::header(<EthereumRelay<T>>::best_header_hash())
			.ok_or(<Error<T>>::BestHeaderNE)?
			.number
			.checked_add(1)
			.ok_or(<Error<T>>::BlockNumberOF)?;
		trace!(target: "ethereum-offchain", "[ethereum-offchain] Target Number: {}", target_number);

		Ok(target_number)
	}

	fn build_request_client(url: Vec<u8>, payload: Vec<u8>) -> impl OffchainRequestTrait {
		OffchainRequest::new(url, payload)
	}

	/// Build the response as EthereumHeader struct after validating
	fn fetch_header(
		url: Vec<u8>,
		target_number: u64,
		option: bool,
	) -> Result<(EthereumHeader, Vec<EthashProof>), DispatchError> {
		let payload = Self::build_payload(target_number, option);
		let mut client = Self::build_request_client(url, payload);
		let maybe_resp_body = client.send();

		let resp_body = Self::validate_response(maybe_resp_body, option)?;

		let eth_header_part =
			extract_from_json_str(&resp_body[..], b"eth_header" as &[u8]).unwrap_or_default();
		let header = if option {
			panic!("FIXME")
		// EthereumHeader::from_str_unchecked(from_utf8(eth_header_part).unwrap_or_default())
		} else {
			let scale_bytes = hex_bytes_unchecked(from_utf8(eth_header_part).unwrap_or_default());
			Decode::decode::<&[u8]>(&mut &scale_bytes[..]).unwrap_or_default()
		};

		let proof_part =
			extract_from_json_str(&resp_body[..], b"proof" as &[u8]).unwrap_or_default();
		let proof_list = if option {
			Self::parse_double_node_with_proof_list_from_json_str(proof_part)?
		} else {
			Self::parse_double_node_with_proof_list_from_scale_str(proof_part)?
		};
		trace!(target: "ethereum-offchain", "[ethereum-offchain] Eth Header: {:?}", header);

		Ok((header, proof_list))
	}

	/// Validate the response is a JSON with enough data not simple error message
	fn validate_response(
		maybe_resp_body: Option<Vec<u8>>,
		with_option: bool,
	) -> Result<Vec<u8>, DispatchError> {
		if let Some(resp_body) = maybe_resp_body {
			trace!(
				target: "ethereum-offchain",
				"[ethereum-offchain] Response: {}",
				from_utf8(&resp_body).unwrap_or_default(),
			);
			if resp_body[0] != 123u8
				|| (with_option && resp_body.len() < 1362)
				|| (!with_option && resp_body.len() < 1353)
			{
				trace!(target: "ethereum-offchain", "[ethereum-offchain] Malresponse");
				Err(<Error<T>>::APIRespUnexp)?;
			}
			Ok(resp_body)
		} else {
			trace!(target: "ethereum-offchain", "[ethereum-offchain] Lack Response");
			Err(<Error<T>>::APIRespUnexp)?
		}
	}

	/// Submit and record the valid header on Hyperspace network
	fn submit_header(
		signer: &Signer<T, T::AuthorityId, ForAll>,
		header: EthereumHeader,
		proof_list: Vec<EthashProof>,
	) {
		// TODO: test support call ethereum-linear-relay
		// https://github.com/hyperspace-network/hyperspace-common/issues/137
		let results = {
			#[cfg(test)]
			{
				let _ = signer;
				vec![(
					(),
					format!("header: {:?}, proof_list: {:?}", header, proof_list),
				)]
			}
			#[cfg(not(test))]
			{
				signer.send_signed_transaction(|_| {
					<EthereumRelayCall<T>>::relay_header(header.clone(), proof_list.clone())
				})
			}
		};

		for (_, result) in &results {
			trace!(
				target: "ethereum-offchain",
				"[ethereum-offchain] Relay: {:?}",
				result,
			);
		}
	}

	/// Build a payload to request the json response or scaled encoded response depence on option
	fn build_payload(target_number: u64, option: bool) -> Vec<u8> {
		let header_part: &[u8] = br#"{"jsonrpc":"2.0","method":"shadow_getEthHeaderWithProofByNumber","params":{"block_num":"#;
		let number_part: &[u8] = &base_n_bytes_unchecked(target_number, 10)[..];
		let transaction_part: &[u8] = br#","transaction":false"#;
		let option_part: &[u8] = br#","options":{"format":"json"}"#;
		let tail: &[u8] = br#"},"id":1}"#;

		if option {
			[
				header_part,
				number_part,
				transaction_part,
				option_part,
				tail,
			]
			.concat()
		} else {
			[header_part, number_part, transaction_part, tail].concat()
		}
	}

	/// Build the merkle proof information from json format response
	fn parse_double_node_with_proof_list_from_json_str(
		json_str: &[u8],
	) -> Result<Vec<EthashProof>, DispatchError> {
		let raw_str = match from_utf8(json_str) {
			Ok(r) => r,
			Err(_) => Err(<Error<T>>::ProofJE)?,
		};

		let mut proof_list: Vec<EthashProof> = Vec::new();
		// The proof list is 64 length, and we use 256 just in case.
		for p in raw_str.splitn(256, '}') {
			if p.len() > 0 {
				proof_list.push(EthashProof::from_str_unchecked(p));
			}
		}
		Ok(proof_list)
	}

	/// Build the merkle proof information from scale encoded response
	fn parse_double_node_with_proof_list_from_scale_str(
		scale_str: &[u8],
	) -> Result<Vec<EthashProof>, DispatchError> {
		if scale_str.len() < 2 {
			Err(<Error<T>>::ProofSE)?;
		};
		let proof_scale_bytes = hex_bytes_unchecked(from_utf8(scale_str).unwrap_or_default());
		Ok(Decode::decode::<&[u8]>(&mut &proof_scale_bytes[..]).unwrap_or_default())
	}
}

#[derive(Default)]
pub struct OffchainRequest {
	location: Vec<u8>,
	payload: Vec<u8>,
	redirect_times: u8,
	cookie: Option<Vec<u8>>,
}
impl OffchainRequest {
	fn new(url: Vec<u8>, payload: Vec<u8>) -> Self {
		OffchainRequest {
			location: url.clone(),
			payload,
			..Default::default()
		}
	}
}

pub trait OffchainRequestTrait {
	fn send(&mut self) -> Option<Vec<u8>>;
}
/// The OffchainRequest handle the request session
/// - set cookie if returns
/// - handle the redirect actions if happened
#[cfg(not(test))]
impl OffchainRequestTrait for OffchainRequest {
	fn send(&mut self) -> Option<Vec<u8>> {
		for _ in 0..=3 {
			let p = self.payload.clone();
			let request = sp_runtime::offchain::http::Request::post(
				from_utf8(&self.location).unwrap_or_default(),
				vec![&p[..]],
			)
			.add_header("Content-Type", "application/json");
			if let Ok(pending) = request.send() {
				if let Ok(mut resp) = pending.wait() {
					if resp.code == 200 {
						return Some(resp.body().collect::<Vec<_>>());
					} else if resp.code == 301 || resp.code == 302 {
						self.redirect_times += 1;
						trace!(
							target: "ethereum-offchain",
							"[ethereum-offchain] Redirect({}), Request Header: {:?}",
							self.redirect_times, resp.headers(),
						);

						let headers = resp.headers();
						if let Some(cookie) = headers.find("set-cookie") {
							self.cookie = Some(cookie.as_bytes().to_vec());
						}
						if let Some(location) = headers.find("location") {
							self.location = location.as_bytes().to_vec();
							trace!(
								target: "ethereum-offchain",
								"[ethereum-offchain] Redirect({}), Location: {:?}",
								self.redirect_times,
								self.location,
							);
						}
					} else {
						trace!(target: "ethereum-offchain", "[ethereum-offchain] Status Code: {}", resp.code);
						trace!(
							target: "ethereum-offchain",
							"[ethereum-offchain] Response: {}",
							from_utf8(&resp.body().collect::<Vec<_>>()).unwrap_or_default(),
						);

						return None;
					}
				}
			}
		}

		None
	}
}
