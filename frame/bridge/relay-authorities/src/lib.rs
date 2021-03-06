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

//! # Relay Authorities Module

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;
// --- hyperspace ---
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

mod types {
	// --- hyperspace ---
	use crate::*;

	pub type AccountId<T> = <T as frame_system::Config>::AccountId;
	pub type BlockNumber<T> = <T as frame_system::Config>::BlockNumber;
	pub type MMRRoot<T> = <T as frame_system::Config>::Hash;
	pub type EtpBalance<T, I> = <EtpCurrency<T, I> as Currency<AccountId<T>>>::Balance;
	pub type EtpCurrency<T, I> = <T as Config<I>>::EtpCurrency;

	pub type RelayAuthoritySigner<T, I> = <<T as Config<I>>::Sign as Sign<BlockNumber<T>>>::Signer;
	pub type RelayAuthorityMessage<T, I> =
		<<T as Config<I>>::Sign as Sign<BlockNumber<T>>>::Message;
	pub type RelayAuthoritySignature<T, I> =
		<<T as Config<I>>::Sign as Sign<BlockNumber<T>>>::Signature;
	pub type RelayAuthorityT<T, I> =
		RelayAuthority<AccountId<T>, RelayAuthoritySigner<T, I>, EtpBalance<T, I>, BlockNumber<T>>;
	pub type ScheduledAuthoritiesChangeT<T, I> = ScheduledAuthoritiesChange<
		AccountId<T>,
		RelayAuthoritySigner<T, I>,
		EtpBalance<T, I>,
		BlockNumber<T>,
	>;
}

// --- crates ---
use codec::Encode;
// --- substrate ---
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	traits::{Currency, EnsureOrigin, Get, LockIdentifier},
	weights::Weight,
	StorageValue,
};
use frame_system::ensure_signed;
use sp_runtime::{
	traits::{Saturating, Zero},
	DispatchError, DispatchResult, Perbill, SaturatedConversion,
};
#[cfg(not(feature = "std"))]
use sp_std::borrow::ToOwned;
use sp_std::prelude::*;
// --- hyperspace ---
use hyperspace_relay_primitives::relay_authorities::*;
use hyperspace_support::balance::lock::*;
use types::*;

pub trait Config<I: Instance = DefaultInstance>: frame_system::Config {
	type Event: From<Event<Self, I>> + Into<<Self as frame_system::Config>::Event>;
	type EtpCurrency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	type LockId: Get<LockIdentifier>;
	type TermDuration: Get<Self::BlockNumber>;
	type MaxCandidates: Get<usize>;
	type AddOrigin: EnsureOrigin<Self::Origin>;
	type RemoveOrigin: EnsureOrigin<Self::Origin>;
	type ResetOrigin: EnsureOrigin<Self::Origin>;
	type HyperspaceMMR: MMR<Self::BlockNumber, Self::Hash>;
	type Sign: Sign<Self::BlockNumber>;
	type OpCodes: Get<(OpCode, OpCode)>;
	type SignThreshold: Get<Perbill>;
	type SubmitDuration: Get<Self::BlockNumber>;
	type WeightInfo: WeightInfo;
}

decl_event! {
	pub enum Event<T, I: Instance = DefaultInstance>
	where
		AccountId = AccountId<T>,
		BlockNumber = BlockNumber<T>,
		EtpBalance = EtpBalance<T, I>,
		MMRRoot = MMRRoot<T>,
		RelayAuthoritySigner = RelayAuthoritySigner<T, I>,
		RelayAuthorityMessage = RelayAuthorityMessage<T, I>,
		RelayAuthoritySignature = RelayAuthoritySignature<T, I>,
	{
		/// A New MMR Root Scheduled Request to be Signed. [block number of the mmr root to sign]
		ScheduleMMRRoot(BlockNumber),
		/// MMR Root Signed. [block number of the mmr root, mmr root, signatures]
		MMRRootSigned(BlockNumber, MMRRoot, Vec<(AccountId, RelayAuthoritySignature)>),
		/// A New Authority Set Change Scheduled Request to be Signed. [message to sign]
		ScheduleAuthoritiesChange(RelayAuthorityMessage),
		/// The Next Authorities Signed. [term, next authorities, signatures]
		AuthoritiesChangeSigned(Term, Vec<RelayAuthoritySigner>, Vec<(AccountId, RelayAuthoritySignature)>),
		/// Slash on Misbehavior. [who, slashed]
		SlashOnMisbehavior(AccountId, EtpBalance),
	}
}

