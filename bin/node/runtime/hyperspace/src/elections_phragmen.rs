// --- substrate ---
use frame_support::traits::{LockIdentifier, U128CurrencyToVote};
// --- hyperspace ---
use crate::*;
use hyperspace_elections_phragmen::{weights::SubstrateWeight, Config};

frame_support::parameter_types! {
	pub const ElectionsPhragmenModuleId: LockIdentifier = *b"da/phrel";
	pub const CandidacyBond: Balance = 1 * COIN;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = constants::deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = constants::deposit(0, 32);
	pub const DesiredMembers: u32 = 13;
	pub const DesiredRunnersUp: u32 = 7;
	/// Daily council elections.
	pub const TermDuration: BlockNumber = 3 * MINUTES;
}

impl Config for Runtime {
	type Event = Event;
	type ModuleId = ElectionsPhragmenModuleId;
	type Currency = Etp;
	type ChangeMembers = Council;
	// NOTE: this implies that council's genesis members cannot be set directly and must come from
	// this module.
	type InitializeMembers = Council;
	type CurrencyToVote = U128CurrencyToVote;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = Treasury;
	type KickedMember = Treasury;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type WeightInfo = SubstrateWeight<Runtime>;
}
