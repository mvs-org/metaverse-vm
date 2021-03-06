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

//! # Relayer Game Module

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;
// --- hyperspace ---
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod types {
	// --- hyperspace ---
	use crate::*;

	pub type AccountId<T> = <T as frame_system::Config>::AccountId;
	pub type BlockNumber<T> = <T as frame_system::Config>::BlockNumber;

	pub type EtpBalance<T, I> = <EtpCurrency<T, I> as Currency<AccountId<T>>>::Balance;
	pub type EtpNegativeImbalance<T, I> =
		<EtpCurrency<T, I> as Currency<AccountId<T>>>::NegativeImbalance;

	pub type RelayHeaderId<T, I> = <RelayableChainT<T, I> as Relayable>::RelayHeaderId;
	pub type RelayHeaderParcel<T, I> = <RelayableChainT<T, I> as Relayable>::RelayHeaderParcel;
	pub type RelayProofs<T, I> = <RelayableChainT<T, I> as Relayable>::RelayProofs;

	pub type RelayAffirmationT<T, I> = RelayAffirmation<
		RelayHeaderParcel<T, I>,
		AccountId<T>,
		EtpBalance<T, I>,
		RelayHeaderId<T, I>,
	>;

	type EtpCurrency<T, I> = <T as Config<I>>::EtpCurrency;

	type RelayableChainT<T, I> = <T as Config<I>>::RelayableChain;
}

// --- substrate ---
use frame_support::{
	debug::*,
	decl_error, decl_module, decl_storage, ensure,
	traits::{Currency, Get, OnUnbalanced},
};
use sp_runtime::{
	traits::{Saturating, Zero},
	DispatchError, DispatchResult,
};
#[cfg(not(feature = "std"))]
use sp_std::borrow::ToOwned;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
// --- hyperspace ---
use hyperspace_relay_primitives::relayer_game::*;
use hyperspace_support::balance::lock::*;
use types::*;

pub trait Config<I: Instance = DefaultInstance>: frame_system::Config {
	/// The currency use for stake
	type EtpCurrency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

	type LockId: Get<LockIdentifier>;

	/// Handler for the unbalanced *ETP* reduction when slashing a relayer.
	type EtpSlash: OnUnbalanced<EtpNegativeImbalance<Self, I>>;

	/// A regulator to adjust relay args for a specific chain
	type RelayerGameAdjustor: AdjustableRelayerGame<
		Balance = EtpBalance<Self, I>,
		Moment = Self::BlockNumber,
		RelayHeaderId = RelayHeaderId<Self, I>,
	>;

