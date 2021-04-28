// --- substrate ---
pub use pallet_session::historical as pallet_session_historical;

// --- substrate ---
use pallet_session_historical::Config;
// --- hyperspace ---
use crate::*;
use hyperspace_staking::{Exposure, ExposureOf};

impl Config for Runtime {
	type FullIdentification = Exposure<AccountId, Balance, Balance>;
	type FullIdentificationOf = ExposureOf<Runtime>;
}
