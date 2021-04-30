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

#![cfg_attr(not(feature = "std"), no_std)]

mod util;

use codec::Decode;
use core::str::FromStr;
use ethabi::{Function, Param, ParamType, Token};
use evm::{Context, ExitError, ExitSucceed};
use frame_support::{
	ensure,
	traits::{Currency, ExistenceRequirement},
};
use sha3::Digest;
use sp_core::{H160, U256};
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::borrow::ToOwned;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;
use sp_std::vec::Vec;

use hyperspace_evm::{AddressMapping, Config, Runner};
use hyperspace_evm_primitives::Precompile;

type AccountId<T> = <T as frame_system::Config>::AccountId;

const TRANSFER_AND_CALL_ACTION: &[u8] = b"transfer_and_call(address,address,uint256)";
const WITHDRAW_ACTION: &[u8] = b"withdraw(bytes32,uint256)";
const DNA_PRECOMPILE: &str = "0000000000000000000000000000000000000016";
/// Dna Precompile Contract is used to support the exchange of DNA native asset between hyperspace and dvm contract
///
/// The contract address: 0000000000000000000000000000000000000016
pub struct Dna<T: Config> {
	_maker: PhantomData<T>,
}

impl<T: Config> Precompile for Dna<T> {
	/// There are two actions, one is `transfer_and_call` and the other is `withdraw`
	/// 1. Transfer_and_call Action, triggered by the user sending a transaction to the dna precompile
	/// 	   special evm address, eg(0000000000000000000000000000000000000016). and transfer the sender's
	///     dna balance to the deployed wdna contract in dvm. The input contain two parts:
	///     - p1: The wdna address, it is important to note that if this address is wrong, the balance cannot be recovered.
	///     - p2: The transfer value
	/// 2. WithDraw Action, the user sends transaction to wdna contract triggering this dna precompile to be called
	///     within the wdna contract, and transfer the balance from wdna balanceof to the hyperspace network. The input contain two parts:
	///     - p1: The to account id, a withdraw hyperspace public key.
	///     - p2: The withdraw value
	fn execute(
		input: &[u8],
		target_limit: Option<u64>,
		context: &Context,
	) -> core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
		let helper = U256::from(10)
			.checked_pow(U256::from(9))
			.unwrap_or(U256::MAX);
		let action = which_action::<T>(&input)?;
		let con_caller = T::AddressMapping::into_account_id(context.caller);
		match action {
			Action::TransferAndCall(tacd) => {
				// 1. Transfer dna from sender to dna erc20 contract
				let wdna_account_id = T::AddressMapping::into_account_id(tacd.wdna_address);
				let transfer_value = tacd.value.saturating_mul(helper).low_u128();
				ensure!(
					T::DnaCurrency::free_balance(&con_caller)
						>= transfer_value.unique_saturated_into(),
					ExitError::OutOfFund
				);
				T::DnaCurrency::transfer(
					&con_caller,
					&wdna_account_id,
					transfer_value.unique_saturated_into(),
					ExistenceRequirement::AllowDeath,
				)
				.map_err(|_| ExitError::Other("Transfer in Dna precompile failed".into()))?;

				// 2. Call wdna sol contract deposit
				let raw_input = make_call_data(context.caller, tacd.value)?;
				let precompile_address = H160::from_str(DNA_PRECOMPILE).unwrap();
				T::Runner::call(
					precompile_address,
					tacd.wdna_address,
					raw_input.to_vec(),
					U256::zero(),
					target_limit.unwrap_or_default(),
					None,
					None,
					T::config(),
				)
				.map_err(|_| ExitError::Other("Call in Dna precompile failed".into()))?;

				Ok((ExitSucceed::Returned, vec![], 20000))
			}
			Action::Withdraw(wd) => {
				let withdraw_value = wd.dna_value.saturating_mul(helper);
				T::DnaCurrency::transfer(
					&con_caller,
					&wd.to_account_id,
					withdraw_value.low_u128().unique_saturated_into(),
					ExistenceRequirement::AllowDeath,
				)
				.map_err(|_| ExitError::Other("Withdraw in Dna precompile failed".into()))?;
				Ok((ExitSucceed::Returned, vec![], 20000))
			}
		}
	}
}

