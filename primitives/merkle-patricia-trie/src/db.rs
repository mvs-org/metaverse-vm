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

use hashbrown::HashMap;
use sp_std::{cell::RefCell, prelude::*};

#[derive(Debug)]
pub struct MemoryDB {
	data: RefCell<HashMap<Vec<u8>, Vec<u8>>>,
}

impl MemoryDB {
	pub fn new() -> Self {
		MemoryDB {
			data: RefCell::new(HashMap::new()),
		}
	}

	pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		let data = self.data.borrow();
		if let Some(d) = data.get(key) {
			Some(d.clone())
		} else {
			None
		}
	}

	pub fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Option<Vec<u8>> {
		self.data.borrow_mut().insert(key, value)
	}

	pub fn contains(&self, key: &[u8]) -> bool {
		self.data.borrow().contains_key(key)
	}

	pub fn remove(&self, key: &[u8]) -> Option<Vec<u8>> {
		self.data.borrow_mut().remove(key)
	}

	/// Insert a batch of data into the cache.
	pub fn insert_batch(&self, keys: Vec<Vec<u8>>, values: Vec<Vec<u8>>) {
		for i in 0..keys.len() {
			let key = keys[i].clone();
			let value = values[i].clone();
			self.insert(key, value);
		}
	}

	/// Remove a batch of data into the cache.
	pub fn remove_batch(&self, keys: &[Vec<u8>]) {
		for key in keys {
			self.remove(key);
		}
	}
}