decl_error! {
	pub enum Error for Module<T: Config<I>, I: Instance> {
		/// Candidate - ALREADY EXISTED
		CandidateAE,
		/// Candidate - NOT EXISTED
		CandidateNE,
		/// Authority - ALREADY EXISTED
		AuthorityAE,
		/// Authority - NOT EXISTED
		AuthorityNE,
		/// Authority - IN TERM
		AuthorityIT,
		/// Authorities Count - TOO LOW
		AuthoritiesCountTL,
		/// Stake - INSUFFICIENT
		StakeIns,
		/// On Authorities Change - DISABLED
		OnAuthoritiesChangeDis,
		/// Scheduled Sign -NOT EXISTED
		ScheduledSignNE,
		/// Hyperspace MMR Root - NOT READY YET
		HyperspaceMMRRootNRY,
		/// Signature - INVALID
		SignatureInv,
		/// Term - MISMATCHED
		TermMis,
		/// Authorities - MISMATCHED
		AuthoritiesMis,
		/// Next Authorities - NOT EXISTED
		NextAuthoritiesNE,
	}
}

decl_storage! {
	trait Store for Module<T: Config<I>, I: Instance = DefaultInstance> as HyperspaceRelayAuthorities {
		/// Anyone can request to be an authority with some stake
		/// Also submit your signer at the same time (for ethereum: your ethereum address in H160 format)
		///
		/// Once you requested, you'll enter the candidates
		///
		/// This request can be canceled at any time
		pub Candidates get(fn candidates): Vec<RelayAuthorityT<T, I>>;

		/// Authority must elect from candidates
		///
		/// Only council or root can be the voter of the election
		///
		/// Once you become an authority, you must serve for a specific term.
		/// Before that, you can't renounce
		pub Authorities get(fn authorities): Vec<RelayAuthorityT<T, I>>;

		/// The scheduled change of authority set
		pub NextAuthorities get(fn next_authorities): Option<ScheduledAuthoritiesChangeT<T, I>>;

		/// A term index counter, play the same role as nonce in extrinsic
		pub NextTerm get(fn next_term): Term;

		/// The authorities change requirements
		///
		/// Once the signatures count reaches the sign threshold storage will be killed then raise a signed event
		///
		/// Params
		/// 	1. the message to sign
		/// 	1. collected signatures
		pub AuthoritiesToSign
			get(fn authorities_to_sign)
			: Option<(RelayAuthorityMessage<T, I>, Vec<(AccountId<T>, RelayAuthoritySignature<T, I>)>)>;

		/// The `MMRRootsToSign` keys cache
		///
		/// Only use for update the `MMRRootsToSign` once the authorities changed
		pub MMRRootsToSignKeys get(fn mmr_root_to_sign_keys): Vec<BlockNumber<T>>;

		/// All the relay requirements from the backing module here
		///
		/// If the map's key has existed, it means the mmr root relay requirement is valid
		///
		/// Once the signatures count reaches the sign threshold storage will be killed then raise a signed event
		///
		/// Params
		/// 	1. collected signatures
		pub MMRRootsToSign
			get(fn mmr_root_to_sign_of)
			: map hasher(identity) BlockNumber<T>
			=> Option<Vec<(AccountId<T>, RelayAuthoritySignature<T, I>)>>;

		/// The mmr root signature submit duration, will be delayed if on authorities change
		pub SubmitDuration get(fn submit_duration): BlockNumber<T> = T::SubmitDuration::get();
	}
	add_extra_genesis {
		config(authorities): Vec<(AccountId<T>, RelayAuthoritySigner<T, I>, EtpBalance<T, I>)>;
		build(|config| {
			let mut authorities = vec![];

			for (account_id, signer, stake) in config.authorities.iter() {
				T::EtpCurrency::set_lock(
					T::LockId::get(),
					account_id,
					LockFor::Common { amount: *stake },
					WithdrawReasons::all(),
				);

				authorities.push(RelayAuthority {
					account_id: account_id.to_owned(),
					signer: signer.to_owned(),
					stake: *stake,
					term: <frame_system::Module<T>>::block_number() + T::TermDuration::get()
				});
			}

			<Authorities<T, I>>::put(authorities);
		});
	}
}

