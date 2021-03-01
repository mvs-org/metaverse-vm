//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0-rc6

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

impl crate::WeightInfo for () {
	fn vest_locked(l: u32) -> Weight {
		(82109000 as Weight)
			.saturating_add((332000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn vest_unlocked(l: u32) -> Weight {
		(88419000 as Weight)
			.saturating_add((3000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn vest_other_locked(l: u32) -> Weight {
		(81277000 as Weight)
			.saturating_add((321000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn vest_other_unlocked(l: u32) -> Weight {
		(87584000 as Weight)
			.saturating_add((19000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn vested_transfer(l: u32) -> Weight {
		(185916000 as Weight)
			.saturating_add((625000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn force_vested_transfer(l: u32) -> Weight {
		(185916000 as Weight)
			.saturating_add((625000 as Weight).saturating_mul(l as Weight))
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
}
