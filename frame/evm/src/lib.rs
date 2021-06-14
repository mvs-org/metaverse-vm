// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! EVM execution module for Substrate

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub mod runner;
mod tests;

pub use crate::runner::Runner;
// --- hyperspace ---
pub use dp_evm::{
	Account, CallInfo, CreateInfo, ExecutionInfo, LinearCostPrecompile, Log, Precompile,
	PrecompileSet, Vicinity,
};
// --- substrate ---
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResultWithPostInfo,
	traits::{Currency, Get},
	weights::{Pays, PostDispatchInfo, Weight},
};
use frame_system::RawOrigin;
use sp_core::{H160, H256, U256};
use sp_runtime::{
	traits::{BadOrigin, UniqueSaturatedInto},
	AccountId32, DispatchResult,
};
use sp_std::vec::Vec;
// --- std ---
#[cfg(feature = "std")]
use codec::{Decode, Encode};
use evm::{Config as EvmConfig, ExitError, ExitReason};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Config that outputs the current transaction gas price.
pub trait FeeCalculator {
	/// Return the minimal required gas price.
	fn min_gas_price() -> U256;
}

impl FeeCalculator for () {
	fn min_gas_price() -> U256 {
		U256::zero()
	}
}

pub trait EnsureAddressOrigin<OuterOrigin> {
	/// Success return type.
	type Success;

	/// Perform the origin check.
	fn ensure_address_origin(
		address: &H160,
		origin: OuterOrigin,
	) -> Result<Self::Success, BadOrigin> {
		Self::try_address_origin(address, origin).map_err(|_| BadOrigin)
	}

	/// Try with origin.
	fn try_address_origin(
		address: &H160,
		origin: OuterOrigin,
	) -> Result<Self::Success, OuterOrigin>;
}

/// Ensure that the address is truncated hash of the origin. Only works if the account id is
/// `AccountId32`.
pub struct EnsureAddressTruncated;

impl<OuterOrigin> EnsureAddressOrigin<OuterOrigin> for EnsureAddressTruncated
where
	OuterOrigin: Into<Result<RawOrigin<AccountId32>, OuterOrigin>> + From<RawOrigin<AccountId32>>,
{
	type Success = AccountId32;

	fn try_address_origin(address: &H160, origin: OuterOrigin) -> Result<AccountId32, OuterOrigin> {
		origin.into().and_then(|o| match o {
			RawOrigin::Signed(who) if AsRef::<[u8; 32]>::as_ref(&who)[0..20] == address[0..20] => {
				Ok(who)
			}
			r => Err(OuterOrigin::from(r)),
		})
	}
}

pub trait AddressMapping<A> {
	fn into_account_id(address: H160) -> A;
}

pub struct ConcatAddressMapping;

/// The ConcatAddressMapping used for transfer from evm 20-length to substrate 32-length address
/// The concat rule inclued three parts:
/// 1. AccountId Prefix: concat("dvm", "0x00000000000000"), length: 11 byetes
/// 2. EVM address: the original evm address, length: 20 bytes
/// 3. CheckSum:  byte_xor(AccountId Prefix + EVM address), length: 1 bytes
impl AddressMapping<AccountId32> for ConcatAddressMapping {
	fn into_account_id(address: H160) -> AccountId32 {
		let mut data = [0u8; 32];
		data[0..4].copy_from_slice(b"dvm:");
		data[11..31].copy_from_slice(&address[..]);
		let checksum: u8 = data[1..31].iter().fold(data[0], |sum, &byte| sum ^ byte);
		data[31] = checksum;
		AccountId32::from(data)
	}
}

pub trait AccountBasic {
	fn account_basic(address: &H160) -> Account;
	fn mutate_account_basic(address: &H160, new: Account);
	fn transfer(source: &H160, target: &H160, value: U256) -> Result<(), ExitError>;
}

/// A mapping function that converts Ethereum gas to Substrate weight
pub trait GasWeightMapping {
	fn gas_to_weight(gas: u64) -> Weight;
	fn weight_to_gas(weight: Weight) -> u64;
}
impl GasWeightMapping for () {
	fn gas_to_weight(gas: u64) -> Weight {
		gas as Weight
	}
	fn weight_to_gas(weight: Weight) -> u64 {
		weight
	}
}

/// A contract handle for ethereum issuing
pub trait IssuingHandler {
	fn handle(address: H160, caller: H160, input: &[u8]) -> DispatchResult;
}

static ISTANBUL_CONFIG: EvmConfig = EvmConfig::istanbul();

/// EVM module trait
pub trait Config: frame_system::Config + pallet_timestamp::Config {
	/// Calculator for current gas price.
	type FeeCalculator: FeeCalculator;
	/// Maps Ethereum gas to Substrate weight.
	type GasWeightMapping: GasWeightMapping;