decl_module! {
	pub struct Module<T: Config<I>, I: Instance = DefaultInstance> for enum Call
	where
		origin: T::Origin
	{
		type Error = Error<T, I>;

		const LockId: LockIdentifier = T::LockId::get();

		const TermDuration: BlockNumber<T> = T::TermDuration::get();

		const MaxCandidates: u32 = T::MaxCandidates::get() as _;

		const SignThreshold: Perbill = T::SignThreshold::get();

		const SubmitDuration: BlockNumber<T> = T::SubmitDuration::get();

		fn deposit_event() = default;

		// Deal with the slash thing. If authority didn't do his job before the deadline
		fn on_initialize(now: BlockNumber<T>) -> Weight {
			Self::check_misbehavior(now);

			0
		}

		/// Request to be an authority
		///
		/// This will be failed if match one of these sections:
		/// - already is a candidate
		/// - already is an authority
		/// - insufficient stake, required at least more than the last candidate's
		///   if too there're many candidates in the candidates' queue
		#[weight = 10_000_000]
		pub fn request_authority(
			origin,
			stake: EtpBalance<T, I>,
			signer: RelayAuthoritySigner<T, I>,
		) {
			let account_id = ensure_signed(origin)?;

			if let Some(scheduled_authorities_change) = <NextAuthorities<T, I>>::get() {
				ensure!(
					find_authority_position::<T, I>(
						&scheduled_authorities_change.next_authorities,
						&account_id
					).is_none(),
					<Error<T, I>>::AuthorityAE
				);
			}

			ensure!(
				find_authority_position::<T, I>(&<Authorities<T, I>>::get(), &account_id).is_none(),
				<Error<T, I>>::AuthorityAE
			);
			ensure!(
				<EtpCurrency<T, I>>::usable_balance(&account_id) > stake,
				<Error<T, I>>::StakeIns
			);

			<Candidates<T, I>>::try_mutate(|candidates| {
				ensure!(
					find_authority_position::<T, I>(candidates, &account_id).is_none(),
					<Error<T, I>>::CandidateAE
				);

				// Max candidates can't be zero
				if candidates.len() == T::MaxCandidates::get() {
					let mut minimum_stake = candidates[0].stake;
					let mut position = 0;

					for (i, candidate) in candidates.iter().skip(1).enumerate() {
						let stake = candidate.stake;

						if stake < minimum_stake {
							minimum_stake = stake;
							position = i;
						}
					}

					ensure!(stake > minimum_stake, <Error<T, I>>::StakeIns);

					// TODO: slash the weed out?
					let weep_out = candidates.remove(position);

					<EtpCurrency<T, I>>::remove_lock(T::LockId::get(), &weep_out.account_id);
				}

				<EtpCurrency<T, I>>::set_lock(
					T::LockId::get(),
					&account_id,
					LockFor::Common { amount: stake },
					WithdrawReasons::all()
				);

				candidates.push(RelayAuthority {
					account_id,
					signer,
					stake,
					term: 0u32.into()
				});

				DispatchResult::Ok(())
			})?;
		}

		/// This would never fail. No-op if can't find the request
		#[weight = 10_000_000]
		pub fn cancel_request(origin) {
			let account_id = ensure_signed(origin)?;
			let _ = Self::remove_candidate_by_id_with(
				&account_id,
				|| <EtpCurrency<T, I>>::remove_lock(T::LockId::get(), &account_id)
			);
		}

		/// Require reset origin
		///
		/// Clear the candidates
		#[weight = 10_000_000]
		pub fn kill_candidates(origin) {
			T::ResetOrigin::ensure_origin(origin)?;

			let lock_id = T::LockId::get();

			for RelayAuthority { account_id, .. } in <Candidates<T, I>>::take() {
				<EtpCurrency<T, I>>::remove_lock(lock_id, &account_id);
			}
		}

		/// Require add origin
		///
		/// Add an authority from the candidates
		///
		/// This call is disallowed during the authorities change
		#[weight = 10_000_000]
		pub fn add_authorities(origin, account_ids: Vec<AccountId<T>>) {
			T::AddOrigin::ensure_origin(origin)?;

			ensure!(!Self::on_authorities_change(), <Error<T, I>>::OnAuthoritiesChangeDis);
			// Won't check duplicated here, MUST make this authority sure is unique
			// As we already make a check in `request_authority`
			let next_authorities = {
				let mut authorities = <Authorities<T, I>>::get();

				for account_id in account_ids {
					let mut authority = Self::remove_candidate_by_id_with(&account_id, || ())?;
					authority.term = <frame_system::Module<T>>::block_number() + T::TermDuration::get();

					authorities.push(authority);
				}

				authorities
			};

			Self::schedule_authorities_change(next_authorities);
		}

		/// Renounce the authority for you
		///
		/// This call is disallowed during the authorities change
		///
		/// No-op if can't find the authority
		///
		/// Will fail if you still in the term
		#[weight = 10_000_000]
		pub fn renounce_authority(origin) {
			let account_id = ensure_signed(origin)?;

			ensure!(!Self::on_authorities_change(), <Error<T, I>>::OnAuthoritiesChangeDis);

			let next_authorities = Self::remove_authority_by_ids_with(
				vec![account_id],
				|authority| if authority.term >= <frame_system::Module<T>>::block_number() {
					Some(<Error<T, I>>::AuthorityIT)
				} else {
					None
				}
			)?;

			if next_authorities.is_empty() {
				Err(<Error<T, I>>::AuthoritiesCountTL)?;
			}

			Self::schedule_authorities_change(next_authorities);
		}

		/// Require remove origin
		///
		/// This call is disallowed during the authorities change
		#[weight = 10_000_000]
		pub fn remove_authorities(origin, account_ids: Vec<AccountId<T>>) {
			T::RemoveOrigin::ensure_origin(origin)?;

			ensure!(!Self::on_authorities_change(), <Error<T, I>>::OnAuthoritiesChangeDis);

			let next_authorities = Self::remove_authority_by_ids_with(account_ids, |_| None)?;

			if next_authorities.is_empty() {
				Err(<Error<T, I>>::AuthoritiesCountTL)?;
			}

			Self::schedule_authorities_change(next_authorities);
		}

		/// Require authority origin
		///
		/// This call is disallowed during the authorities change
		///
		/// No-op if already submit
		///
		/// Verify
		/// - the relay requirement is valid
		/// - the signature is signed by the submitter
		#[weight = 10_000_000]
		pub fn submit_signed_mmr_root(
			origin,
			block_number: BlockNumber<T>,
			signature: RelayAuthoritySignature<T, I>
		) {
			let authority = ensure_signed(origin)?;

			// Not allow to submit during the authority set change
			ensure!(!Self::on_authorities_change(), <Error<T, I>>::OnAuthoritiesChangeDis);

			let mut signatures =
				<MMRRootsToSign<T, I>>::get(block_number).ok_or(<Error<T, I>>::ScheduledSignNE)?;

			// No-op if was already submitted
			if signatures.iter().position(|(authority_, _)| authority_ == &authority).is_some() {
				return Ok(());
			}

			let authorities = <Authorities<T, I>>::get();
			let signer = find_signer::<T, I>(
				&authorities,
				&authority
			).ok_or(<Error<T, I>>::AuthorityNE)?;
			let mmr_root =
				T::HyperspaceMMR::get_root(block_number).ok_or(<Error<T, I>>::HyperspaceMMRRootNRY)?;

			// The message is composed of:
			//
			// hash(
			// 	codec(
			// 		spec_name: String,
			// 		op_code: OpCode,
			// 		block number: Compact<BlockNumber>,
			// 		mmr_root: Hash
			// 	)
			// )
			let message = T::Sign::hash(
				&_S {
					_1: T::Version::get().spec_name,
					_2: T::OpCodes::get().0,
					_3: block_number,
					_4: mmr_root
				}
				.encode()
			);

			ensure!(
				T::Sign::verify_signature(&signature, &message, &signer),
				 <Error<T, I>>::SignatureInv
			);

			signatures.push((authority, signature));

			if Perbill::from_rational_approximation(signatures.len() as u32, authorities.len() as _)
				>= T::SignThreshold::get()
			{
				// TODO: clean the mmr root which was contains in this mmr root?

				Self::mmr_root_signed(block_number);
				Self::deposit_event(RawEvent::MMRRootSigned(block_number, mmr_root, signatures));
			} else {
				<MMRRootsToSign<T, I>>::insert(block_number, signatures);
			}
		}

		/// Require authority origin
		///
		/// This call is only allowed during the authorities change
		///
		/// No-op if already submit
		///
		/// Verify
		/// - the relay requirement is valid
		/// - the signature is signed by the submitter
		#[weight = 10_000_000]
		pub fn submit_signed_authorities(origin, signature: RelayAuthoritySignature<T, I>) {
			let authority = ensure_signed(origin)?;

			ensure!(Self::on_authorities_change(), <Error<T, I>>::OnAuthoritiesChangeDis);

			let (message, mut signatures) = if let Some(signatures) = <AuthoritiesToSign<T, I>>::get() {
				signatures
			} else {
				return Ok(());
			};

			if signatures
				.iter()
				.position(|(authority_, _)| authority_ == &authority)
				.is_some()
			{
				return Ok(());
			}

			let authorities = <Authorities<T, I>>::get();
			let signer = find_signer::<T, I>(
				&authorities,
				&authority
			).ok_or(<Error<T, I>>::AuthorityNE)?;

			ensure!(
				T::Sign::verify_signature(&signature, &message, &signer),
				 <Error<T, I>>::SignatureInv
			);

			signatures.push((authority, signature));

			if Perbill::from_rational_approximation(signatures.len() as u32, authorities.len() as _)
				>= T::SignThreshold::get()
			{
				Self::apply_authorities_change()?;
				Self::deposit_event(RawEvent::AuthoritiesChangeSigned(
					<NextTerm<I>>::get(),
					<NextAuthorities<T, I>>::get()
						.ok_or(<Error<T, I>>::NextAuthoritiesNE)?
						.next_authorities
						.into_iter()
						.map(|authority| authority.signer)
						.collect(),
					signatures
				));
			} else {
				<AuthoritiesToSign<T, I>>::put((message, signatures));
			}
		}

		#[weight = 10_000_000]
		pub fn kill_authorities(origin) {
			T::ResetOrigin::ensure_origin(origin)?;

			let lock_id = T::LockId::get();

			for RelayAuthority { account_id, .. } in <Authorities<T, I>>::take() {
				<EtpCurrency<T, I>>::remove_lock(lock_id, &account_id);
			}

			<NextAuthorities<T, I>>::kill();
			<AuthoritiesToSign<T, I>>::kill();
			{
				<MMRRootsToSign<T, I>>::remove_all();
				let schedule = (
					<frame_system::Module<T>>::block_number().saturated_into::<u64>() / 10 * 10 + 10
				).saturated_into();
				<MMRRootsToSignKeys<T, I>>::mutate(|schedules| *schedules = vec![schedule]);
				Self::schedule_mmr_root(schedule);
			}
			<SubmitDuration<T, I>>::kill();
		}

		#[weight = 10_000_000]
		pub fn force_new_term(origin) {
			T::ResetOrigin::ensure_origin(origin)?;

			Self::apply_authorities_change()?;
			Self::sync_authorities_change()?;

			<NextAuthorities<T, I>>::kill();
		}
	}
}