	/// A chain which implemented `Relayable` trait
	type RelayableChain: Relayable;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

decl_error! {
	pub enum Error for Module<T: Config<I>, I: Instance> {
		/// Relay Parcel - ALREADY RELAYED
		RelayParcelAR,
		/// Round - MISMATCHED
		RoundMis,
		/// Active Games - TOO MANY
		ActiveGamesTM,
		/// Existed Affirmation(s) Found - CONFLICT
		ExistedAffirmationsFoundC,
		/// Game at This Round - CLOSED
		GameAtThisRoundC,
		/// Relay Affirmation - DUPLICATED
		RelayAffirmationDup,
		/// Usable *ETP* for Stake - INSUFFICIENT
		StakeIns,
		/// Relay Proofs Quantity - INVALID
		RelayProofsQuantityInv,
		/// Relay Affirmation - NOT EXISTED
		RelayAffirmationNE,
		/// Extended Relay Affirmation - NOT EXISTED
		ExtendedRelayAffirmationNE,
		/// Previous Relay Proofs - INCOMPLETE
		PreviousRelayProofsInc,
		/// Pending Relay Parcel - NOT EXISTED
		PendingRelayParcelNE,
	}
}

decl_storage! {
	trait Store for Module<T: Config<I>, I: Instance = DefaultInstance> as HyperspaceRelayerGame {
		/// Active games' relay header parcel's ids
		pub RelayHeaderParcelToResolve
			get(fn relay_header_parcel_to_resolve)
			: Vec<RelayHeaderId<T, I>>;

		/// All the active games' affirmations here
		///
		/// The first key is game id, the second key is round index
		/// then you will get the affirmations under that round in that game
		pub Affirmations
			get(fn affirmations_of_game_at)
			: double_map
				hasher(identity) RelayHeaderId<T, I>,
				hasher(identity) u32
			=> Vec<RelayAffirmationT<T, I>>;

		/// The best confirmed header id record of a game when it start
		pub BestConfirmedHeaderId
			get(fn best_confirmed_header_id_of)
			: map hasher(identity) RelayHeaderId<T, I>
			=> RelayHeaderId<T, I>;

		/// The total rounds of a game
		///
		/// `total rounds - 1 = last round index`
		pub RoundCounts
			get(fn round_count_of)
			: map hasher(identity) RelayHeaderId<T, I>
			=> u32;

		/// All the closed games here
		///
		/// Game close at this moment, closed games won't accept any affirmation
		pub AffirmTime
			get(fn affirm_end_time_of)
			: map hasher(identity) RelayHeaderId<T, I>
			=> Option<(BlockNumber<T>, u32)>;

		/// All the closed rounds here
		///
		/// Record the closed rounds endpoint which use for settling or updating
		/// Settle or update a game will be scheduled which will start at this moment
		pub GamesToUpdate
			get(fn games_to_update_at)
			: map hasher(identity) BlockNumber<T>
			=> Vec<RelayHeaderId<T, I>>;

		/// All the stakes here
		pub Stakes
			get(fn stakes_of)
			: map hasher(blake2_128_concat) AccountId<T>
			=> EtpBalance<T, I>;

		pub GameSamplePoints
			get(fn game_sample_points)
			:map hasher(identity) RelayHeaderId<T, I>
			=> Vec<Vec<RelayHeaderId<T, I>>>;
	}
}

decl_module! {
	pub struct Module<T: Config<I>, I: Instance = DefaultInstance> for enum Call
	where
		origin: T::Origin
	{
		type Error = Error<T, I>;

		const LOCK_ID: LockIdentifier = T::LockId::get();

		const MAX_ACTIVE_GAMES: u8 = T::RelayerGameAdjustor::max_active_games();

		fn on_finalize(now: BlockNumber<T>) {
			let game_ids = <GamesToUpdate<T, I>>::take(now);

			if !game_ids.is_empty() {
				if let Err(e) = Self::update_games_at(game_ids, now) {
					error!(target: "relayer-game", "{:?}", e);
				}
			}

			// Return while no closed rounds found
		}
	}
}

impl<T: Config<I>, I: Instance> Module<T, I> {
	/// Check if time for proposing
	pub fn is_game_open_at(
		game_id: &RelayHeaderId<T, I>,
		moment: BlockNumber<T>,
		round: u32,
	) -> bool {
		if let Some((affirm_time, affirm_round)) = Self::affirm_end_time_of(game_id) {
			affirm_time > moment && affirm_round == round
		} else {
			false
		}
	}

	/// Check if others already make a same affirmation
	pub fn is_unique_affirmation(
		proposed_relay_header_parcels: &[RelayHeaderParcel<T, I>],
		existed_affirmations: &[RelayAffirmationT<T, I>],
	) -> bool {
		!existed_affirmations.iter().any(|existed_affirmation| {
			existed_affirmation.relay_header_parcels.as_slice() == proposed_relay_header_parcels
		})
	}

	/// Check if relayer can afford the stake
	///
	/// Make sure relayer have enough balance,
	/// this won't let the account's free balance drop below existential deposit
	pub fn ensure_can_stake(
		relayer: &AccountId<T>,
		round: u32,
		affirmations_count: u32,
	) -> Result<EtpBalance<T, I>, DispatchError> {
		let stake = T::RelayerGameAdjustor::estimate_stake(round, affirmations_count);

		ensure!(
			T::EtpCurrency::usable_balance(relayer) >= stake,
			<Error<T, I>>::StakeIns
		);

		Ok(stake)
	}

	pub fn update_timer_of_game_at(
		game_id: &RelayHeaderId<T, I>,
		round: u32,
		moment: BlockNumber<T>,
	) {
		let affirm_time = moment + T::RelayerGameAdjustor::affirm_time(round);
		let complete_proofs_time = T::RelayerGameAdjustor::complete_proofs_time(round);

		<AffirmTime<T, I>>::insert(game_id, (affirm_time, round));
		<GamesToUpdate<T, I>>::mutate(affirm_time + complete_proofs_time, |games_to_update| {
			games_to_update.push(game_id.to_owned());
		});
	}

	pub fn update_games_at(
		game_ids: Vec<RelayHeaderId<T, I>>,
		moment: BlockNumber<T>,
	) -> DispatchResult {
		trace!(target: "relayer-game", "Found Closed Rounds at `{:?}`", moment);
		trace!(target: "relayer-game", "---");

		// let call = Call::update_games_unsigned(game_ids).into();
		//
		// <SubmitTransaction<T, Call<T, I>>>::submit_unsigned_transaction(call)
		// .map_err(|_| "TODO".into())

		Self::update_games(game_ids)?;

		Ok(())
	}

