// --- substrate ---
use sp_runtime::ModuleId;
// --- hyperspace ---
use crate::*;
use hyperspace_ethereum_backing::Config;

frame_support::parameter_types! {
	pub const EthereumBackingModuleId: ModuleId = ModuleId(*b"da/ethbk");
	pub const EthereumBackingFeeModuleId: ModuleId = ModuleId(*b"da/ethfe");
	pub const EtpLockLimit: Balance = 10_000_000 * COIN;
	pub const DnaLockLimit: Balance = 1000 * COIN;
	pub const AdvancedFee: Balance = 50 * COIN;
	pub const SyncReward: Balance = 1000 * COIN;
}
impl Config for Runtime {
	type ModuleId = EthereumBackingModuleId;
	type FeeModuleId = EthereumBackingFeeModuleId;
	type Event = Event;
	type RedeemAccountId = AccountId;
	type EthereumRelay = EthereumRelay;
	type OnDepositRedeem = Staking;
	type EtpCurrency = Etp;
	type DnaCurrency = Dna;
	type EtpLockLimit = EtpLockLimit;
	type DnaLockLimit = DnaLockLimit;
	type AdvancedFee = AdvancedFee;
	type SyncReward = SyncReward;
	type EcdsaAuthorities = EthereumRelayAuthorities;
	type WeightInfo = ();
}
