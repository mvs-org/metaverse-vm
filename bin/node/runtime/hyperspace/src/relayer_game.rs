// --- hyperspace ---
pub use hyperspace_relayer_game::Instance0 as EthereumRelayerGameInstance;

// --- substrate ---
use frame_support::traits::LockIdentifier;
// --- hyperspace ---
use crate::*;
use hyperspace_relayer_game::Config;

frame_support::parameter_types! {
	pub const EthereumRelayerGameLockId: LockIdentifier = *b"ethrgame";
}
impl Config<EthereumRelayerGameInstance> for Runtime {
	type EtpCurrency = Etp;
	type LockId = EthereumRelayerGameLockId;
	type EtpSlash = Treasury;
	type RelayerGameAdjustor = relay::EthereumRelayerGameAdjustor;
	type RelayableChain = EthereumRelay;
	type WeightInfo = ();
}
