use std::collections::HashMap;
use std::cmp::Ordering;
use crate::entry::Entry;

pub struct Log {
	fn_sort: Box<dyn Fn(&Entry,&Entry) -> Ordering>,
}

impl Log {
	//very much ad hoc
	pub fn get (&self, hash: &String) -> Option<Entry> {
		Some(Entry::empty())
	}

	pub fn traverse (&self, roots: Vec<Entry>, amount: Option<usize>, end_hash: Option<String>) -> HashMap<String,Entry> {
		let mut stack = roots;
		stack.sort_by(&self.fn_sort);
		stack.reverse();
		let mut traversed: HashMap<String,bool> = HashMap::new();
		let mut result: HashMap<String,Entry> = HashMap::new();
		let mut count = 0;

		while !stack.is_empty() && (amount.is_none() || count < amount.unwrap()) {
			let e = stack.remove(0);
			let hash = &e.hash().to_owned();
			count += 1;
			for h in e.next() {
				let x = self.get(h);
				if x.is_some() {
					let e = x.unwrap();
					if !traversed[e.hash()] {
						let hash = e.hash().to_owned();
						stack.insert(0,e);
						stack.sort_by(&self.fn_sort);
						stack.reverse();
						traversed.insert(hash,true);
					}
				}
			}
			result.insert(e.hash().to_owned(),e);

			if let Some(ref eh) = end_hash {
				if eh == hash {
					break;
				}
			}
		}

		result
	}

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
