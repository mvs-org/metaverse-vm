// --- substrate ---
use sp_runtime::{ModuleId, Perbill};
// --- hyperspace ---
use crate::*;
use hyperspace_ethereum_relay::Config;
use ethereum_primitives::EthereumNetworkType;

frame_support::parameter_types! {
	pub const EthereumRelayModuleId: ModuleId = ModuleId(*b"da/ethrl");
	pub const EthereumNetwork: EthereumNetworkType = EthereumNetworkType::Ropsten;
	pub const ConfirmPeriod: BlockNumber = 30;
	pub const ApproveThreshold: Perbill = Perbill::from_percent(60);
	pub const RejectThreshold: Perbill = Perbill::from_percent(1);
}
impl Config for Runtime {
	type ModuleId = EthereumRelayModuleId;
	type Event = Event;
	type EthereumNetwork = EthereumNetwork;
	type Call = Call;
	type Currency = Etp;
	type RelayerGame = EthereumRelayerGame;
	type ApproveOrigin = ApproveOrigin;
	type RejectOrigin = EnsureRootOrHalfTechnicalComittee;
	type ConfirmPeriod = ConfirmPeriod;
	type TechnicalMembership = TechnicalMembership;
	type ApproveThreshold = ApproveThreshold;
	type RejectThreshold = RejectThreshold;
	type WeightInfo = ();
}
