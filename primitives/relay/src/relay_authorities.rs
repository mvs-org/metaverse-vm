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

//! Relay Authorities Primitives

// --- core ---
use core::fmt::Debug;
// --- crates ---
use codec::{Decode, Encode, FullCodec};
// --- substrate ---
use sp_runtime::{DispatchResult, RuntimeDebug};
use sp_std::prelude::*;

pub type OpCode = [u8; 4];
pub type Term = u32;

pub trait Sign<BlockNumber> {
	type Signature: Clone + Debug + PartialEq + FullCodec;
	type Message: Clone + Debug + Default + PartialEq + FullCodec;
	type Signer: Clone + Debug + Default + Ord + PartialEq + FullCodec;

	fn hash(raw_message: impl AsRef<[u8]>) -> Self::Message;

	fn verify_signature(
		signature: &Self::Signature,
		message: &Self::Message,
		signer: &Self::Signer,
	) -> bool;
}

pub trait RelayAuthorityProtocol<BlockNumber> {
	type Signer;

	fn schedule_mmr_root(block_number: BlockNumber);

	fn check_authorities_change_to_sync(
		term: Term,
		authorities: Vec<Self::Signer>,
	) -> DispatchResult;

	fn sync_authorities_change() -> DispatchResult;
}

pub trait MMR<BlockNumber, Root> {
	fn get_root(block_number: BlockNumber) -> Option<Root>;
}
// Only for test
impl<BlockNumber, Root> MMR<BlockNumber, Root> for () {
	fn get_root(_: BlockNumber) -> Option<Root> {
		None
	}
}

// Avoid duplicate type
// Use `RelayAuthority` instead `Authority`
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct RelayAuthority<AccountId, Signer, EtpBalance, BlockNumber> {
	pub account_id: AccountId,
	pub signer: Signer,
	pub stake: EtpBalance,
	pub term: BlockNumber,
}
impl<AccountId, Signer, EtpBalance, BlockNumber> PartialEq
	for RelayAuthority<AccountId, Signer, EtpBalance, BlockNumber>
where
	AccountId: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.account_id == other.account_id
	}
}
impl<AccountId, Signer, EtpBalance, BlockNumber> PartialEq<AccountId>
	for RelayAuthority<AccountId, Signer, EtpBalance, BlockNumber>
where
	AccountId: PartialEq,
{
	fn eq(&self, account_id: &AccountId) -> bool {
		&self.account_id == account_id
	}
}

#[derive(Encode)]
pub struct _S<_1, _2, _3, _4>
where
	_1: Encode,
	_2: Encode,
	_3: Encode,
	_4: Encode,
{
	pub _1: _1,
	pub _2: _2,
	#[codec(compact)]
	pub _3: _3,
	pub _4: _4,
}

/// The scheduled change of authority set
#[derive(Clone, Default, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct ScheduledAuthoritiesChange<AccountId, Signer, EtpBalance, BlockNumber> {
	/// The new authorities after the change
	pub next_authorities: Vec<RelayAuthority<AccountId, Signer, EtpBalance, BlockNumber>>,
	/// The deadline of the previous authorities to sign for the next authorities
	pub deadline: BlockNumber,
}