impl<T, I> Module<T, I>
where
	T: Config<I>,
	I: Instance,
{
	pub fn remove_candidate_by_id_with<F>(
		account_id: &AccountId<T>,
		f: F,
	) -> Result<RelayAuthorityT<T, I>, DispatchError>
	where
		F: Fn(),
	{
		Ok(<Candidates<T, I>>::try_mutate(|candidates| {
			if let Some(position) = find_authority_position::<T, I>(&candidates, account_id) {
				f();

				Ok(candidates.remove(position))
			} else {
				Err(<Error<T, I>>::CandidateNE)
			}
		})?)
	}

	pub fn remove_authority_by_ids_with<F>(
		account_ids: Vec<AccountId<T>>,
		f: F,
	) -> Result<Vec<RelayAuthorityT<T, I>>, DispatchError>
	where
		F: Fn(&RelayAuthorityT<T, I>) -> Option<Error<T, I>>,
	{
		let mut authorities = <Authorities<T, I>>::get();
		let mut remove_authorities = vec![];

		for account_id in account_ids.iter() {
			let position = find_authority_position::<T, I>(&authorities, account_id)
				.ok_or(<Error<T, I>>::AuthorityNE)?;

			if let Some(e) = f(&authorities[position]) {
				Err(e)?;
			}

			authorities.remove(position);
			remove_authorities.push(account_id);
		}

		if remove_authorities.is_empty() {
			Err(<Error<T, I>>::AuthorityNE)?
		}

		// TODO: optimize DB R/W, but it's ok in real case, since the set won't grow so large
		for key in <MMRRootsToSignKeys<T, I>>::get() {
			if let Some(mut signatures) = <MMRRootsToSign<T, I>>::get(key) {
				for account_id in &remove_authorities {
					if let Some(position) = signatures
						.iter()
						.position(|(authority, _)| &authority == account_id)
					{
						signatures.remove(position);
					}

					<MMRRootsToSign<T, I>>::insert(key, &signatures);
				}
			} else {
				// Should never enter this condition
				// TODO: error log
			}
		}

		Ok(authorities)
	}

	pub fn on_authorities_change() -> bool {
		<NextAuthorities<T, I>>::exists()
	}

	pub fn schedule_authorities_change(next_authorities: Vec<RelayAuthorityT<T, I>>) {
		// The message is composed of:
		//
		// hash(
		// 	codec(
		// 		spec_name: String,
		// 		op_code: OpCode,
		// 		term: Compact<u32>,
		// 		next authorities: Vec<Signer>
		// 	)
		// )
		let message = T::Sign::hash(
			&_S {
				_1: T::Version::get().spec_name,
				_2: T::OpCodes::get().1,
				_3: <NextTerm<I>>::get(),
				_4: next_authorities
					.iter()
					.map(|authority| authority.signer.clone())
					.collect::<Vec<_>>(),
			}
			.encode(),
		);

		<AuthoritiesToSign<T, I>>::put((
			&message,
			<Vec<(AccountId<T>, RelayAuthoritySignature<T, I>)>>::new(),
		));

		let submit_duration = T::SubmitDuration::get();

		<NextAuthorities<T, I>>::put(ScheduledAuthoritiesChange {
			next_authorities,
			deadline: <frame_system::Module<T>>::block_number() + submit_duration,
		});
		<SubmitDuration<T, I>>::mutate(|submit_duration_| *submit_duration_ += submit_duration);

		Self::deposit_event(RawEvent::ScheduleAuthoritiesChange(message));
	}

	pub fn apply_authorities_change() -> DispatchResult {
		let next_authorities = <NextAuthorities<T, I>>::get()
			.ok_or(<Error<T, I>>::NextAuthoritiesNE)?
			.next_authorities;
		let authorities = <Authorities<T, I>>::get();

		for RelayAuthority { account_id, .. } in authorities {
			if next_authorities
				.iter()
				.position(
					|RelayAuthority {
					     account_id: account_id_,
					     ..
					 }| account_id_ == &account_id,
				)
				.is_none()
			{
				<EtpCurrency<T, I>>::remove_lock(T::LockId::get(), &account_id);
			}
		}

		<AuthoritiesToSign<T, I>>::kill();
		<SubmitDuration<T, I>>::kill();

		Ok(())
	}

	pub fn mmr_root_signed(block_number: BlockNumber<T>) {
		<MMRRootsToSign<T, I>>::remove(block_number);
		<MMRRootsToSignKeys<T, I>>::mutate(|mmr_roots_to_sign_keys| {
			if let Some(position) = mmr_roots_to_sign_keys
				.iter()
				.position(|key| key == &block_number)
			{
				mmr_roots_to_sign_keys.remove(position);
			}
		});
	}

	pub fn check_misbehavior(now: BlockNumber<T>) {
		let find_and_slash_misbehavior =
			|signatures: Vec<(AccountId<T>, RelayAuthoritySignature<T, I>)>| {
				let _ = <Authorities<T, I>>::try_mutate(|authorities| {
					let mut storage_changed = false;

					for RelayAuthority {
						account_id, stake, ..
					} in authorities.iter_mut()
					{
						if signatures
							.iter()
							.position(|(authority, _)| authority == account_id)
							.is_none()
						{
							Self::deposit_event(RawEvent::SlashOnMisbehavior(
								account_id.to_owned(),
								*stake,
							));

							if !stake.is_zero() {
								// Can not set lock 0, so remove the lock
								T::EtpCurrency::remove_lock(T::LockId::get(), account_id);
								<EtpCurrency<T, I>>::slash(account_id, *stake);

								*stake = 0u32.into();
								storage_changed = true;
							}

							// TODO: schedule a new set
						}
					}

					if storage_changed {
						Ok(())
					} else {
						Err(())
					}
				});
			};

		if let Some(mut scheduled_authorities_change) = <NextAuthorities<T, I>>::get() {
			if scheduled_authorities_change.deadline == now {
				if let Some((_, signatures)) = <AuthoritiesToSign<T, I>>::get() {
					find_and_slash_misbehavior(signatures);
				} else {
					// Should never enter this condition
					// TODO: error log
				}

				let submit_duration = T::SubmitDuration::get();

				scheduled_authorities_change.deadline += submit_duration;

				<NextAuthorities<T, I>>::put(scheduled_authorities_change);
				<SubmitDuration<T, I>>::mutate(|submit_duration_| {
					*submit_duration_ += submit_duration
				});
			}
		} else {
			let at = now.saturating_sub(<SubmitDuration<T, I>>::get());

			if let Some(signatures) = <MMRRootsToSign<T, I>>::take(at) {
				let _ = <MMRRootsToSignKeys<T, I>>::try_mutate(|keys| {
					if let Some(position) = keys.iter().position(|key| key == &at) {
						keys.remove(position);

						Ok(())
					} else {
						Err(())
					}
				});

				find_and_slash_misbehavior(signatures);

				// TODO: schedule a new mmr root (greatest one in the keys)
			}
		}
	}
}