	/// Allow the origin to call on behalf of given address.
	type CallOrigin: EnsureAddressOrigin<Self::Origin>;
	/// Allow the origin to withdraw on behalf of given address.
	type WithdrawOrigin: EnsureAddressOrigin<Self::Origin, Success = Self::AccountId>;

	/// Mapping from address to account id.
	type AddressMapping: AddressMapping<Self::AccountId>;
	/// Etp Currency type
	type EtpCurrency: Currency<Self::AccountId>;
	/// Dna Currency type
	type DnaCurrency: Currency<Self::AccountId>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	/// Precompiles associated with this EVM engine.
	type Precompiles: PrecompileSet;
	/// Chain ID of EVM.
	type ChainId: Get<u64>;
	/// The block gas limit. Can be a simple constant, or an adjustment algorithm in another pallet.
	type BlockGasLimit: Get<U256>;
	/// EVM execution runner.
	type Runner: Runner<Self>;
	/// The account basic mapping way
	type EtpAccountBasic: AccountBasic;
	type DnaAccountBasic: AccountBasic;
	/// Issuing contracts handler
	type IssuingHandler: IssuingHandler;

	/// EVM config used in the module.
	fn config() -> &'static EvmConfig {
		&ISTANBUL_CONFIG
	}
}

#[cfg(feature = "std")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, Serialize, Deserialize)]
/// Account definition used for genesis block construction.
pub struct GenesisAccount {
	/// Account nonce.
	pub nonce: U256,
	/// Account balance.
	pub balance: U256,
	/// Full account storage.
	pub storage: std::collections::BTreeMap<H256, H256>,
	/// Account code.
	pub code: Vec<u8>,
}

decl_storage! {
	trait Store for Module<T: Config> as EVM {
		pub AccountCodes get(fn account_codes): map hasher(blake2_128_concat) H160 => Vec<u8>;
		pub AccountStorages get(fn account_storages):
			double_map hasher(blake2_128_concat) H160, hasher(blake2_128_concat) H256 => H256;
	}

	add_extra_genesis {
		config(accounts): std::collections::BTreeMap<H160, GenesisAccount>;
		build(|config: &GenesisConfig| {
			for (address, account) in &config.accounts {
				T::EtpAccountBasic::mutate_account_basic(&address, Account {
					balance: account.balance,
					nonce: account.nonce,
				});
				T::DnaAccountBasic::mutate_account_basic(&address, Account {
					balance: account.balance,
					nonce: account.nonce,
				});
				AccountCodes::insert(address, &account.code);

				for (index, value) in &account.storage {
					AccountStorages::insert(address, index, value);
				}
			}
		});
	}
}

