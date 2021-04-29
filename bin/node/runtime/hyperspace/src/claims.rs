// --- substrate ---
use sp_runtime::ModuleId;
// --- hyperspace ---
use crate::*;
use hyperspace_claims::Config;

frame_support::parameter_types! {
	pub const ClaimsModuleId: ModuleId = ModuleId(*b"da/claim");
	pub Prefix: &'static [u8] = b"Pay PETPs to the Hyperspace account:";
}
impl Config for Runtime {
	type Event = Event;
	type ModuleId = ClaimsModuleId;
	type Prefix = Prefix;
	type EtpCurrency = Etp;
	type MoveClaimOrigin = EnsureRootOrMoreThanHalfCouncil;
}