impl<T, I> RelayAuthorityProtocol<BlockNumber<T>> for Module<T, I>
where
	T: Config<I>,
	I: Instance,
{
	type Signer = RelayAuthoritySigner<T, I>;

	fn schedule_mmr_root(block_number: BlockNumber<T>) {
		let _ = <MMRRootsToSign<T, I>>::try_mutate(block_number, |signed_mmr_root| {
			// No-op if the sign was already scheduled
			if signed_mmr_root.is_some() {
				return Err(());
			}

			<MMRRootsToSignKeys<T, I>>::append(block_number);

			*signed_mmr_root = Some(<Vec<(AccountId<T>, RelayAuthoritySignature<T, I>)>>::new());

			Self::deposit_event(RawEvent::ScheduleMMRRoot(block_number));

			Ok(())
		});
	}

	fn check_authorities_change_to_sync(
		term: Term,
		mut authorities_change_to_sync: Vec<Self::Signer>,
	) -> DispatchResult {
		ensure!(term == <NextTerm<I>>::get(), <Error<T, I>>::TermMis);

		let mut next_authorities = <NextAuthorities<T, I>>::get()
			.ok_or(<Error<T, I>>::NextAuthoritiesNE)?
			.next_authorities
			.into_iter()
			.map(|authority| authority.signer)
			.collect::<Vec<_>>();

		authorities_change_to_sync.sort();
		next_authorities.sort();

		if authorities_change_to_sync == next_authorities {
			Ok(())
		} else {
			Err(<Error<T, I>>::AuthoritiesMis)?
		}
	}

	fn sync_authorities_change() -> DispatchResult {
		let next_authorities = <NextAuthorities<T, I>>::take()
			.ok_or(<Error<T, I>>::NextAuthoritiesNE)?
			.next_authorities;

		<Authorities<T, I>>::put(next_authorities);
		<NextTerm<I>>::mutate(|next_term| *next_term += 1);

		Ok(())
	}
}

pub fn find_authority_position<T, I>(
	authorities: &[RelayAuthorityT<T, I>],
	account_id: &AccountId<T>,
) -> Option<usize>
where
	T: Config<I>,
	I: Instance,
{
	authorities
		.iter()
		.position(|relay_authority| relay_authority == account_id)
}

pub fn find_signer<T, I>(
	authorities: &[RelayAuthorityT<T, I>],
	account_id: &AccountId<T>,
) -> Option<RelayAuthoritySigner<T, I>>
where
	T: Config<I>,
	I: Instance,
{
	if let Some(position) = authorities
		.iter()
		.position(|relay_authority| relay_authority == account_id)
	{
		Some(authorities[position].signer.to_owned())
	} else {
		None
	}
}