decl_event! {
	/// EVM events
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
	{
		/// Ethereum events from contracts.
		Log(Log),
		/// A contract has been created at given \[address\].
		Created(H160),
		/// A \[contract\] was attempted to be created, but the execution failed.
		CreatedFailed(H160),
		/// A \[contract\] has been executed successfully with states applied.
		Executed(H160),
		/// A \[contract\] has been executed with errors. States are reverted with only gas fees applied.
		ExecutedFailed(H160),
		/// A deposit has been made at a given address. \[sender, address, value\]
		BalanceDeposit(AccountId, H160, U256),
		/// A withdrawal has been made from a given address. \[sender, address, value\]
		BalanceWithdraw(AccountId, H160, U256),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Not enough balance to perform action
		BalanceLow,
		/// Calculating total fee overflowed
		FeeOverflow,
		/// Calculating total payment overflowed
		PaymentOverflow,
		/// Withdraw fee failed
		WithdrawFailed,
		/// Gas price is too low.
		GasPriceTooLow,
		/// Nonce is invalid
		InvalidNonce,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Issue an EVM call operation. This is similar to a message call transaction in Ethereum.
		#[weight = T::GasWeightMapping::gas_to_weight(*gas_limit)]
		fn call(
			origin,
			source: H160,
			target: H160,
			input: Vec<u8>,
			value: U256,
			gas_limit: u64,
			gas_price: U256,
			nonce: Option<U256>,
		) -> DispatchResultWithPostInfo {
			T::CallOrigin::ensure_address_origin(&source, origin)?;

			let info = T::Runner::call(
				source,
				target,
				input,
				value,
				gas_limit,
				Some(gas_price),
				nonce,
				T::config(),
			)?;

			match info.exit_reason {
				ExitReason::Succeed(_) => {
					Module::<T>::deposit_event(Event::<T>::Executed(target));
				},
				_ => {
					Module::<T>::deposit_event(Event::<T>::ExecutedFailed(target));
				},
			};

			Ok(PostDispatchInfo {
				actual_weight: Some(T::GasWeightMapping::gas_to_weight(info.used_gas.unique_saturated_into())),
				pays_fee: Pays::No,
			})
		}

		/// Issue an EVM create operation. This is similar to a contract creation transaction in
		/// Ethereum.
		#[weight = T::GasWeightMapping::gas_to_weight(*gas_limit)]
		fn create(
			origin,
			source: H160,
			init: Vec<u8>,
			value: U256,
			gas_limit: u64,
			gas_price: U256,
			nonce: Option<U256>,
		) -> DispatchResultWithPostInfo {
			T::CallOrigin::ensure_address_origin(&source, origin)?;

			let info = T::Runner::create(
				source,
				init,
				value,
				gas_limit,
				Some(gas_price),
				nonce,
				T::config(),
			)?;
			match info {
				CreateInfo {
					exit_reason: ExitReason::Succeed(_),
					value: create_address,
					..
				} => {
					Module::<T>::deposit_event(Event::<T>::Created(create_address));
				},
				CreateInfo {
					exit_reason: _,
					value: create_address,
					..
				} => {
					Module::<T>::deposit_event(Event::<T>::CreatedFailed(create_address));
				},
			}

			Ok(PostDispatchInfo {
				actual_weight: Some(T::GasWeightMapping::gas_to_weight(info.used_gas.unique_saturated_into())),
				pays_fee: Pays::No,
			})
		}

		/// Issue an EVM create2 operation.
		#[weight = T::GasWeightMapping::gas_to_weight(*gas_limit)]
		fn create2(
			origin,
			source: H160,
			init: Vec<u8>,
			salt: H256,
			value: U256,
			gas_limit: u64,
			gas_price: U256,
			nonce: Option<U256>,
		) -> DispatchResultWithPostInfo {
			T::CallOrigin::ensure_address_origin(&source, origin)?;

			let info = T::Runner::create2(
				source,
				init,
				salt,
				value,
				gas_limit,
				Some(gas_price),
				nonce,
				T::config(),
			)?;
			match info {
				CreateInfo {
					exit_reason: ExitReason::Succeed(_),
					value: create_address,
					..
				} => {
					Module::<T>::deposit_event(Event::<T>::Created(create_address));
				},
				CreateInfo {
					exit_reason: _,
					value: create_address,
					..
				} => {
					Module::<T>::deposit_event(Event::<T>::CreatedFailed(create_address));
				},
			}

			Ok(PostDispatchInfo {
				actual_weight: Some(T::GasWeightMapping::gas_to_weight(info.used_gas.unique_saturated_into())),
				pays_fee: Pays::No,
			})
		}
	}
}

impl<T: Config> Module<T> {
	fn remove_account(address: &H160) {
		if AccountCodes::contains_key(address) {
			let account_id = T::AddressMapping::into_account_id(*address);
			let _ = <frame_system::Pallet<T>>::dec_consumers(&account_id);
		}

		AccountCodes::remove(address);
		AccountStorages::remove_prefix(address);
	}

	/// Create an account.
	pub fn create_account(address: H160, code: Vec<u8>) {
		if code.is_empty() {
			return;
		}

		if !AccountCodes::contains_key(&address) {
			let account_id = T::AddressMapping::into_account_id(address);
			let _ = <frame_system::Pallet<T>>::inc_consumers(&account_id);
		}

		AccountCodes::insert(address, code);
	}

	/// Check whether an account is empty.
	pub fn is_account_empty(address: &H160) -> bool {
		let account = T::EtpAccountBasic::account_basic(address);
		let code_len = AccountCodes::decode_len(address).unwrap_or(0);

		account.nonce == U256::zero() && account.balance == U256::zero() && code_len == 0
	}

	pub fn is_contract_code_empty(address: &H160) -> bool {
		let code_len = AccountCodes::decode_len(address).unwrap_or(0);
		code_len == 0
	}

	/// Remove an account if its empty.
	pub fn remove_account_if_empty(address: &H160) {
		if Self::is_account_empty(address) {
			Self::remove_account(address);
		}
	}

	/// Withdraw fee.
	pub fn withdraw_fee(address: &H160, value: U256) {
		let account = T::EtpAccountBasic::account_basic(address);
		let new_account_balance = account.balance.saturating_sub(value);

		T::EtpAccountBasic::mutate_account_basic(
			&address,
			Account {
				nonce: account.nonce,
				balance: new_account_balance,
			},
		);
	}

	/// Deposit fee.
	pub fn deposit_fee(address: &H160, value: U256) {
		let account = T::EtpAccountBasic::account_basic(address);
		let new_account_balance = account.balance.saturating_add(value);

		T::EtpAccountBasic::mutate_account_basic(
			&address,
			Account {
				nonce: account.nonce,
				balance: new_account_balance,
			},
		);
	}
}
