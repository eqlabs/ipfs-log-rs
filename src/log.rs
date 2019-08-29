use std::cmp::Ordering;
use crate::entry::Entry;

pub struct Log {
	fn_sort: Box<dyn Fn(&Entry,&Entry) -> Ordering>,
}

impl Log {
	pub fn last_write_wins (a: &Entry, b: &Entry) -> Ordering {
		Log::sort_step_by_step(|_,_| Ordering::Less)(a,b)
	}

	pub fn sort_by_entry_hash (a: &Entry, b: &Entry) -> Ordering {
		Log::sort_step_by_step(|a,b| a.hash().cmp(&b.hash()))(a,b)
	}

	pub fn sort_step_by_step<F: 'static + Fn(&Entry,&Entry) -> Ordering> (resolve: F) -> Box<dyn Fn(&Entry,&Entry) -> Ordering> {
		Box::new(Log::sort_by_clocks(Log::sort_by_clock_ids(resolve)))
	}

	pub fn sort_by_clocks<F: 'static + Fn(&Entry,&Entry) -> Ordering> (resolve: F) -> Box<dyn Fn(&Entry,&Entry) -> Ordering> {
		Box::new(move |a,b| {
			let mut diff = a.clock().cmp(&b.clock());
			if diff == Ordering::Equal {
				diff = resolve(a,b);
			}
			diff
		})
	}

	pub fn sort_by_clock_ids<F: 'static + Fn(&Entry,&Entry) -> Ordering> (resolve: F) -> Box<dyn Fn(&Entry,&Entry) -> Ordering> {
		Box::new(move |a,b| {
			let mut diff = a.clock().id().cmp(&b.clock().id());
			if diff == Ordering::Equal {
				diff = resolve(a,b);
			}
			diff
		})
	}

	pub fn no_zeroes<F: 'static + Fn(&Entry,&Entry) -> Ordering> (fn_sort: F) -> Box<dyn Fn(&Entry,&Entry) -> Ordering> {
		Box::new(move |a,b| {
			let diff = fn_sort(a,b);
			if diff == Ordering::Equal {
				panic!("Your log's tiebreaker function {}",
				"has returned zero and therefore cannot be");
			}
			diff
		})
	}
}
