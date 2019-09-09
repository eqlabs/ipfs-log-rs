use std::collections::HashMap;
use std::collections::HashSet;
use std::cmp::Ordering;
use std::cmp::max;
use std::time::SystemTime;
use std::rc::Rc;
use serde_json::json;
use crate::entry::Entry;
use crate::entry::EntryOrHash;
use crate::identity::Identity;
use crate::lamport_clock::LamportClock;

pub struct Log {
	id: String,
	identity: Identity,
	access: AdHocAccess,
	entries: HashMap<String,Rc<Entry>>,
	length: usize,
	heads: Vec<Rc<Entry>>,
	nexts: HashSet<String>,
	fn_sort: Box<dyn Fn(&Entry,&Entry) -> Ordering>,
	clock: LamportClock,
}

impl Log {
	pub fn new (identity: Identity, id: Option<&str>, access: AdHocAccess,
	entries: &[Rc<Entry>], heads: &[Rc<Entry>], clock: Option<LamportClock>,
	fn_sort: Option<Box<dyn Fn(&Entry,&Entry) -> Ordering>>) -> Log {
		let fn_sort = Log::no_zeroes(fn_sort.unwrap_or(Box::new(Log::last_write_wins)));
		let id = if let Some(s) = id {
			s.to_owned()
		}
		else {
			SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string()
		};
		let length = entries.len();

		let heads = Log::dedup(&if heads.is_empty() {
			Log::find_heads(&entries)
		}
		else {
			heads.to_owned()
		});

		let mut nexts = HashSet::new();
		for e in entries {
			for n in e.next() {
				nexts.insert(n.to_owned());
			}
		}

		let mut entry_set = HashMap::new();
		for e in entries {
			entry_set.insert(e.hash().to_owned(),e.clone());
		}

		let mut t_max = 0;
		if let Some(c) = clock {
			t_max = c.time();
		}
		for h in &heads {
			t_max = max(t_max,h.clock().time());
		}
		let clock = LamportClock::new(identity.public_key()).set_time(t_max);

		Log {
			id: id,
			identity: identity,
			access: access,
			entries: entry_set,
			length: length,
			heads: heads,
			nexts: nexts,
			fn_sort: fn_sort,
			clock: clock,
		}
	}

	pub fn find_heads (entries: &[Rc<Entry>]) -> Vec<Rc<Entry>> {
		let mut parents = HashMap::<&str,&str>::new();
		for e in entries {
			for n in e.next() {
				parents.insert(n,e.hash());
			}
		}
		let mut heads = Vec::new();
		for e in entries {
			if !parents.contains_key(e.hash()) {
				heads.push(e.clone());
			}
		}
		//inequality correct?
		heads.sort_by(|a,b| b.clock().id().cmp(a.clock().id()));
		heads
	}

	fn dedup (v: &[Rc<Entry>]) -> Vec<Rc<Entry>> {
		let mut s = HashSet::new();
		v.iter().filter(|x| s.insert(x.hash())).map(|x| x.clone()).collect()
	}

	pub fn has (&self, hash: &str) -> bool {
		self.entries.contains_key(hash)
	}

	pub fn get (&self, hash: &str) -> Option<&Rc<Entry>> {
		self.entries.get(hash)
	}

	pub fn values (&self) -> Vec<Rc<Entry>> {
		let mut es = self.traverse(&self.heads,None,None);
		es.reverse();
		es
	}

	pub fn all (&self) -> String {
		let mut s = String::from("[ ");
		for e in &self.entries {
			if self.heads.iter().any(|x| x.hash() == e.1.hash()) {
				s.push_str("^");
			}
			s.push_str(e.0);
			s.push_str(", ");
		}
		s = String::from(&s[..s.len() - 2]);
		s.push_str(" ]");
		s
	}

	pub fn entries (&self) -> String {
		let mut s = String::new();
		for e in &self.entries {
			s.push_str(e.0);
			if !e.1.next().is_empty() {
				s.push_str("\t\t>");
				s.push_str(&e.1.next()[0]);
				s.push_str(", >");
				s.push_str(&e.1.next()[1]);
			}
			else {
				s.push_str("\t\t.,.");
			}
			s.push_str("\n");
		}
		s
	}