	pub fn update_stakes_with<F>(relayer: &AccountId<T>, calc_stakes: F)
	where
		F: FnOnce(EtpBalance<T, I>) -> EtpBalance<T, I>,
	{
		let stakes = calc_stakes(Self::stakes_of(relayer));

		if stakes.is_zero() {
			T::EtpCurrency::remove_lock(T::LockId::get(), relayer);

			<Stakes<T, I>>::take(relayer);
		} else {
			T::EtpCurrency::set_lock(
				T::LockId::get(),
				relayer,
				LockFor::Common { amount: stakes },
				WithdrawReasons::all(),
			);

			<Stakes<T, I>>::insert(relayer, stakes);
		}
	}

	pub fn slash_on(relayer: &AccountId<T>, stake: EtpBalance<T, I>) {
		Self::update_stakes_with(relayer, |old_stakes| old_stakes.saturating_sub(stake));

		T::EtpSlash::on_unbalanced(T::EtpCurrency::slash(relayer, stake).0);
	}

	pub fn payout_honesties_and_slash_evils(
		honesties: BTreeMap<AccountId<T>, (EtpBalance<T, I>, EtpBalance<T, I>)>,
		evils: BTreeMap<AccountId<T>, EtpBalance<T, I>>,
	) {
		for (relayer, (unstakes, rewards)) in honesties {
			// Unlock stakes for honesty
			Self::update_stakes_with(&relayer, |stakes| stakes.saturating_sub(unstakes));

			// Reward honesty
			T::EtpCurrency::deposit_creating(&relayer, rewards);
		}

		for (relayer, slashs) in evils {
			// Unlock stakes for honesty
			Self::update_stakes_with(&relayer, |stakes| stakes.saturating_sub(slashs));

			// Punish evil
			T::EtpCurrency::slash(&relayer, slashs);
		}
	}

	pub fn settle_without_challenge(
		mut winning_relay_affirmation: RelayAffirmationT<T, I>,
	) -> Option<RelayHeaderParcel<T, I>> {
		Self::update_stakes_with(&winning_relay_affirmation.relayer, |stakes| {
			stakes.saturating_sub(winning_relay_affirmation.stake)
		});

		// TODO: reward on no challenge

		if winning_relay_affirmation.relay_header_parcels.len() == 1 {
			Some(
				winning_relay_affirmation
					.relay_header_parcels
					.pop()
					.unwrap(),
			)
		} else {
			// Should never enter this condition
			error!(target: "relayer-game", "   >  Relay Parcels Count - MISMATCHED");

			None
		}
	}

	pub fn settle_abandon(game_id: &RelayHeaderId<T, I>) {
		for relay_affirmations in <Affirmations<T, I>>::iter_prefix_values(&game_id) {
			for RelayAffirmation { relayer, stake, .. } in relay_affirmations {
				Self::slash_on(&relayer, stake);
			}
		}
	}

	pub fn settle_with_challenge(
		game_id: &RelayHeaderId<T, I>,
		relay_affirmation: RelayAffirmationT<T, I>,
	) -> Option<RelayHeaderParcel<T, I>> {
		let RelayAffirmation {
			relayer,
			stake,
			mut maybe_extended_relay_affirmation_id,
			..
		} = relay_affirmation;
		// BTreeMap<(relayer, unstake, reward)>
		let mut honesties = <BTreeMap<AccountId<T>, (EtpBalance<T, I>, EtpBalance<T, I>)>>::new();
		// BTreeMap<(relayer, slash)>
		let mut evils = <BTreeMap<AccountId<T>, EtpBalance<T, I>>>::new();

		// TODO: reward on no challenge
		honesties.insert(relayer, (stake, Zero::zero()));

		while let Some(RelayAffirmationId { round, index, .. }) =
			maybe_extended_relay_affirmation_id.take()
		{
			let relay_affirmations = Self::affirmations_of_game_at(&game_id, round);

			if let Some(RelayAffirmation {
				relayer: honesty,
				stake,
				maybe_extended_relay_affirmation_id: previous_maybe_extended_relay_affirmation_id,
				..
			}) = relay_affirmations.get(index as usize)
			{
				maybe_extended_relay_affirmation_id =
					previous_maybe_extended_relay_affirmation_id.to_owned();

				honesties
					.entry(honesty.to_owned())
					.and_modify(|(unstakes, _)| *unstakes = unstakes.saturating_add(*stake))
					.or_insert((*stake, Zero::zero()));

				for (index_, RelayAffirmation { relayer, stake, .. }) in
					relay_affirmations.iter().enumerate()
				{
					if index_ as u32 != index {
						honesties
							.entry(honesty.to_owned())
							.and_modify(|(_, rewards)| *rewards = rewards.saturating_add(*stake));
						evils
							.entry(relayer.to_owned())
							.and_modify(|slashs| *slashs = slashs.saturating_add(*stake))
							.or_insert(*stake);
					}
				}

				if previous_maybe_extended_relay_affirmation_id.is_none() {
					let mut relay_header_parcels = relay_affirmations
						.into_iter()
						.nth(index as usize)
						.unwrap()
						.relay_header_parcels;

					if relay_header_parcels.len() == 1 {
						Self::payout_honesties_and_slash_evils(honesties, evils);

						return Some(relay_header_parcels.pop().unwrap());
					} else {
						// Should never enter this condition
						error!(target: "relayer-game", "   >  Relay Parcels - MORE THAN ONE");

						return None;
					}
				}
			} else {
				// Should never enter this condition
				error!(target: "relayer-game", "   >  Extended Relay Proposal - NOT EXISTED");

				return None;
			}
		}

		// Should never enter this condition
		error!(target: "relayer-game", "   >  Extended Relay Proposal - NOT EXISTED");

		None
	}

