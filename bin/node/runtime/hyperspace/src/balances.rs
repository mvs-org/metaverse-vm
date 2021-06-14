// --- hyperspace ---
pub use hyperspace_balances::{Instance0 as EtpInstance, Instance1 as DnaInstance};

// --- substrate ---
use frame_support::traits::Currency;
use frame_system::Config as SystemConfig;
// --- hyperspace ---
use crate::*;
use hyperspace_balances::{weights::SubstrateWeight, Config, Module};

pub type NegativeImbalance = <Module<Runtime, EtpInstance> as Currency<
	<Runtime as SystemConfig>::AccountId,
>>::NegativeImbalance;

frame_support::parameter_types! {
	pub const ExistentialDeposit: Balance = 0;
	pub const MaxLocks: u32 = 50;
}
impl Config<EtpInstance> for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type OtherCurrencies = (Dna,);
	type WeightInfo = SubstrateWeight<Runtime>;
}
impl Config<DnaInstance> for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type OtherCurrencies = (Etp,);
	type WeightInfo = SubstrateWeight<Runtime>;
}
