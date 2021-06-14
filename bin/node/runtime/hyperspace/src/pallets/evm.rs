// --- substrate ---
use sp_core::U256;
// --- hyperspace ---
use crate::*;
use hyperspace_evm::{
	runner::stack::Runner, ConcatAddressMapping, Config, EnsureAddressTruncated, FeeCalculator,
};
use hyperspace_evm_precompile::HyperspacePrecompiles;
use dvm_ethereum::account_basic::DvmAccountBasic;
use dvm_ethereum::account_basic::{DnaRemainBalance, EtpRemainBalance};

/// Fixed gas price.
pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> U256 {
		// Gas price is always one token per gas.
		10_000_000_000u64.into()
	}
}
frame_support::parameter_types! {
	pub const ChainId: u64 = 23;
	pub BlockGasLimit: U256 = U256::from(u32::max_value());
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
	type BlockGasLimit = BlockGasLimit;
	type EtpAccountBasic = DvmAccountBasic<Self, Etp, EtpRemainBalance>;
	type DnaAccountBasic = DvmAccountBasic<Self, Dna, DnaRemainBalance>;
	type Runner = Runner<Self>;
	type IssuingHandler = EthereumIssuing;
}
