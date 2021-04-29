// --- substrate ---
use frame_system::{EnsureOneOf, EnsureRoot};
use pallet_collective::{EnsureMember, EnsureProportionAtLeast};
use sp_core::u32_trait::{_1, _2, _3};
// --- hyperspace ---
use crate::*;
use hyperspace_democracy::{weights::SubstrateWeight, Config};

frame_support::parameter_types! {
	pub const LaunchPeriod: BlockNumber = 3 * MINUTES;
	pub const VotingPeriod: BlockNumber = 3 * MINUTES;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * MINUTES;
	pub const MinimumDeposit: Balance = 1 * COIN;
	pub const EnactmentPeriod: BlockNumber = 3 * MINUTES;
	pub const CooloffPeriod: BlockNumber = 3 * MINUTES;
	pub const PreimageByteDeposit: Balance = 1 * MILLI;
	pub const InstantAllowed: bool = true;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}
impl Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Etp;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EnsureRootOrHalfCouncil;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = EnsureRootOrHalfCouncil;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
	type InstantOrigin = EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type OperationalPreimageOrigin = EnsureMember<AccountId, CouncilCollective>;
	type MaxProposals = MaxProposals;
	type WeightInfo = SubstrateWeight<Runtime>;
}
