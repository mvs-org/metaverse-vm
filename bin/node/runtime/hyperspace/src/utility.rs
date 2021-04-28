// --- substrate ---
use pallet_utility::Config;
// --- hyperspace ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}
