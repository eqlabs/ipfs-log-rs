#![allow(dead_code)]

mod log;
mod gset;
mod identity;
mod lamport_clock;
mod entry;

#[allow(unused_imports)]
use gset::GSet;
#[allow(unused_imports)]
use lamport_clock::LamportClock;
#[allow(unused_imports)]
use identity::IdentityBuilder;
#[allow(unused_imports)]
use log::Log;
#[allow(unused_imports)]
use entry::Entry;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_gset () {
		let mut x: GSet<i32> = GSet::new();
		assert!(x.is_empty());
		x.insert(2);
		x.insert(3);
		x.insert(5);
		x.insert(8);
		assert!(!x.is_empty());
		assert_eq!(x.len(),4);
		let mut y: GSet<i32> = GSet::new();
		y.insert(4);
		y.insert(5);
		y.insert(10);
		y.insert(12);
		assert!(!x.is_subset(&y));
		assert!(!y.is_subset(&x));
		let z = GSet::union(&x,&y);
		assert_eq!(z.len(),7);
		let mut w = GSet::new();
		w.insert(2);
		w.insert(4);
		w.insert(8);
		assert!(w.is_subset(&z));
		assert!(!z.is_subset(&w));
	}

	#[test]
	fn test_clock () {
		let mut ib = IdentityBuilder::new();
		let mut x = LamportClock::new(ib.build());
		let y = LamportClock::new(ib.build());
		let mut z = LamportClock::new(ib.build());
		assert!(x < y);
		assert!(y < z);
		z.tick();
		x.merge(&z);
		assert!(x > y);
		let w = LamportClock::new(ib.build()).set_time(4);
		assert!(x < w);
		for _ in 0..3 {
			x.tick();
		}
		assert!(x < w);
		x.tick();
		assert!(x > w);
	}
}