	pub fn on_chain_arbitrate(game_id: &RelayHeaderId<T, I>) -> Option<RelayHeaderParcel<T, I>> {
		let relay_affirmations =
			<Affirmations<T, I>>::iter_prefix_values(&game_id).collect::<Vec<_>>();
		let mut last_round_winning_relay_chain_indexes = vec![];

		if let Some(last_round_relay_affirmations) = relay_affirmations.last() {
			let mut maybe_extended_relay_affirmation_id;

			for (
				index,
				RelayAffirmation {
					maybe_extended_relay_affirmation_id: current_maybe_extended_relay_affirmation_id,
					relay_header_parcels,
					..
				},
			) in last_round_relay_affirmations.iter().enumerate()
			{
				maybe_extended_relay_affirmation_id = current_maybe_extended_relay_affirmation_id;

				let mut relay_chain = vec![];

				for relay_header_parcel in relay_header_parcels.iter() {
					relay_chain.push(relay_header_parcel);
				}

				while let Some(RelayAffirmationId { round, index, .. }) =
					maybe_extended_relay_affirmation_id
				{
					if let Some(round_relay_affirmations) = relay_affirmations.get(*round as usize)
					{
						if let Some(RelayAffirmation {
							maybe_extended_relay_affirmation_id:
								previous_maybe_extended_relay_affirmation_id,
							relay_header_parcels,
							..
						}) = round_relay_affirmations.get(*index as usize)
						{
							maybe_extended_relay_affirmation_id =
								previous_maybe_extended_relay_affirmation_id;

							for relay_header_parcel in relay_header_parcels.iter() {
								relay_chain.push(relay_header_parcel);
							}
						} else {
							// Should never enter this condition
							error!(target: "relayer-game", "   >  Index - NOT EXISTED");

							return None;
						}
					} else {
						// Should never enter this condition
						error!(target: "relayer-game", "   >  Round - NOT EXISTED");

						return None;
					}
				}

				if T::RelayableChain::verify_relay_chain(relay_chain).is_ok() {
					last_round_winning_relay_chain_indexes.push(index);
				} else {
					trace!(
						target: "relayer-game",
						"   >  Relay Chain - INVALID",
					);
				}
			}

			match last_round_winning_relay_chain_indexes.len() {
				0 => None,
				1 => {
					let last_round_winning_relay_chain_index =
						last_round_winning_relay_chain_indexes.pop().unwrap();
					let RelayAffirmation {
						relayer: honesty,
						stake,
						maybe_extended_relay_affirmation_id,
						..
					} = &relay_affirmations.last().unwrap()[last_round_winning_relay_chain_index];
					let mut maybe_extended_relay_affirmation_id =
						maybe_extended_relay_affirmation_id.to_owned();
					// BTreeMap<(relayer, unstake, reward)>
					let mut honesties =
						<BTreeMap<AccountId<T>, (EtpBalance<T, I>, EtpBalance<T, I>)>>::new();
					// BTreeMap<(relayer, slash)>
					let mut evils = <BTreeMap<AccountId<T>, EtpBalance<T, I>>>::new();

					honesties
						.entry(honesty.to_owned())
						.and_modify(|(unstakes, _)| *unstakes = unstakes.saturating_add(*stake))
						.or_insert((*stake, Zero::zero()));

					for (index, RelayAffirmation { relayer, stake, .. }) in
						last_round_relay_affirmations.iter().enumerate()
					{
						if index != last_round_winning_relay_chain_index {
							honesties
								.entry(honesty.to_owned())
								.and_modify(|(_, rewards)| {
									*rewards = rewards.saturating_add(*stake)
								});
							evils
								.entry(relayer.to_owned())
								.and_modify(|slashs| *slashs = slashs.saturating_add(*stake))
								.or_insert(*stake);
						}
					}

					while let Some(RelayAffirmationId { round, index, .. }) =
						maybe_extended_relay_affirmation_id.take()
					{
						let round_relay_affirmations = &relay_affirmations[round as usize];

						if let Some(RelayAffirmation {
							relayer: honesty,
							stake,
							maybe_extended_relay_affirmation_id:
								previous_maybe_extended_relay_affirmation_id,
							..
						}) = round_relay_affirmations.get(index as usize)
						{
							maybe_extended_relay_affirmation_id =
								previous_maybe_extended_relay_affirmation_id.to_owned();

							honesties
								.entry(honesty.to_owned())
								.and_modify(|(unstakes, _)| {
									*unstakes = unstakes.saturating_add(*stake)
								})
								.or_insert((*stake, Zero::zero()));

							for (index_, RelayAffirmation { relayer, stake, .. }) in
								round_relay_affirmations.iter().enumerate()
							{
								if index_ as u32 != index {
									honesties.entry(honesty.to_owned()).and_modify(
										|(_, rewards)| *rewards = rewards.saturating_add(*stake),
									);
									evils
										.entry(relayer.to_owned())
										.and_modify(|slashs| {
											*slashs = slashs.saturating_add(*stake)
										})
										.or_insert(*stake);
								}
							}

							if previous_maybe_extended_relay_affirmation_id.is_none() {
								let relay_header_parcels = &round_relay_affirmations
									.into_iter()
									.nth(index as usize)
									.unwrap()
									.relay_header_parcels;

								if relay_header_parcels.len() == 1 {
									Self::payout_honesties_and_slash_evils(honesties, evils);

									return Some(relay_header_parcels[0].to_owned());
								} else {
									// Should never enter this condition
									error!(target: "relayer-game", "   >  Relay Parcels - MORE THAN ONE");

									return None;
								}
							}
						} else {
							// Should never enter this condition
							error!(target: "relayer-game", "   >  Extended Relay Proposal - NOT EXISTED");

							return None;
						}
					}

					// Should never enter this condition
					error!(target: "relayer-game", "   >  Extended Relay Proposal - NOT EXISTED");

					None
				}
				_ => {
					// Should never enter this condition
					error!(target: "relayer-game", "   >  Honesty Relayer - MORE THAN ONE");

					None
				}
			}
		} else {
			// Should never enter this condition
			error!(target: "relayer-game", "   >  Relay Affirmations - EMPTY");

			None
		}
	}

