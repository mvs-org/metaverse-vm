// --- substrate ---
pub use pallet_collective::{Instance0 as CouncilCollective, Instance1 as TechnicalCollective};

// --- substrate ---
use frame_system::{EnsureOneOf, EnsureRoot};
use pallet_collective::{
	weights::SubstrateWeight, Config, EnsureProportionAtLeast, EnsureProportionMoreThan,
	PrimeDefaultVote,
};
use sp_core::u32_trait::{_1, _2, _3, _5};
// --- hyperspace ---
use crate::*;

pub type EnsureRootOrHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>,
>;
pub type EnsureRootOrMoreThanHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;
pub type EnsureRootOrHalfTechnicalComittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>,
>;

pub type ApproveOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
>;

frame_support::parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 3 * MINUTES;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub const TechnicalMotionDuration: BlockNumber = 3 * MINUTES;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}
// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
static_assertions::const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());
impl Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = PrimeDefaultVote;
	type WeightInfo = SubstrateWeight<Runtime>;
}
impl Config<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = PrimeDefaultVote;
	type WeightInfo = SubstrateWeight<Runtime>;
}
