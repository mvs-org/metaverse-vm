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
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Hyperspace. If not, see <https://www.gnu.org/licenses/>.

//! Prototype module for cross chain assets issuing.

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

pub use ethabi::{Event, Log};

// --- alloc ---
use alloc::vec::Vec;
// --- crates ---
use ethereum_types::{Address as EthereumAddress, H160, H256, U256};
// --- github ---
use ethabi::{
	param_type::ParamType, token::Token, Bytes, Error, EventParam, Function, Param, RawLog,
	Result as AbiResult,
};

pub struct Abi;

impl Abi {
	fn cross_receive() -> Function {
		let inputs = vec![
			Param {
				name: "token".into(),
				kind: ParamType::Address,
			},
			Param {
				name: "recipient".into(),
				kind: ParamType::Address,
			},
			Param {
				name: "amount".into(),
				kind: ParamType::Uint(256),
			},
		];

		Function {
			name: "crossReceive".into(),
			inputs,
			outputs: vec![],
			constant: false,
		}
	}

	fn create_erc20() -> Function {
		let inputs = vec![
			Param {
				name: "name".into(),
				kind: ParamType::String,
			},
			Param {
				name: "symbol".into(),
				kind: ParamType::String,
			},
			Param {
				name: "decimals".into(),
				kind: ParamType::Uint(8),
			},
			Param {
				name: "backing".into(),
				kind: ParamType::Address,
			},
			Param {
				name: "source".into(),
				kind: ParamType::Address,
			},
		];

		let outputs = vec![Param {
			name: "token".into(),
			kind: ParamType::Address,
		}];

		Function {
			name: "createERC20Contract".into(),
			inputs,
			outputs,
			constant: false,
		}
	}

	/// this New Register Event comes from the outer chains
	/// @params token: source erc20 token address
	/// @params name:  source erc20 token name
	/// @params symbol: source erc20 token symbol, which will added by m to generate mapped token
	/// @params decimals: source erc20 token decimals
	/// @params fee: register fee from the outer chain to hyperspace
	pub fn register_event() -> Event {
		Event {
			name: "NewTokenRegistered".into(),
			inputs: vec![
				EventParam {
					name: "token".into(),
					kind: ParamType::Address,
					indexed: true,
				},
				EventParam {
					name: "name".into(),
					kind: ParamType::String,
					indexed: false,
				},
				EventParam {
					name: "symbol".into(),
					kind: ParamType::String,
					indexed: false,
				},
				EventParam {
					name: "decimals".into(),
					kind: ParamType::Uint(8),
					indexed: false,
				},
				EventParam {
					name: "fee".into(),
					kind: ParamType::Uint(256),
					indexed: false,
				},
			],
			anonymous: false,
		}
	}

	/// this Token Lock Event comes from the outer chains
	/// @params token: source erc20 token address
	/// @params target:  mapped erc20 token address
	/// @params amount: transfer amount of the token
	/// @params recipient: the receiver on hyperspace of the asset
	/// @params fee: transfer fee from the outer chain to hyperspace
	pub fn backing_event() -> Event {
		Event {
			name: "BackingLock".into(),
			inputs: vec![
				EventParam {
					name: "token".into(),
					kind: ParamType::Address,
					indexed: true,
				},
				EventParam {
					name: "target".into(),
					kind: ParamType::Address,
					indexed: false,
				},
				EventParam {
					name: "amount".into(),
					kind: ParamType::Uint(256),
					indexed: false,
				},
				EventParam {
					name: "recipient".into(),
					kind: ParamType::Address,
					indexed: false,
				},
				EventParam {
					name: "fee".into(),
					kind: ParamType::Uint(256),
					indexed: false,
				},
			],
			anonymous: false,
		}
	}

	/// encode mint function for erc20
	pub fn encode_cross_receive(
		token: EthereumAddress,
		recipient: EthereumAddress,
		amount: U256,
	) -> AbiResult<Bytes> {
		let receive = Self::cross_receive();
		receive.encode_input(
			vec![
				Token::Address(token.into()),
				Token::Address(recipient.into()),
				Token::Uint(amount.into()),
			]
			.as_slice(),
		)
	}