	pub fn update_game_at(game_id: &RelayHeaderId<T, I>, last_round: u32, moment: BlockNumber<T>) {
		Self::update_timer_of_game_at(game_id, last_round + 1, moment);

		<RoundCounts<T, I>>::mutate(&game_id, |round_count| {
			*round_count = round_count.saturating_add(1)
		});
		<GameSamplePoints<T, I>>::mutate(game_id, |game_sample_points| {
			T::RelayerGameAdjustor::update_sample_points(game_sample_points);
			T::RelayableChain::new_round(
				game_id,
				game_sample_points
					.last()
					.map(ToOwned::to_owned)
					.unwrap_or_default(),
			);
		});
	}

	pub fn game_over(game_id: RelayHeaderId<T, I>) {
		// TODO: error trace
		let _ = <RelayHeaderParcelToResolve<T, I>>::try_mutate(|relay_header_parcel_to_resolve| {
			if let Some(i) = relay_header_parcel_to_resolve
				.iter()
				.position(|block_id| block_id == &game_id)
			{
				relay_header_parcel_to_resolve.remove(i);

				Ok(())
			} else {
				Err(())
			}
		});
		<Affirmations<T, I>>::remove_prefix(&game_id);
		<BestConfirmedHeaderId<T, I>>::take(&game_id);
		<RoundCounts<T, I>>::take(&game_id);
		<AffirmTime<T, I>>::take(&game_id);
		<GameSamplePoints<T, I>>::take(&game_id);

		T::RelayableChain::game_over(&game_id);
	}

