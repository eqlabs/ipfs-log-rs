use std::cmp::Ordering;
use crate::entry::Entry;

pub struct Log {
	fn_sort: Box<dyn Fn(&Entry,&Entry) -> Ordering>,
}

impl Log {
	pub fn sort_step_by_step<F: Fn(&Entry,&Entry) -> Ordering> (a: &Entry, b: &Entry,
	resolve: F) -> Ordering {
		Log::sort_by_clocks(a,b,|a,b| Log::sort_by_clock_ids(a,b,&resolve))
	}

	pub fn last_write_wins (a: &Entry, b: &Entry) -> Ordering {
		Log::sort_step_by_step(a,b,|_,_| Ordering::Less)
	}

	pub fn sort_by_entry_hash (a: &Entry, b: &Entry) -> Ordering {
		Log::sort_step_by_step(a,b,|a,b| a.hash().cmp(&b.hash()))
	}

	pub fn sort_by_clocks<F: Fn(&Entry,&Entry) -> Ordering> (a: &Entry, b: &Entry,
	resolve: F) -> Ordering {
		let mut diff = a.clock().cmp(&b.clock());
		if diff == Ordering::Equal {
			diff = resolve(a,b);
		}
		diff
	}

	pub fn sort_by_clock_ids<F: Fn(&Entry,&Entry) -> Ordering> (a: &Entry, b: &Entry,
	resolve: F) -> Ordering {
		let mut diff = a.clock().id().cmp(&b.clock().id());
		if diff == Ordering::Equal {
			diff = resolve(a,b);
		}
		diff
	}

	pub fn no_zeroes<'a,F: 'static + Fn(&Entry,&Entry) -> Ordering> (fn_sort: F) -> Box<dyn Fn(&'a Entry,&'a Entry) -> Ordering> {
		let comparator = move |a,b| {
			let diff = fn_sort(a,b);
			if diff == Ordering::Equal {
				panic!("Your log's tiebreaker function {}",
				"has returned zero and therefore cannot be");
			}
			diff
		};
		Box::new(comparator)
	}
}
