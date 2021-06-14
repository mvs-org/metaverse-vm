// --- substrate ---
use pallet_authorship::Config;
use pallet_session::FindAccountFromAuthorIndex;
// --- hyperspace ---
use crate::*;

frame_support::parameter_types! {
	pub const UncleGenerations: BlockNumber = 5;
}
impl Config for Runtime {
	type FindAuthor = FindAccountFromAuthorIndex<Self, Babe>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (Staking, ImOnline);
}