	pub fn update_games(game_ids: Vec<RelayHeaderId<T, I>>) -> DispatchResult {
		let now = <frame_system::Module<T>>::block_number();
		let mut relay_header_parcels = vec![];

		for game_id in game_ids {
			trace!(
				target: "relayer-game",
				">  Trying to Settle Game `{:?}`", game_id
			);

			let round_count = Self::round_count_of(&game_id);
			let last_round = if let Some(last_round) = round_count.checked_sub(1) {
				last_round
			} else {
				// Should never enter this condition
				error!(target: "relayer-game", "   >  Rounds - EMPTY");

				continue;
			};
			let mut relay_affirmations = Self::affirmations_of_game_at(&game_id, last_round);

			match (last_round, relay_affirmations.len()) {
				// Should never enter this condition
				(0, 0) => error!(target: "relayer-game", "   >  Affirmations - EMPTY"),
				// At first round and only one affirmation found
				(0, 1) => {
					trace!(target: "relayer-game", "   >  Challenge - NOT EXISTED");

					if let Some(relay_header_parcel) =
						Self::settle_without_challenge(relay_affirmations.pop().unwrap())
					{
						relay_header_parcels.push(relay_header_parcel);
					}
				}
				// No relayer response for the latest round
				(_, 0) => {
					trace!(target: "relayer-game", "   >  All Relayers Abstain, Settle Abandon");

					Self::settle_abandon(&game_id);
				}
				// No more challenge found at latest round, only one relayer win
				(_, 1) => {
					trace!(target: "relayer-game", "   >  No More Challenge, Settle With Challenge");

					if let Some(relay_header_parcel) =
						Self::settle_with_challenge(&game_id, relay_affirmations.pop().unwrap())
					{
						relay_header_parcels.push(relay_header_parcel);
					} else {
						// Should never enter this condition

						Self::settle_abandon(&game_id);
					}
				}
				(last_round, _) => {
					let distance = T::RelayableChain::distance_between(
						&game_id,
						Self::best_confirmed_header_id_of(&game_id),
					);

					if distance == round_count {
						trace!(target: "relayer-game", "   >  A Full Chain Gave, On Chain Arbitrate");

						// A whole chain gave, start continuous verification
						if let Some(relay_header_parcel) = Self::on_chain_arbitrate(&game_id) {
							relay_header_parcels.push(relay_header_parcel);
						} else {
							Self::settle_abandon(&game_id);
						}
					} else {
						trace!(target: "relayer-game", "   >  Still In Challenge, Update Games");

						// Update game, start new round
						Self::update_game_at(&game_id, last_round, now);

						continue;
					}
				}
			}

			Self::game_over(game_id);
		}

		// TODO: handle error
		let _ = T::RelayableChain::try_confirm_relay_header_parcels(relay_header_parcels);

		trace!(target: "relayer-game", "---");

		Ok(())
	}
}

impl<T: Config<I>, I: Instance> RelayerGameProtocol for Module<T, I> {
	type Relayer = AccountId<T>;
	type RelayHeaderId = RelayHeaderId<T, I>;
	type RelayHeaderParcel = RelayHeaderParcel<T, I>;
	type RelayProofs = RelayProofs<T, I>;

	fn get_proposed_relay_header_parcels(
		relay_affirmation_id: &RelayAffirmationId<Self::RelayHeaderId>,
	) -> Option<Vec<Self::RelayHeaderParcel>> {
		let RelayAffirmationId {
			game_id,
			round,
			index,
		} = relay_affirmation_id;

		Self::affirmations_of_game_at(&game_id, round)
			.into_iter()
			.nth(*index as usize)
			.map(|relay_affirmation| relay_affirmation.relay_header_parcels)
	}

	fn best_confirmed_header_id_of(game_id: &Self::RelayHeaderId) -> Self::RelayHeaderId {
		Self::best_confirmed_header_id_of(game_id)
	}

