// --- substrate ---
use pallet_im_online::{sr25519::AuthorityId, weights::SubstrateWeight, Config};
use sp_runtime::transaction_validity::TransactionPriority;
// --- hyperspace ---
use crate::*;

frame_support::parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
}
impl Config for Runtime {
	type AuthorityId = AuthorityId;
	type Event = Event;
	type NextSessionRotation = Babe;
	type ValidatorSet = Historical;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = SubstrateWeight<Runtime>;
}
