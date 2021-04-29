// --- substrate ---
use sp_runtime::ModuleId;
// --- hyperspace ---
use crate::*;
use hyperspace_oldetp_backing::Config;

frame_support::parameter_types! {
	pub const OldetpBackingModuleId: ModuleId = ModuleId(*b"da/oldek");
}
impl Config for Runtime {
	type ModuleId = OldetpBackingModuleId;
	type EtpCurrency = Etp;
	type WeightInfo = ();
}