	fn affirm(
		relayer: &Self::Relayer,
		relay_header_parcel: Self::RelayHeaderParcel,
		optional_relay_proofs: Option<Self::RelayProofs>,
	) -> Result<Self::RelayHeaderId, DispatchError> {
		trace!(
			target: "relayer-game",
			"Relayer `{:?}` affirm:\n{:#?}",
			relayer,
			relay_header_parcel
		);

		let best_confirmed_relay_header_id = T::RelayableChain::best_confirmed_relay_header_id();
		let game_id = relay_header_parcel.header_id();

		// Check if the proposed header has already been confirmed
		ensure!(
			game_id > best_confirmed_relay_header_id,
			<Error<T, I>>::RelayParcelAR
		);
		// Make sure the game is at first round
		ensure!(
			<Affirmations<T, I>>::decode_len(&game_id, 1).unwrap_or(0) == 0,
			<Error<T, I>>::RoundMis
		);

		let now = <frame_system::Module<T>>::block_number();
		let proposed_relay_header_parcels = vec![relay_header_parcel];

		// Check if it is a new game
		ensure!(
			<Affirmations<T, I>>::decode_len(&game_id, 0).unwrap_or(0) == 0,
			<Error<T, I>>::ExistedAffirmationsFoundC
		);
		// Check if it is ok to open more games
		ensure!(
			<RelayHeaderParcelToResolve<T, I>>::decode_len()
				.map(|length| length as u8)
				.unwrap_or(0) < T::RelayerGameAdjustor::max_active_games(),
			<Error<T, I>>::ActiveGamesTM
		);

		let stake = Self::ensure_can_stake(relayer, 0, 1)?;

		Self::update_stakes_with(relayer, |old_stakes| old_stakes.saturating_add(stake));

		let relay_affirmation = {
			let mut relay_affirmation = RelayAffirmation::new();

			relay_affirmation.relayer = relayer.to_owned();
			relay_affirmation.relay_header_parcels = proposed_relay_header_parcels;
			relay_affirmation.stake = stake;

			// Allow affirm without relay proofs
			// The relay proofs can be completed later through `complete_proofs`
			if let Some(relay_proofs) = optional_relay_proofs {
				T::RelayableChain::verify_relay_proofs(
					&game_id,
					&relay_affirmation.relay_header_parcels[0],
					&relay_proofs,
					Some(&best_confirmed_relay_header_id),
				)?;

				relay_affirmation.verified_on_chain = true;
			}

			relay_affirmation
		};

		<Affirmations<T, I>>::append(&game_id, 0, relay_affirmation);
		<BestConfirmedHeaderId<T, I>>::insert(&game_id, best_confirmed_relay_header_id);
		<RoundCounts<T, I>>::insert(&game_id, 1);
		<RelayHeaderParcelToResolve<T, I>>::mutate(|relay_header_parcel_to_resolve| {
			relay_header_parcel_to_resolve.push(game_id.clone())
		});
		<GameSamplePoints<T, I>>::append(&game_id, vec![game_id.clone()]);

		Self::update_timer_of_game_at(&game_id, 0, now);

		Ok(game_id)
	}

	fn dispute_and_affirm(
		relayer: &Self::Relayer,
		relay_header_parcel: Self::RelayHeaderParcel,
		optional_relay_proofs: Option<Self::RelayProofs>,
	) -> Result<(Self::RelayHeaderId, u32), DispatchError> {
		trace!(
			target: "relayer-game",
			"Relayer `{:?}` dispute and affirm:\n{:#?}",
			relayer,
			relay_header_parcel
		);

		let best_confirmed_relay_header_id = T::RelayableChain::best_confirmed_relay_header_id();
		let game_id = relay_header_parcel.header_id();

		// Check if the proposed header has already been confirmed
		ensure!(
			game_id > best_confirmed_relay_header_id,
			<Error<T, I>>::RelayParcelAR
		);

		let now = <frame_system::Module<T>>::block_number();

		ensure!(
			Self::is_game_open_at(&game_id, now, 0),
			<Error<T, I>>::GameAtThisRoundC
		);

		let proposed_relay_header_parcels = vec![relay_header_parcel];
		let existed_affirmations = Self::affirmations_of_game_at(&game_id, 0);

		// Currently not allow to vote for(relay) the same parcel
		ensure!(
			Self::is_unique_affirmation(&proposed_relay_header_parcels, &existed_affirmations),
			<Error<T, I>>::RelayAffirmationDup
		);

		let existed_relay_affirmations_count = existed_affirmations.len() as u32;
		let stake = Self::ensure_can_stake(
			relayer,
			0,
			existed_relay_affirmations_count.saturating_add(1),
		)?;

		Self::update_stakes_with(relayer, |old_stakes| old_stakes.saturating_add(stake));

		let relay_affirmation = {
			let mut relay_affirmation = RelayAffirmation::new();

			relay_affirmation.relayer = relayer.to_owned();
			relay_affirmation.relay_header_parcels = proposed_relay_header_parcels;
			relay_affirmation.stake = stake;

			// Allow affirm without relay proofs
			// The relay proofs can be completed later through `complete_proofs`
			if let Some(relay_proofs) = optional_relay_proofs {
				T::RelayableChain::verify_relay_proofs(
					&game_id,
					&relay_affirmation.relay_header_parcels[0],
					&relay_proofs,
					Some(&best_confirmed_relay_header_id),
				)?;

				relay_affirmation.verified_on_chain = true;
			}

			relay_affirmation
		};

		<Affirmations<T, I>>::append(&game_id, 0, relay_affirmation);

		Ok((
			game_id,
			// index == affirmations_count - 1 == existed_relay_affirmations_count
			existed_relay_affirmations_count,
		))
	}

