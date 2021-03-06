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

use frame_support::traits::{Currency, ExistenceRequirement};
use sp_core::U256;
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;
use sp_std::vec::Vec;

use codec::Decode;
use hyperspace_evm::{AddressMapping, Config};
use hyperspace_evm_primitives::Precompile;
use evm::{Context, ExitError, ExitSucceed};

type AccountId<T> = <T as frame_system::Config>::AccountId;

/// WithDraw Precompile Contract, used to withdraw balance from evm account to hyperspace account
///
/// The contract address: 0000000000000000000000000000000000000015
pub struct WithDraw<T: Config> {
	_maker: PhantomData<T>,
}

impl<T: Config> Precompile for WithDraw<T> {
	/// The Withdraw process is divided into two part:
	/// 1. parse the withdrawal address from the input parameter and get the contract address and value from the context
	/// 2. transfer from the contract address to withdrawal address
	///
	/// Input data: 32-bit substrate withdrawal public key
	fn execute(
		input: &[u8],
		_: Option<u64>,
		context: &Context,
	) -> core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
		// Decode input data
		let input = InputData::<T>::decode(&input)?;

		let helper = U256::from(10)
			.checked_pow(U256::from(9))
			.unwrap_or(U256::MAX);
		let contract_address = T::AddressMapping::into_account_id(context.address);
		let context_value = context.apparent_value.div_mod(helper).0;
		let context_value = context_value.low_u128().unique_saturated_into();

		let result = T::EtpCurrency::transfer(
			&contract_address,
			&input.dest,
			context_value,
			ExistenceRequirement::AllowDeath,
		);

		match result {
			Ok(()) => Ok((ExitSucceed::Returned, vec![], 10000)),
			Err(error) => match error {
				sp_runtime::DispatchError::BadOrigin => Err(ExitError::Other("BadOrigin".into())),
				sp_runtime::DispatchError::CannotLookup => {
					Err(ExitError::Other("CannotLookup".into()))
				}
				sp_runtime::DispatchError::Other(message) => Err(ExitError::Other(message.into())),
				sp_runtime::DispatchError::Module { message, .. } => {
					Err(ExitError::Other(message.unwrap_or("Module Error").into()))
				}
				_ => Err(ExitError::Other("Module Error".into())),
			},
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub struct InputData<T: frame_system::Config> {
	pub dest: AccountId<T>,
}

impl<T: frame_system::Config> InputData<T> {
	pub fn decode(data: &[u8]) -> Result<Self, ExitError> {
		if data.len() == 32 {
			let mut dest_bytes = [0u8; 32];
			dest_bytes.copy_from_slice(&data[0..32]);

			return Ok(InputData {
				dest: <T as frame_system::Config>::AccountId::decode(&mut dest_bytes.as_ref())
					.map_err(|_| ExitError::Other("Invalid destination address".into()))?,
			});
		}
		Err(ExitError::Other("Invalid input data length".into()))
	}
}
