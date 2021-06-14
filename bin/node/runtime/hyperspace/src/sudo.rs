// --- substrate ---
use pallet_sudo::Config;
// --- hyperspace ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type Call = Call;
}