/// Action about DNA precompile
pub enum Action<T: frame_system::Config> {
	/// Transfer from substrate account to wdna contract
	TransferAndCall(CallData),
	/// Withdraw from wdna contract to substrate account
	Withdraw(WithdrawData<T>),
}

/// which action depends on the function selector
pub fn which_action<T: frame_system::Config>(input_data: &[u8]) -> Result<Action<T>, ExitError> {
	let transfer_and_call_action = &sha3::Keccak256::digest(&TRANSFER_AND_CALL_ACTION)[0..4];
	let withdraw_action = &sha3::Keccak256::digest(&WITHDRAW_ACTION)[0..4];
	if &input_data[0..4] == transfer_and_call_action {
		let decoded_data = CallData::decode(&input_data[4..])?;
		return Ok(Action::TransferAndCall(decoded_data));
	} else if &input_data[0..4] == withdraw_action {
		let decoded_data = WithdrawData::decode(&input_data[4..])?;
		return Ok(Action::Withdraw(decoded_data));
	}
	Err(ExitError::Other("Invalid Action！".into()))
}

#[derive(Debug, PartialEq, Eq)]
pub struct CallData {
	wdna_address: H160,
	value: U256,
}

impl CallData {
	pub fn decode(data: &[u8]) -> Result<Self, ExitError> {
		let tokens = ethabi::decode(&[ParamType::Address, ParamType::Uint(256)], &data)
			.map_err(|_| ExitError::Other("ethabi decoded error".into()))?;
		match (tokens[0].clone(), tokens[1].clone()) {
			(Token::Address(eth_wdna_address), Token::Uint(eth_value)) => Ok(CallData {
				wdna_address: util::e2s_address(eth_wdna_address),
				value: util::e2s_u256(eth_value),
			}),
			_ => Err(ExitError::Other("Invlid call data".into())),
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct WithdrawData<T: frame_system::Config> {
	pub to_account_id: AccountId<T>,
	pub dna_value: U256,
}

impl<T: frame_system::Config> WithdrawData<T> {
	pub fn decode(data: &[u8]) -> Result<Self, ExitError> {
		let tokens = ethabi::decode(&[ParamType::FixedBytes(32), ParamType::Uint(256)], &data)
			.map_err(|_| ExitError::Other("ethabi decoded error".into()))?;
		match (tokens[0].clone(), tokens[1].clone()) {
			(Token::FixedBytes(address), Token::Uint(eth_value)) => Ok(WithdrawData {
				to_account_id: <T as frame_system::Config>::AccountId::decode(
					&mut address.as_ref(),
				)
				.map_err(|_| ExitError::Other("Invalid destination address".into()))?,
				dna_value: util::e2s_u256(eth_value),
			}),
			_ => Err(ExitError::Other("Invlid withdraw input data".into())),
		}
	}
}

fn make_call_data(
	sp_address: sp_core::H160,
	sp_value: sp_core::U256,
) -> Result<Vec<u8>, ExitError> {
	let eth_address = util::s2e_address(sp_address);
	let eth_value = util::s2e_u256(sp_value);
	let func = Function {
		name: "deposit".to_owned(),
		inputs: vec![
			Param {
				name: "address".to_owned(),
				kind: ParamType::Address,
			},
			Param {
				name: "value".to_owned(),
				kind: ParamType::Uint(256),
			},
		],
		outputs: vec![],
		constant: false,
	};
	func.encode_input(&[Token::Address(eth_address), Token::Uint(eth_value)])
		.map_err(|_| ExitError::Other("Make call data error happened".into()))
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;

	#[test]
	fn test_make_input() {
		let mock_address =
			sp_core::H160::from_str("Aa01a1bEF0557fa9625581a293F3AA7770192632").unwrap();
		let mock_value_1 = sp_core::U256::from(30);
		let expected_str = "0x47e7ef24000000000000000000000000aa01a1bef0557fa9625581a293f3aa7770192632000000000000000000000000000000000000000000000000000000000000001e";

		let encoded_str =
			array_bytes::bytes2hex("0x", make_call_data(mock_address, mock_value_1).unwrap());
		assert_eq!(encoded_str, expected_str);

		let mock_value_2 = sp_core::U256::from(25);
		let encoded_str =
			array_bytes::bytes2hex("0x", make_call_data(mock_address, mock_value_2).unwrap());
		assert_ne!(encoded_str, expected_str);
	}
}
