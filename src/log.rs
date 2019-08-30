use std::collections::HashMap;
use std::cmp::Ordering;
use std::cmp::max;
use crate::entry::Entry;
use crate::entry::Data;
use crate::entry::EntryOrHash;
use crate::identity::Identity;
use crate::lamport_clock::LamportClock;

pub struct Log {
	id: String,
	identity: Identity,
	access: AdHocAccess,
	entries: HashMap<String,Entry>,
	length: usize,
	heads: Vec<String>,
	nexts: HashMap<String,String>,
	fn_sort: Box<dyn Fn(&Entry,&Entry) -> Ordering>,
	clock: LamportClock,
}

impl Log {
	pub fn get (&self, hash: &String) -> Option<&Entry> {
		self.entries.get(hash)
	}

	pub fn traverse<'a> (&'a self, roots: Vec<&'a Entry>, amount: Option<usize>, end_hash: Option<String>) -> HashMap<String,&'a Entry> {
		let mut stack = roots;
		stack.sort_by(|a,b| (self.fn_sort)(a,b));
		stack.reverse();
		let mut traversed: HashMap<String,bool> = HashMap::new();
		let mut result: HashMap<String,&Entry> = HashMap::new();
		let mut count = 0;

		while !stack.is_empty() && (amount.is_none() || count < amount.unwrap()) {
			let e = stack.remove(0);
			let hash = &e.hash().to_owned();
			count += 1;
			for h in e.next() {
				if let Some(e) = self.get(h) {
					if !traversed[e.hash()] {
						let hash = e.hash().to_owned();
						stack.insert(0,e);
						stack.sort_by(|a,b| (self.fn_sort)(a,b));
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

	pub fn append (&mut self, data: Data, n_ptr: Option<usize>) -> &Entry {
		let mut t_new = self.clock.time();
		for h in &self.heads {
			t_new = max(t_new,self.get(&h).unwrap().clock().time());
		}
		t_new = t_new + 1;
		self.clock = LamportClock::new(self.clock.id().clone()).set_time(t_new);

		let mut heads = Vec::new();
		for h in &self.heads {
			heads.push(self.get(&h).unwrap());
		}
		let refs = self.traverse(heads,Some(max(n_ptr.unwrap_or(1),self.heads.len())),None);
		let mut keys = Vec::new();
		for k in refs {
			keys.push(k.0);
		}
		//i)	why reverse?
		//ii)	does it need to be deduped, like in the original JS version?
		self.heads.reverse();
		self.heads.append(&mut keys);
		let mut hashes = Vec::new();
		for s in &self.heads {
			hashes.push(EntryOrHash::Hash(s.to_owned()));
		}

		//should be created asynchronically in IPFS
		let entry = Entry::new(self.identity.clone(),&self.id,data,&hashes,Some(self.clock.clone()));
		//should be queried asynchronically
		if !self.access.can_access(&entry) {
			panic!("Could not append entry, key \"{}\" is not allowed to write in the log",
			self.identity.id());
		}

		let eh = entry.hash().to_owned();
		self.entries.insert(eh.to_owned(),entry);
		for h in hashes {
			match h {
				EntryOrHash::Hash(h)	=>	{
												self.nexts.insert(h.to_owned(),eh.to_owned());
											},
				_						=>	unreachable!(),
			}
		}
		self.heads.clear();
		self.heads.push(eh.to_owned());
		self.length += 1;

		&self.entries[&eh]
	}

	pub fn last_write_wins (a: &Entry, b: &Entry) -> Ordering {
		Log::sort_step_by_step(|_,_| Ordering::Less)(a,b)
	}

	pub fn sort_by_entry_hash (a: &Entry, b: &Entry) -> Ordering {
		Log::sort_step_by_step(|a,b| a.hash().cmp(&b.hash()))(a,b)
	}

	pub fn sort_step_by_step<F> (resolve: F) -> Box<dyn Fn(&Entry,&Entry) -> Ordering>
	where F: 'static + Fn(&Entry,&Entry) -> Ordering {
		Box::new(Log::sort_by_clocks(Log::sort_by_clock_ids(resolve)))
	}

	pub fn sort_by_clocks<F> (resolve: F) -> Box<dyn Fn(&Entry,&Entry) -> Ordering>
	where F: 'static + Fn(&Entry,&Entry) -> Ordering {
		Box::new(move |a,b| {
			let mut diff = a.clock().cmp(&b.clock());
			if diff == Ordering::Equal {
				diff = resolve(a,b);
			}
			diff
		})
	}

	pub fn sort_by_clock_ids<F> (resolve: F) -> Box<dyn Fn(&Entry,&Entry) -> Ordering>
	where F: 'static + Fn(&Entry,&Entry) -> Ordering {
		Box::new(move |a,b| {
			let mut diff = a.clock().id().cmp(&b.clock().id());
			if diff == Ordering::Equal {
				diff = resolve(a,b);
			}
			diff
		})
	}

	pub fn no_zeroes<F> (fn_sort: F) -> Box<dyn Fn(&Entry,&Entry) -> Ordering>
	where F: 'static + Fn(&Entry,&Entry) -> Ordering {
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

struct AdHocAccess;

impl AdHocAccess {
	fn can_access (&self, entry: &Entry) -> bool {
		true
	}
}
