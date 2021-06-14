// --- substrate ---
use sp_runtime::ModuleId;
// --- hyperspace ---
use crate::*;
use hyperspace_ethereum_issuing::Config;

frame_support::parameter_types! {
	pub const EthereumIssuingModuleId: ModuleId = ModuleId(*b"da/ethis");
}

impl Config for Runtime {
	type ModuleId = EthereumIssuingModuleId;
	type Event = Event;
	type EthereumRelay = EthereumRelay;
	type EtpCurrency = Etp;
	type EcdsaAuthorities = EthereumRelayAuthorities;
	type WeightInfo = ();
}
