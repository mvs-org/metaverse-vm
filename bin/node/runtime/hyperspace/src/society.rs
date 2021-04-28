// --- substrate ---
use pallet_society::{Config, EnsureFounder};
use sp_runtime::ModuleId;
// --- hyperspace ---
use crate::*;

frame_support::parameter_types! {
	pub const SocietyModuleId: ModuleId = ModuleId(*b"da/socie");
	pub const CandidateDeposit: Balance = 10 * COIN;
	pub const WrongSideDeduction: Balance = 2 * COIN;
	pub const MaxStrikes: u32 = 10;
	pub const RotationPeriod: BlockNumber = 3 * MINUTES;
	pub const PeriodSpend: Balance = 500 * COIN;
	pub const MaxLockDuration: BlockNumber = 3 * MINUTES;
	pub const ChallengePeriod: BlockNumber = 3 * MINUTES;
}
impl Config for Runtime {
	type Event = Event;
	type ModuleId = SocietyModuleId;
	type Currency = Etp;
	type Randomness = RandomnessCollectiveFlip;
	type CandidateDeposit = CandidateDeposit;
	type WrongSideDeduction = WrongSideDeduction;
	type MaxStrikes = MaxStrikes;
	type PeriodSpend = PeriodSpend;
	type MembershipChanged = ();
	type RotationPeriod = RotationPeriod;
	type MaxLockDuration = MaxLockDuration;
	type FounderSetOrigin = EnsureRootOrMoreThanHalfCouncil;
	type SuspensionJudgementOrigin = EnsureFounder<Runtime>;
	type ChallengePeriod = ChallengePeriod;
}