	fn complete_relay_proofs(
		relay_affirmation_id: RelayAffirmationId<Self::RelayHeaderId>,
		relay_proofs: Vec<Self::RelayProofs>,
	) -> DispatchResult {
		let RelayAffirmationId {
			game_id,
			round,
			index,
		} = relay_affirmation_id;

		<Affirmations<T, I>>::try_mutate(&game_id, round, |relay_affirmations| {
			if let Some(relay_affirmation) = relay_affirmations.get_mut(index as usize) {
				for (relay_header_parcel, relay_proofs) in relay_affirmation
					.relay_header_parcels
					.iter()
					.zip(relay_proofs.into_iter())
				{
					if round == 0 {
						T::RelayableChain::verify_relay_proofs(
							&game_id,
							relay_header_parcel,
							&relay_proofs,
							Some(&Self::best_confirmed_header_id_of(&game_id)),
						)?;
					} else {
						T::RelayableChain::verify_relay_proofs(
							&game_id,
							relay_header_parcel,
							&relay_proofs,
							None,
						)?;
					}
				}

				relay_affirmation.verified_on_chain = true;

				Ok(())
			} else {
				Err(<Error<T, I>>::RelayAffirmationNE.into())
			}
		})
	}

	fn extend_affirmation(
		relayer: &Self::Relayer,
		extended_relay_affirmation_id: RelayAffirmationId<Self::RelayHeaderId>,
		game_sample_points: Vec<Self::RelayHeaderParcel>,
		optional_relay_proofs: Option<Vec<Self::RelayProofs>>,
	) -> Result<(Self::RelayHeaderId, u32, u32), DispatchError> {
		trace!(
			target: "relayer-game",
			"Relayer `{:?}` extend affirmation: {:?} with: {:?}",
			relayer,
			extended_relay_affirmation_id,
			game_sample_points,
		);

		let RelayAffirmationId {
			game_id,
			round: previous_round,
			index: previous_index,
		} = extended_relay_affirmation_id.clone();
		let round = previous_round + 1;

		ensure!(
			Self::is_game_open_at(&game_id, <frame_system::Module<T>>::block_number(), round),
			<Error<T, I>>::GameAtThisRoundC
		);

		if let Some(ref relay_proofs) = &optional_relay_proofs {
			ensure!(
				relay_proofs.len() == game_sample_points.len(),
				<Error<T, I>>::RelayProofsQuantityInv
			);
		}

		let existed_affirmations = Self::affirmations_of_game_at(&game_id, previous_round);

		ensure!(
			Self::is_unique_affirmation(&game_sample_points, &existed_affirmations),
			<Error<T, I>>::RelayAffirmationDup
		);

		let extended_affirmation = existed_affirmations
			.get(previous_index as usize)
			.ok_or(<Error<T, I>>::ExtendedRelayAffirmationNE)?;

		// Currently only accept extending from a completed affirmation
		ensure!(
			extended_affirmation.verified_on_chain,
			<Error<T, I>>::PreviousRelayProofsInc
		);

		T::RelayableChain::preverify_game_sample_points(
			&extended_relay_affirmation_id,
			&game_sample_points,
		)?;

		let stake = Self::ensure_can_stake(
			relayer,
			round,
			(existed_affirmations.len() as u32).saturating_add(1),
		)?;

		Self::update_stakes_with(relayer, |old_stakes| old_stakes.saturating_add(stake));

		let relay_affirmation = {
			let mut relay_affirmation = RelayAffirmation::new();

			relay_affirmation.relayer = relayer.to_owned();
			relay_affirmation.relay_header_parcels = game_sample_points;
			relay_affirmation.stake = stake;
			relay_affirmation.maybe_extended_relay_affirmation_id =
				Some(extended_relay_affirmation_id);

			// Allow affirm without relay proofs
			// The relay proofs can be completed later through `complete_proofs`
			if let Some(relay_proofs) = optional_relay_proofs {
				for (relay_header_parcel, relay_proofs) in relay_affirmation
					.relay_header_parcels
					.iter()
					.zip(relay_proofs.into_iter())
				{
					T::RelayableChain::verify_relay_proofs(
						&game_id,
						relay_header_parcel,
						&relay_proofs,
						None,
					)?;
				}

				relay_affirmation.verified_on_chain = true;
			}

			relay_affirmation
		};
		let index = <Affirmations<T, I>>::decode_len(&game_id, round)
			.map(|length| length as u32)
			.unwrap_or(0);

		<Affirmations<T, I>>::append(&game_id, round, relay_affirmation);

		Ok((game_id, round, index))
	}
}
