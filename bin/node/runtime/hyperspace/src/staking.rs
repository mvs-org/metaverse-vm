// --- substrate ---
use frame_support::weights::{constants::BlockExecutionWeight, DispatchClass, Weight};
use sp_runtime::{transaction_validity::TransactionPriority, ModuleId, Perbill};
use sp_staking::SessionIndex;
// --- hyperspace ---
use crate::*;
use hyperspace_staking::{weights::SubstrateWeight, Config, EraIndex};

frame_support::parameter_types! {
	pub const StakingModuleId: ModuleId = ModuleId(*b"da/staki");
	pub const SessionsPerEra: SessionIndex = SESSIONS_PER_ERA;
	pub const BondingDurationInEra: EraIndex = 2;
	pub const BondingDurationInBlockNumber: BlockNumber = 2 * BLOCKS_PER_SESSION * SESSIONS_PER_ERA;
	pub const SlashDeferDuration: EraIndex = 1;
	// quarter of the last session will be for election.
	pub const ElectionLookahead: BlockNumber = BLOCKS_PER_SESSION / 2;
	pub const MaxIterations: u32 = 5;
	pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
	pub const MaxNominatorRewardedPerValidator: u32 = 128;
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
		.saturating_sub(BlockExecutionWeight::get());
	pub const Cap: Balance = CAP;
	pub const TotalPower: Power = TOTAL_POWER;
}
impl Config for Runtime {
	type Event = Event;
	type ModuleId = StakingModuleId;
	type UnixTime = Timestamp;
	type SessionsPerEra = SessionsPerEra;
	type BondingDurationInEra = BondingDurationInEra;
	type BondingDurationInBlockNumber = BondingDurationInBlockNumber;
	type SlashDeferDuration = SlashDeferDuration;
	/// A super-majority of the council can cancel the slash.
	type SlashCancelOrigin = EnsureRootOrHalfCouncil;
	type SessionInterface = Self;
	type NextNewSession = Session;
	type ElectionLookahead = ElectionLookahead;
	type Call = Call;
	type MaxIterations = MaxIterations;
	type MinSolutionScoreBump = MinSolutionScoreBump;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type UnsignedPriority = StakingUnsignedPriority;
	// The unsigned solution weight targeted by the OCW. We set it to the maximum possible value of
	// a single extrinsic.
	type OffchainSolutionWeightLimit = OffchainSolutionWeightLimit;
	type EtpCurrency = Etp;
	type EtpRewardRemainder = Treasury;
	// send the slashed funds to the treasury.
	type EtpSlash = Treasury;
	// rewards are minted from the void
	type EtpReward = ();
	type DnaCurrency = Dna;
	// send the slashed funds to the treasury.
	type DnaSlash = Treasury;
	// rewards are minted from the void
	type DnaReward = ();
	type Cap = Cap;
	type TotalPower = TotalPower;
	type WeightInfo = SubstrateWeight<Runtime>;
}
