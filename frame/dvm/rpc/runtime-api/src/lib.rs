// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Frontier.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate. If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use hyperspace_support::evm::INTERNAL_CALLER;
use ethereum::{Block as EthereumBlock, Log};
use ethereum_types::Bloom;
use sp_core::{H160, H256, U256};
use sp_std::vec::Vec;

#[derive(Eq, PartialEq, Clone, Encode, Decode, sp_runtime::RuntimeDebug)]
pub struct TransactionStatus {
	pub transaction_hash: H256,
	pub transaction_index: u32,
	pub from: H160,
	pub to: Option<H160>,
	pub contract_address: Option<H160>,
	pub logs: Vec<Log>,
	pub logs_bloom: Bloom,
}

impl Default for TransactionStatus {
	fn default() -> Self {
		TransactionStatus {
			transaction_hash: H256::default(),
			transaction_index: 0 as u32,
			from: H160::default(),
			to: None,
			contract_address: None,
			logs: Vec::new(),
			logs_bloom: Bloom::default(),
		}
	}
}

/// The dvm transaction used by inner pallets, such as ethereum-issuing.
pub struct DVMTransaction {
	/// source of the transaction
	pub source: H160,
	/// gas price wrapped by Option
	pub gas_price: Option<U256>,
	/// the transaction defined in ethereum lib
	pub tx: ethereum::Transaction,
}

impl DVMTransaction {
	/// the internal transaction usually used by pallets
	/// the source account is specified by INTERNAL_CALLER
	/// gas_price is None means no need for gas fee
	/// a default signature which will not be verified
	pub fn new(nonce: U256, target: H160, input: Vec<u8>) -> Self {
		let transaction = ethereum::Transaction {
			nonce,
			// Not used, and will be overwritten by None later.
			gas_price: U256::zero(),
			gas_limit: U256::from(0x100000),
			action: ethereum::TransactionAction::Call(target),
			value: U256::zero(),
			input,
			signature: ethereum::TransactionSignature::new(
				// Reference https://github.com/ethereum/EIPs/issues/155
				//
				// But this transaction is sent by hyperspace-issuing system from `0x0`
				// So ignore signature checking, simply set `chain_id` to `1`
				1 * 2 + 36,
				H256::from_slice(&[55u8; 32]),
				H256::from_slice(&[55u8; 32]),
			)
			.unwrap(),
		};
		Self {
			source: INTERNAL_CALLER,
			gas_price: None,
			tx: transaction,
		}
	}
}

sp_api::decl_runtime_apis! {
	/// API necessary for Ethereum-compatibility layer.
	pub trait EthereumRuntimeRPCApi {
		/// Returns runtime defined hyperspace_evm::ChainId.
		fn chain_id() -> u64;
		/// Returns hyperspace_evm::Accounts by address.
		fn account_basic(address: H160) -> dp_evm::Account;
		/// Returns FixedGasPrice::min_gas_price
		fn gas_price() -> U256;
		/// For a given account address, returns hyperspace_evm::AccountCodes.
		fn account_code_at(address: H160) -> Vec<u8>;
		/// Returns the converted FindAuthor::find_author authority id.
		fn author() -> H160;
		/// For a given account address and index, returns hyperspace_evm::AccountStorages.
		fn storage_at(address: H160, index: U256) -> H256;
		/// Returns a dvm_ethereum::call response.
		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			gas_price: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
		) -> Result<dp_evm::CallInfo, sp_runtime::DispatchError>;
		/// Returns a frame_ethereum::create response.
		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			gas_price: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
		) -> Result<dp_evm::CreateInfo, sp_runtime::DispatchError>;
		/// Return the current block.
		fn current_block() -> Option<EthereumBlock>;
		/// Return the current receipt.
		fn current_receipts() -> Option<Vec<ethereum::Receipt>>;
		/// Return the current transaction status.
		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>>;
		/// Return all the current data for a block in a single runtime call.
		fn current_all() -> (
			Option<EthereumBlock>,
			Option<Vec<ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		);
	}
}

pub trait ConvertTransaction<E> {
	fn convert_transaction(&self, transaction: ethereum::Transaction) -> E;
}
