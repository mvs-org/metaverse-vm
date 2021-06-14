// --- substrate ---
use sp_runtime::ModuleId;
// --- hyperspace ---
use crate::*;
use hyperspace_oldetp_issuing::Config;

frame_support::parameter_types! {
	pub const OldetpIssuingModuleId: ModuleId = ModuleId(*b"da/crais");
}
impl Config for Runtime {
	type WeightInfo = ();
	type ModuleId = OldetpIssuingModuleId;
	type EtpCurrency = Etp;
}