	/// encode erc20 create function
	pub fn encode_create_erc20(
		name: &str,
		symbol: &str,
		decimals: u8,
		backing: EthereumAddress,
		source: EthereumAddress,
	) -> AbiResult<Bytes> {
		let create = Self::create_erc20();
		create.encode_input(
			vec![
				Token::String(name.into()),
				Token::String(symbol.into()),
				Token::Uint(U256::from(decimals)),
				Token::Address(backing.into()),
				Token::Address(source.into()),
			]
			.as_slice(),
		)
	}

	/// parse token register event
	pub fn parse_event(topics: Vec<H256>, data: Vec<u8>, eth_event: Event) -> AbiResult<Log> {
		//let eth_event = Self::register_event();
		let log = RawLog {
			topics: topics.into_iter().map(|t| -> H256 { t.into() }).collect(),
			data: data.clone(),
		};
		eth_event.parse_log(log)
	}

	/// get mapped token from source
	pub fn mapping_token() -> Function {
		let inputs = vec![
			Param {
				name: "backing".into(),
				kind: ParamType::Address,
			},
			Param {
				name: "source".into(),
				kind: ParamType::Address,
			},
		];

		let outputs = vec![Param {
			name: "target".into(),
			kind: ParamType::Address,
		}];

		Function {
			name: "mappingToken".into(),
			inputs,
			outputs,
			constant: true,
		}
	}

	/// encode get mapping token info function
	pub fn encode_mapping_token(
		backing: EthereumAddress,
		source: EthereumAddress,
	) -> AbiResult<Bytes> {
		let mapping = Self::mapping_token();
		mapping.encode_input(
			vec![
				Token::Address(backing.into()),
				Token::Address(source.into()),
			]
			.as_slice(),
		)
	}
}

/// token register info
/// this is the response from hyperspace after token registered
/// and would be sent to the outer chain
#[derive(Debug, PartialEq, Eq)]
pub struct TokenRegisterInfo(pub H160, pub H160, pub H160);

impl TokenRegisterInfo {
	pub fn decode(data: &[u8]) -> AbiResult<Self> {
		let tokens = ethabi::decode(
			&[ParamType::Address, ParamType::Address, ParamType::Address],
			&data,
		)?;
		match (tokens[0].clone(), tokens[1].clone(), tokens[2].clone()) {
			(Token::Address(backing), Token::Address(source), Token::Address(target)) => {
				Ok(TokenRegisterInfo(backing, source, target))
			}
			_ => Err(Error::InvalidData),
		}
	}
}

/// token burn info
/// this is the event proof from hyperspace after some user burn their mapped token
/// and would be sent to the outer chain to unlock the source token
/// @backing: the backing address on the source chain
/// @source: the source token address
/// @recipient: the final receiver of the token to be unlocked on the source chain
/// @amount: the amount of the burned token
#[derive(Debug, PartialEq, Eq)]
pub struct TokenBurnInfo {
	pub backing: H160,
	pub sender: H160,
	pub source: H160,
	pub recipient: H160,
	pub amount: U256,
}

impl TokenBurnInfo {
	pub fn decode(data: &[u8]) -> AbiResult<Self> {
		let tokens = ethabi::decode(
			&[
				ParamType::Address,
				ParamType::Address,
				ParamType::Address,
				ParamType::Address,
				ParamType::Uint(256),
			],
			&data,
		)?;
		match (
			tokens[0].clone(),
			tokens[1].clone(),
			tokens[2].clone(),
			tokens[3].clone(),
			tokens[4].clone(),
		) {
			(
				Token::Address(backing),
				Token::Address(sender),
				Token::Address(source),
				Token::Address(recipient),
				Token::Uint(amount),
			) => Ok(TokenBurnInfo {
				backing,
				sender,
				source,
				recipient,
				amount,
			}),
			_ => Err(Error::InvalidData),
		}
	}
}
