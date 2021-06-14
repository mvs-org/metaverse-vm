// --- substrate ---
use sp_core::U256;
// --- hyperspace ---
use crate::*;
use hyperspace_evm::{
	runner::stack::Runner, ConcatAddressMapping, Config, EnsureAddressTruncated, FeeCalculator,
};
use hyperspace_evm_precompile::HyperspacePrecompiles;
use dvm_ethereum::account_basic::DVMAccountBasicMapping;

/// Fixed gas price of `1`.
pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> U256 {
		// Gas price is always one token per gas.
		10_000_000_000u64.into()
	}
}
frame_support::parameter_types! {
	pub const ChainId: u64 mc= 23;
}
impl Config for Runtime {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressTruncated;
	type WithdrawOrigin = EnsureAddressTruncated;
	type AddressMapping = ConcatAddressMapping;
	type EtpCurrency = Etp;
	type DnaCurrency = Dna;
	type Event = Event;
	type Precompiles = HyperspacePrecompiles<Self>;
	type ChainId = ChainId;
	type AccountBasicMapping = DVMAccountBasicMapping<Self>;
	type Runner = Runner<Self>;
}
