use crate::Config;
use hyperspace_evm::{Account as EVMAccount, AccountBasicMapping, AddressMapping};
use frame_support::traits::Currency;
use sp_core::{H160, U256};
use sp_runtime::{traits::UniqueSaturatedInto, SaturatedConversion};

pub struct DVMAccountBasicMapping<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> AccountBasicMapping for DVMAccountBasicMapping<T> {
	/// Get the account basic in EVM format.
	fn account_basic(address: &H160) -> EVMAccount {
		let account_id = <T as hyperspace_evm::Config>::AddressMapping::into_account_id(*address);
		let nonce = frame_system::Module::<T>::account_nonce(&account_id);
		let helper = U256::from(10)
			.checked_pow(U256::from(10))
			.unwrap_or(U256::from(0));

		// Get balance from <T as hyperspace_evm::Config>::EtpCurrency
		let balance: U256 = <T as Config>::EtpCurrency::free_balance(&account_id)
			.saturated_into::<u128>()
			.into();

		// Get remaining balance from dvm
		let remaining_balance: U256 = crate::Module::<T>::remaining_balance(&account_id)
			.saturated_into::<u128>()
			.into();

		// Final balance = balance * 10^9 + remaining_balance
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
		let helper = U256::from(10)
			.checked_pow(U256::from(10))
			.unwrap_or(U256::MAX);
		let existential_deposit: u128 = <T as Config>::EtpCurrency::minimum_balance()
			.saturated_into::<u128>()
			.into();
		let existential_deposit_dvm = U256::from(existential_deposit) * helper;

		let account_id = <T as hyperspace_evm::Config>::AddressMapping::into_account_id(*address);
		let current = T::AccountBasicMapping::account_basic(address);
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

		let nb = new.balance;
		match current.balance {
			cb if cb > nb => {
				let diff = cb - nb;
				let (diff_balance, diff_remaining_balance) = diff.div_mod(helper);
				// If the dvm storage < diff remaining balance, we can not do sub operation directly.
				// Otherwise, slash <T as hyperspace_evm::Config>::EtpCurrency, dec dvm storage balance directly.
				if dvm_balance < diff_remaining_balance {
					let remaining_balance = dvm_balance
						.saturating_add(U256::from(1) * helper)
						.saturating_sub(diff_remaining_balance);

					<T as Config>::EtpCurrency::slash(
						&account_id,
						(diff_balance + 1).low_u128().unique_saturated_into(),
					);
					crate::Module::<T>::set_remaining_balance(
						&account_id,
						remaining_balance.low_u128().saturated_into(),
					);
				} else {
					<T as Config>::EtpCurrency::slash(
						&account_id,
						diff_balance.low_u128().unique_saturated_into(),
					);
					crate::Module::<T>::dec_remaining_balance(
						&account_id,
						diff_remaining_balance.low_u128().saturated_into(),
					);
				}
			}
			cb if cb < nb => {
				let diff = nb - cb;
				let (diff_balance, diff_remaining_balance) = diff.div_mod(helper);

				// If dvm storage balance + diff remaining balance > helper, we must update <T as hyperspace_evm::Config>::EtpCurrency balance.
				if dvm_balance + diff_remaining_balance >= helper {
					let remaining_balance = dvm_balance + diff_remaining_balance - helper;

					<T as Config>::EtpCurrency::deposit_creating(
						&account_id,
						(diff_balance + 1).low_u128().unique_saturated_into(),
					);
					crate::Module::<T>::set_remaining_balance(
						&account_id,
						remaining_balance.low_u128().saturated_into(),
					);
				} else {
					<T as Config>::EtpCurrency::deposit_creating(
						&account_id,
						diff_balance.low_u128().unique_saturated_into(),
					);
					crate::Module::<T>::inc_remaining_balance(
						&account_id,
						diff_remaining_balance.low_u128().saturated_into(),
					);
				}
			}
			_ => return,
		}
		let after_mutate = T::AccountBasicMapping::account_basic(address);
		if after_mutate.balance < existential_deposit_dvm {
			crate::Module::<T>::remove_remaining_balance(&account_id);
		}
	}
}
