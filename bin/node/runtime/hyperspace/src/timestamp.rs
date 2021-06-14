// --- substrate ---
use pallet_timestamp::{weights::SubstrateWeight, Config};
// --- hyperspace ---
use crate::*;

frame_support::parameter_types! {
	pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}
impl Config for Runtime {
	type Moment = Moment;
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = SubstrateWeight<Runtime>;
}
