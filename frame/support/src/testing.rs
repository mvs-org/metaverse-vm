// This file is part of Hyperspace.
//
// Copyright (C) 2018-2021 Hyperspace Network
// SPDX-License-Identifier: GPL-3.0
//
// Hyperspace is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Hyperspace is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Hyperspace. If not, see <https://www.gnu.org/licenses/>.

#[macro_export]
macro_rules! impl_test_account_data {
	() => {
		pub type EtpInstance = hyperspace_balances::Instance0;
		pub type EtpError = hyperspace_balances::Error<Test, EtpInstance>;
		pub type DnaInstance = hyperspace_balances::Instance1;
		pub type DnaError = hyperspace_balances::Error<Test, DnaInstance>;

		$crate::impl_account_data! {
			struct AccountData<Balance>
			for
				EtpInstance,
				DnaInstance
			where
				Balance = Balance
			{}
		}
	};
	(deprecated) => {
		pub type EtpInstance = hyperspace_balances::Instance0;
		pub type EtpError = hyperspace_balances::Error<Test, EtpInstance>;
		pub type EtpConfig = hyperspace_balances::GenesisConfig<Test, EtpInstance>;
		pub type Etp = hyperspace_balances::Module<Test, EtpInstance>;
		pub type DnaInstance = hyperspace_balances::Instance1;
		pub type DnaError = hyperspace_balances::Error<Test, DnaInstance>;
		pub type DnaConfig = hyperspace_balances::GenesisConfig<Test, DnaInstance>;
		pub type Dna = hyperspace_balances::Module<Test, DnaInstance>;

		$crate::impl_account_data! {
			struct AccountData<Balance>
			for
				EtpInstance,
				DnaInstance
			where
				Balance = Balance
			{}
		}
	};
}
