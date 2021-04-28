// --- substrate ---
use sp_runtime::ModuleId;
// --- hyperspace ---
use crate::*;
use hyperspace_oldna_backing::Config;

frame_support::parameter_types! {
	pub const OldnaBackingModuleId: ModuleId = ModuleId(*b"da/trobk");
}
impl Config for Runtime {
	type ModuleId = OldnaBackingModuleId;
	type EtpCurrency = Etp;
	type DnaCurrency = Dna;
	type WeightInfo = ();
}
