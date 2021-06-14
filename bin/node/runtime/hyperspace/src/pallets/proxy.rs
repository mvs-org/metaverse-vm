// --- crates.io ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::traits::InstanceFilter;
use pallet_proxy::{weights::SubstrateWeight, Config};
use sp_runtime::{traits::BlakeTwo256, RuntimeDebug};
// --- hyperspace ---
use crate::*;

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
pub enum ProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
	EthereumBridge,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				Call::System(..) |
							Call::Babe(..) |
							Call::Timestamp(..) |
							// Specifically omitting the entire Balances pallet
							Call::Authorship(..) |
							Call::Staking(..) |
							Call::Offences(..) |
							Call::Session(..) |
							Call::Grandpa(..) |
							Call::ImOnline(..) |
							Call::AuthorityDiscovery(..) |
							Call::Democracy(..) |
							Call::Council(..) |
							Call::TechnicalCommittee(..) |
							Call::ElectionsPhragmen(..) |
							Call::TechnicalMembership(..) |
							Call::Treasury(..) |
							Call::Sudo(..) |
							// Specifically omitting Vesting `vested_transfer`, and `force_vested_transfer`
							Call::Vesting(hyperspace_vesting::Call::vest(..)) |
							Call::Vesting(hyperspace_vesting::Call::vest_other(..)) |
							Call::Utility(..)|
							Call::Identity(..)|
							Call::Society(..)|
							// Specifically omitting Recovery `create_recovery`, `initiate_recovery`
							Call::Recovery(pallet_recovery::Call::as_recovered(..)) |
							Call::Recovery(pallet_recovery::Call::vouch_recovery(..)) |
							Call::Recovery(pallet_recovery::Call::claim_recovery(..)) |
							Call::Recovery(pallet_recovery::Call::close_recovery(..)) |
							Call::Recovery(pallet_recovery::Call::remove_recovery(..)) |
							Call::Recovery(pallet_recovery::Call::cancel_recovered(..)) |
							Call::Scheduler(..)|
							Call::Proxy(..)|
							Call::Multisig(..)|
							Call::HeaderMMR(..)|
							// Specifically omitting the entire OldetpIssuing pallet
							// Specifically omitting the entire OldetpBacking pallet
							Call::EthereumRelay(..) |
							// Specifically omitting the entire EthereumBacking pallet
							Call::EthereumRelayAuthorities(..) // Specifically omitting the entire OldnaBacking pallet
				                                      // Specifically omitting the entire EVM pallet
				                                      // Specifically omitting the entire Ethereum pallet
			),
			ProxyType::Governance => matches!(
				c,
				Call::Democracy(..)
					| Call::Council(..) | Call::TechnicalCommittee(..)
					| Call::ElectionsPhragmen(..)
					| Call::Treasury(..)
			),
			ProxyType::Staking => matches!(c, Call::Staking(..)),
			ProxyType::EthereumBridge => matches!(
				c,
				Call::EthereumBacking(..)
					| Call::EthereumRelay(..)
					| Call::EthereumRelayAuthorities(..)
			),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}
frame_support::parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = constants::deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = constants::deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = constants::deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = constants::deposit(0, 66);
	pub const MaxPending: u16 = 32;
}
impl Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Etp;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type WeightInfo = SubstrateWeight<Runtime>;
}
