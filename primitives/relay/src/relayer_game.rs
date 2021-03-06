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

//! Relayer Game Primitives

// --- core ---
use core::fmt::Debug;
// --- crates ---
use codec::{Decode, Encode, FullCodec};
// --- substrate ---
use sp_runtime::{traits::Zero, DispatchError, DispatchResult, RuntimeDebug};
use sp_std::prelude::*;

pub trait RelayHeaderParcelInfo {
	type HeaderId: Clone;

	fn header_id(&self) -> Self::HeaderId;
}

/// Implement this for target chain's relay module's
/// to expose some necessary APIs for relayer game
pub trait Relayable {
	/// The Id which point to a unique header, for ethereum it's block number
	type RelayHeaderId: Clone + Debug + Default + PartialOrd + FullCodec;
	type RelayHeaderParcel: Clone
		+ Debug
		+ PartialEq
		+ FullCodec
		+ RelayHeaderParcelInfo<HeaderId = Self::RelayHeaderId>;
	type RelayProofs;

	/// The latest finalize block's id which recorded in hyperspace
	fn best_confirmed_relay_header_id() -> Self::RelayHeaderId;

	/// Some custom preverify logic for different chain
	fn preverify_game_sample_points(
		extended_relay_affirmation_id: &RelayAffirmationId<Self::RelayHeaderId>,
		game_sample_points: &[Self::RelayHeaderParcel],
	) -> DispatchResult;

	// TODO: optimize this
	fn verify_relay_proofs(
		// This Id is use for getting the mmr root's block's number
		// For ethereum
		// 	header id = block number
		// 	last leaf = block number - 1
		relay_header_id: &Self::RelayHeaderId,
		relay_header_parcel: &Self::RelayHeaderParcel,
		relay_proofs: &Self::RelayProofs,
		optional_best_confirmed_relay_header_id: Option<&Self::RelayHeaderId>,
	) -> DispatchResult;

	fn verify_relay_chain(relay_chain: Vec<&Self::RelayHeaderParcel>) -> DispatchResult;

	fn distance_between(
		relay_header_id: &Self::RelayHeaderId,
		best_confirmed_relay_header_id: Self::RelayHeaderId,
	) -> u32;

	/// Trying to confirm a relay header parcel
	///
	/// If there's a guard then it goes pended else confirmed
	fn try_confirm_relay_header_parcel(
		relay_header_parcel: Self::RelayHeaderParcel,
	) -> DispatchResult;

	fn try_confirm_relay_header_parcels(
		relay_header_parcels: Vec<Self::RelayHeaderParcel>,
	) -> Vec<Result<(), DispatchError>> {
		relay_header_parcels
			.into_iter()
			.map(Self::try_confirm_relay_header_parcel)
			.collect::<Vec<Result<(), DispatchError>>>()
	}

	fn new_round(game_id: &Self::RelayHeaderId, game_sample_points: Vec<Self::RelayHeaderId>);

	fn game_over(game_id: &Self::RelayHeaderId);
}

/// A regulator to adjust relay args for a specific chain
/// Implement this in runtime's `impls.rs`
pub trait AdjustableRelayerGame {
	type Moment;
	type Balance;
	type RelayHeaderId;

	/// The maximum number of active games
	///
	/// This might relate to the validators count
	fn max_active_games() -> u8;

	fn affirm_time(round: u32) -> Self::Moment;

	fn complete_proofs_time(round: u32) -> Self::Moment;

	/// Update the game's sample points
	///
	/// Push the new samples to the `sample_points`, the index of `sample_points` aka round index
	/// And return the new samples
	fn update_sample_points(sample_points: &mut Vec<Vec<Self::RelayHeaderId>>);

	/// Give an estimate stake value for a specify round
	///
	/// Usally the stake value go expensive wihle the round and the affirmations count increase
	fn estimate_stake(round: u32, affirmations_count: u32) -> Self::Balance;
}

