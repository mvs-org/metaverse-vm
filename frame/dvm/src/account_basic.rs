use hyperspace_evm::{Account as EVMAccount, AccountBasicMapping, AddressMapping};
use frame_support::traits::Currency;
use sp_core::{H160, U256};
use sp_runtime::{
	traits::{UniqueSaturatedFrom, UniqueSaturatedInto},
	SaturatedConversion,
};

pub struct DVMAccountBasicMapping<T>(sp_std::marker::PhantomData<T>);

impl<T: crate::Config + hyperspace_balances::Config<hyperspace_balances::Instance0>> AccountBasicMapping
	for DVMAccountBasicMapping<T>
{
	/// Get the account basic in EVM format.
	fn account_basic(address: &H160) -> EVMAccount {
		let account_id = <T as hyperspace_evm::Config>::AddressMapping::into_account_id(*address);
		let nonce = frame_system::Module::<T>::account_nonce(&account_id);
		let helper = U256::from(10)
			.checked_pow(U256::from(10))
			.unwrap_or(U256::from(0));

		// Get balance from <T as hyperspace_evm::Config>::EtpCurrency
		let balance: U256 = <T as hyperspace_evm::Config>::EtpCurrency::free_balance(&account_id)
			.saturated_into::<u128>()
			.into();

		// Get remaining balance from dvm
		let remaining_balance: U256 = crate::Module::<T>::remaining_balance(&account_id)
			.saturated_into::<u128>()
			.into();

		// Final balance = balance * 10^8 + remaining_balance
		let final_balance = U256::from(balance * helper)
			.checked_add(remaining_balance)
			.unwrap_or_default();

		EVMAccount {
			nonce: nonce.saturated_into::<u128>().into(),
			balance: final_balance,
		}
	}

	/// Mutate the basic account
	fn mutate_account_basic(address: &H160, new: EVMAccount) {
		let account_id = <T as hyperspace_evm::Config>::AddressMapping::into_account_id(*address);
		let current = T::AccountBasicMapping::account_basic(address);
		let helper = U256::from(10)
			.checked_pow(U256::from(10))
			.unwrap_or(U256::MAX);
		let dvm_balance: U256 = crate::Module::<T>::remaining_balance(&account_id)
			.saturated_into::<u128>()
			.into();

		if current.nonce < new.nonce {
			// ASSUME: in one single EVM transaction, the nonce will not increase more than
			// `u128::max_value()`.
			for _ in 0..(new.nonce - current.nonce).low_u128() {
				frame_system::Module::<T>::inc_account_nonce(&account_id);
			}
		}

		if current.balance > new.balance {
			let diff = current.balance - new.balance;
			let (diff_balance, diff_remaining_balance) = diff.div_mod(helper);
			// If the dvm storage < diff remaining balance, we can not do sub operation directly.
			// Otherwise, slash <T as hyperspace_evm::Config>::EtpCurrency, dec dvm storage balance directly.
			if dvm_balance < diff_remaining_balance {
				let remaining_balance = dvm_balance
					.saturating_add(U256::from(1) * helper)
					.saturating_sub(diff_remaining_balance);

				<T as hyperspace_evm::Config>::EtpCurrency::slash(
					&account_id,
					(diff_balance + 1).low_u128().unique_saturated_into(),
				);
				let value = <T as hyperspace_balances::Config<hyperspace_balances::Instance0>>::Balance::unique_saturated_from(
					remaining_balance.low_u128(),
				);
				crate::Module::<T>::set_remaining_balance(&account_id, value);
			} else {
				<T as hyperspace_evm::Config>::EtpCurrency::slash(
					&account_id,
					diff_balance.low_u128().unique_saturated_into(),
				);
				let value = <T as hyperspace_balances::Config<hyperspace_balances::Instance0>>::Balance::unique_saturated_from(
					diff_remaining_balance.low_u128(),
				);
				crate::Module::<T>::dec_remain_balance(&account_id, value);
			}
		} else if current.balance < new.balance {
			let diff = new.balance - current.balance;
			let (diff_balance, diff_remaining_balance) = diff.div_mod(helper);

			// If dvm storage balance + diff remaining balance > helper, we must update <T as hyperspace_evm::Config>::EtpCurrency balance.
			if dvm_balance + diff_remaining_balance >= helper {
				let remaining_balance = dvm_balance + diff_remaining_balance - helper;

				<T as hyperspace_evm::Config>::EtpCurrency::deposit_creating(
					&account_id,
					(diff_balance + 1).low_u128().unique_saturated_into(),
				);
				let value = <T as hyperspace_balances::Config<hyperspace_balances::Instance0>>::Balance::unique_saturated_from(
					remaining_balance.low_u128(),
				);
				crate::Module::<T>::set_remaining_balance(&account_id, value);
			} else {
				<T as hyperspace_evm::Config>::EtpCurrency::deposit_creating(
					&account_id,
					diff_balance.low_u128().unique_saturated_into(),
				);
				let value = <T as hyperspace_balances::Config<hyperspace_balances::Instance0>>::Balance::unique_saturated_from(
					diff_remaining_balance.low_u128(),
				);
				crate::Module::<T>::inc_remain_balance(&account_id, value);
			}
		}
	}
}
