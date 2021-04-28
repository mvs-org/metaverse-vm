// --- substrate ---
use frame_support::traits::KeyOwnerProofSystem;
use pallet_grandpa::{AuthorityId, Config, EquivocationHandler};
use sp_core::crypto::KeyTypeId;
// --- hyperspace ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type Call = Call;
	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, AuthorityId)>>::Proof;
	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		AuthorityId,
	)>>::IdentificationTuple;
	type KeyOwnerProofSystem = Historical;
	type HandleEquivocation = EquivocationHandler<Self::KeyOwnerIdentification, Offences>;
	type WeightInfo = ();
}