pub trait RelayerGameProtocol {
	type Relayer;
	type RelayHeaderId: Clone + PartialOrd;
	type RelayHeaderParcel: Clone
		+ Debug
		+ PartialEq
		+ FullCodec
		+ RelayHeaderParcelInfo<HeaderId = Self::RelayHeaderId>;
	type RelayProofs;

	fn get_proposed_relay_header_parcels(
		affirmation_id: &RelayAffirmationId<Self::RelayHeaderId>,
	) -> Option<Vec<Self::RelayHeaderParcel>>;

	/// The best confirmed header id record of a game when it start
	fn best_confirmed_header_id_of(game_id: &Self::RelayHeaderId) -> Self::RelayHeaderId;

	/// Arrirm a new affirmation
	///
	/// Game's entry point, call only at the first round
	fn affirm(
		relayer: &Self::Relayer,
		relay_header_parcel: Self::RelayHeaderParcel,
		optional_relay_proofs: Option<Self::RelayProofs>,
	) -> Result<Self::RelayHeaderId, DispatchError>;

	/// Dispute Found
	///
	/// Arrirm a new affirmation to against the existed affirmation(s)
	fn dispute_and_affirm(
		relayer: &Self::Relayer,
		relay_header_parcel: Self::RelayHeaderParcel,
		optional_relay_proofs: Option<Self::RelayProofs>,
	) -> Result<(Self::RelayHeaderId, u32), DispatchError>;

	/// Verify a specify affirmation
	///
	/// Proofs is a `Vec` because the sampling function might give more than 1 sample points,
	/// so need to verify each sample point with its proofs
	fn complete_relay_proofs(
		affirmation_id: RelayAffirmationId<Self::RelayHeaderId>,
		relay_proofs: Vec<Self::RelayProofs>,
	) -> DispatchResult;

	/// Once there're different opinions in a game,
	/// chain will ask relayer to submit more samples
	/// to help the chain make a on chain arbitrate finally
	fn extend_affirmation(
		relayer: &Self::Relayer,
		extended_relay_affirmation_id: RelayAffirmationId<Self::RelayHeaderId>,
		game_sample_points: Vec<Self::RelayHeaderParcel>,
		optional_relay_proofs: Option<Vec<Self::RelayProofs>>,
	) -> Result<(Self::RelayHeaderId, u32, u32), DispatchError>;
}

/// Game id, round and the index under the round point to a unique affirmation AKA affirmation id
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct RelayAffirmationId<RelayHeaderId> {
	/// Game id aka relay header id
	pub game_id: RelayHeaderId,
	/// Round index
	pub round: u32,
	/// Index of a affirmation list which under a round
	pub index: u32,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct RelayAffirmation<RelayHeaderParcel, Relayer, Balance, RelayHeaderId> {
	pub relayer: Relayer,
	pub relay_header_parcels: Vec<RelayHeaderParcel>,
	pub stake: Balance,
	pub maybe_extended_relay_affirmation_id: Option<RelayAffirmationId<RelayHeaderId>>,
	pub verified_on_chain: bool,
}
impl<RelayHeaderParcel, Relayer, Balance, RelayHeaderId>
	RelayAffirmation<RelayHeaderParcel, Relayer, Balance, RelayHeaderId>
where
	Relayer: Default,
	Balance: Zero,
{
	pub fn new() -> Self {
		Self {
			relayer: Relayer::default(),
			relay_header_parcels: vec![],
			stake: Zero::zero(),
			maybe_extended_relay_affirmation_id: None,
			verified_on_chain: false,
		}
	}
}

/// Info for keeping track of a proposal being voted on.
#[derive(Default, Encode, Decode, RuntimeDebug)]
pub struct RelayVotingState<TechnicalMember> {
	/// The current set of technical members that approved it.
	pub ayes: Vec<TechnicalMember>,
	/// The current set of technical members that rejected it.
	pub nays: Vec<TechnicalMember>,
}
