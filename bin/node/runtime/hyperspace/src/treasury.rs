// --- substrate ---
use sp_runtime::{ModuleId, Percent, Permill};
// --- hyperspace ---
use crate::*;
use hyperspace_treasury::{weights::SubstrateWeight, Config};

frame_support::parameter_types! {
	pub const TreasuryModuleId: ModuleId = ModuleId(*b"da/trsry");
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const EtpProposalBondMinimum: Balance = 20 * COIN;
	pub const DnaProposalBondMinimum: Balance = 20 * COIN;
	pub const SpendPeriod: BlockNumber = 3 * MINUTES;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TipCountdown: BlockNumber = 3 * MINUTES;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * COIN;
	pub const DataDepositPerByte: Balance = 1 * MILLI;
	pub const BountyDepositBase: Balance = 1 * COIN;
	pub const BountyDepositPayoutDelay: BlockNumber = 3 * MINUTES;
	pub const BountyUpdatePeriod: BlockNumber = 3 * MINUTES;
	pub const MaximumReasonLength: u32 = 16384;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 2 * COIN;
}
impl Config for Runtime {
	type ModuleId = TreasuryModuleId;
	type EtpCurrency = Etp;
	type DnaCurrency = Dna;
	type ApproveOrigin = ApproveOrigin;
	type RejectOrigin = EnsureRootOrMoreThanHalfCouncil;
	type Tippers = ElectionsPhragmen;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type DataDepositPerByte = DataDepositPerByte;
	type Event = Event;
	type OnSlashEtp = Treasury;
	type OnSlashDna = Treasury;
	type ProposalBond = ProposalBond;
	type EtpProposalBondMinimum = EtpProposalBondMinimum;
	type DnaProposalBondMinimum = DnaProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type MaximumReasonLength = MaximumReasonLength;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type EtpBurnDestination = ();
	type DnaBurnDestination = ();
	type WeightInfo = SubstrateWeight<Runtime>;
}
