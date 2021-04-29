// --- substrate ---
use pallet_im_online::{sr25519::AuthorityId, weights::SubstrateWeight, Config};
use sp_runtime::transaction_validity::TransactionPriority;
// --- hyperspace ---
use crate::*;

frame_support::parameter_types! {
	pub const SessionDuration: BlockNumber = BLOCKS_PER_SESSION as _;
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
}
impl Config for Runtime {
	type AuthorityId = AuthorityId;
	type Event = Event;
	type SessionDuration = SessionDuration;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = SubstrateWeight<Runtime>;
}