	pub fn traverse<'a> (&'a self, roots: &[Rc<Entry>], amount: Option<usize>, end_hash: Option<String>) -> Vec<Rc<Entry>> {
		let mut stack = roots.to_owned();
		stack.sort_by(|a,b| (self.fn_sort)(a,b));
		stack.reverse();
		let mut traversed = HashSet::<&str>::new();
		let mut result = Vec::new();
		let mut count = 0;

		while !stack.is_empty() && (amount.is_none() || count < amount.unwrap()) {
			let e = stack.remove(0);
			let hash = e.hash().to_owned();
			count += 1;
			for h in e.next() {
				if let Some(e) = self.get(h) {
					if !traversed.contains(e.hash()) {
						stack.insert(0,e.clone());
						stack.sort_by(|a,b| (self.fn_sort)(a,b));
						stack.reverse();
						traversed.insert(e.hash());
					}
				}
			}
			result.push(e);

			if let Some(ref eh) = end_hash {
				if eh == &hash {
					break;
				}
			}
		}

		result
	}

	pub fn append (&mut self, data: &str, n_ptr: Option<usize>) -> &Entry {
		let mut t_new = self.clock.time();
		for h in &self.heads {
			t_new = max(t_new,h.clock().time());
		}
		t_new = t_new + 1;
		self.clock = LamportClock::new(self.clock.id()).set_time(t_new);

		let mut heads = Vec::new();
		for h in &self.heads {
			heads.push(h.clone());
		}
		let mut refs = self.traverse(&heads[..],Some(max(n_ptr.unwrap_or(1),self.heads.len())),None);
		self.heads.reverse();
		self.heads = Log::dedup(&self.heads);
		self.heads.append(&mut refs);

		//should be created asynchronically in IPFS
		let entry = Entry::new(self.identity.clone(),&self.id,data,
		&self.heads.iter().map(|x| EntryOrHash::Hash(x.hash().to_owned())).collect::<Vec<_>>()[..],
		Some(self.clock.clone()));
		//should be queried asynchronically
		if !self.access.can_access(&entry) {
			panic!("Could not append entry, key \"{}\" is not allowed to write in the log",
			self.identity.id());
		}

		let eh = entry.hash().to_owned();
		let rc = Rc::new(entry);
		self.entries.insert(eh.to_owned(),rc.clone());
		for h in &self.heads {
			self.nexts.insert(h.hash().to_owned());
		}
		self.heads.clear();
		self.heads.push(rc);
		self.length += 1;

		&self.entries[&eh]
	}

	pub fn join (&mut self, other: &Log, size: Option<usize>) -> bool {
		if self.id != other.id {
			return false;
		}
		let new_hashes = other.diff(&self);

		//something about identify provider and verification,
		//implement later
		//...
		//...

		for h in &new_hashes {
			if let None = self.get(*h) {
				self.length += 1;
			}
			let e = other.get(*h).unwrap();
			for n in e.next() {
				self.nexts.insert(other.get(n).unwrap().hash().to_owned());
			}
		}

		for h in &new_hashes {
			self.entries.insert((*h).to_owned(),other.get(*h).unwrap().clone());
		}

		let mut nexts_from_new_items = HashSet::new();
		for h in &new_hashes {
			other.get(h).unwrap().next().iter().for_each(|n| {
				nexts_from_new_items.insert(n);
			});
		}
		let all_heads = Log::find_heads(&self.heads.iter().chain(other.heads.iter()).map(|x| x.clone()).collect::<Vec<_>>()[..]);
		let merged_heads: Vec<Rc<Entry>> = all_heads.into_iter().filter(|x| !nexts_from_new_items.contains(&x.hash().to_owned())).
		filter(|x| !self.nexts.contains(&x.hash().to_owned())).collect();
		self.heads = Log::dedup(&merged_heads[..]);

		//incorrect, reimplement this
		/*
		if let Some(n) = size {
			let mut vs = self.traverse(&self.heads.iter().map(|x| self.get(x).unwrap()).collect::<Vec<_>>()[..],None,None);
			vs.reverse();
			let mut s = HashSet::new();
			vs = vs.into_iter().filter(|x| s.insert(x.to_owned())).take(n).collect();
			self.heads = Log::find_heads(&vs.iter().map(|x| self.get(x).unwrap()).collect::<Vec<_>>()[..]);
			let es = HashMap::new();
			vs.into_iter().for_each(|x| {
				self.entries.insert(x.to_owned(),self.get(&x).unwrap().clone());
			});
			self.entries = es;
			self.length = self.entries.len();
		}*/

		let mut t_max = 0;
		for h in &self.heads {
			t_max = max(t_max,h.clock().time());
		}
		self.clock = LamportClock::new(&self.id).set_time(t_max);

		true
	}

	pub fn diff (&self, other: &Log) -> Vec<&str> {
		let mut stack: Vec<String> = self.heads.iter().map(|x| x.hash().to_owned()).collect();
		let mut traversed = HashSet::<&str>::new();
		let mut diff = Vec::new();
		while !stack.is_empty() {
			let hash = stack.remove(0);
			let a = self.get(&hash);
			let b = other.get(&hash);
			if a.is_some() && b.is_none()
			&& a.unwrap().id() == other.id {
				let a = a.unwrap();
				diff.push(a.hash());
				traversed.insert(a.hash());
				for n in a.next() {
					if !traversed.contains(&n[..]) && other.get(n).is_none() {
						stack.push(n.to_owned());
						traversed.insert(n);
					}
				}
			}
		}
		diff
	}

	pub fn json (&self) -> String {
		let mut hs = self.heads.to_owned();
		hs.sort_by(|a,b| (self.fn_sort)(a,b));
		hs.reverse();
		json!({
			"id": self.id,
			"heads": hs.into_iter().map(|x| x.hash().to_owned()).collect::<Vec<_>>(),
		}).to_string()
	}

	pub fn snapshot (&self) -> String {
		let hs = self.heads.to_owned();
		let vs = self.values().to_owned();
		json!({
			"id": self.id,
			"heads": hs.into_iter().map(|x| serde_json::to_string(&*x).unwrap()).collect::<Vec<_>>(),
			"values": vs.into_iter().map(|x| serde_json::to_string(&*x).unwrap()).collect::<Vec<_>>(),
		}).to_string()
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

#[derive(Copy,Clone)]
pub struct AdHocAccess;

impl AdHocAccess {
	fn can_access (&self, _entry: &Entry) -> bool {
		true
	}
}
