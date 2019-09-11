use std::collections::HashMap;
use std::collections::HashSet;
use std::cmp::Ordering;
use std::cmp::max;
use std::time::SystemTime;
use std::rc::Rc;
use std::fmt::{Display,Formatter,Result};
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

	pub fn find_tails (entries: &[Rc<Entry>]) -> Vec<Rc<Entry>> {
		let mut no_nexts = Vec::new();
		let mut reverses = HashMap::new();
		let mut nexts = HashSet::new();
		let mut hashes: HashSet<&str> = HashSet::new();
		for e in entries {
			if e.next().is_empty() {
				no_nexts.push(e.clone());
			}
			for n in e.next() {
				reverses.insert(n,e.clone());
				nexts.insert(n);
			}
			hashes.insert(e.hash());
		}
		//correct order?
		let mut tails = Log::dedup(&nexts.iter().filter(|&&x| !hashes.contains(&x[..])).
		map(|x| reverses[x].clone()).chain(no_nexts.into_iter()).collect::<Vec<_>>()[..]);
		tails.sort();
		tails
	}

	pub fn find_tail_hashes (entries: &[Rc<Entry>]) -> Vec<String> {
		let mut hashes: HashSet<&str> = HashSet::new();
		for e in entries {
			hashes.insert(e.hash());
		}
		let mut ths = Vec::new();
		for e in entries {
			for i in e.next().len() - 1..0 {
				let n = &e.next()[i];
				if !hashes.contains(&n[..]) {
					ths.push(n.to_owned());
				}
			}
		}
		ths.reverse();
		ths
	}

	fn dedup (v: &[Rc<Entry>]) -> Vec<Rc<Entry>> {
		let mut s = HashSet::new();
		v.iter().filter(|x| s.insert(x.hash())).map(|x| x.clone()).collect()
	}

	pub fn id (&self) -> &str {
		&self.id
	}

	pub fn clock (&self) -> &LamportClock {
		&self.clock
	}

	pub fn has (&self, hash: &str) -> bool {
		self.entries.contains_key(hash)
	}

	pub fn len (&self) -> usize {
		self.length
	}

	pub fn get (&self, hash: &str) -> Option<&Rc<Entry>> {
		self.entries.get(hash)
	}

	pub fn values (&self) -> Vec<Rc<Entry>> {
		let mut es = self.traverse(&self.heads,None,None);
		es.reverse();
		es
	}

	pub fn heads (&self) -> Vec<Rc<Entry>> {
		let mut hs = self.heads.to_owned();
		hs.sort_by(|a,b| (self.fn_sort)(a,b));
		hs.reverse();
		hs
	}

	pub fn tails (&self) -> Vec<Rc<Entry>> {
		Log::find_tails(&self.values())
	}

	pub fn tail_hashes (&self) -> Vec<String> {
		Log::find_tail_hashes(&self.values())
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

	pub fn traverse (&self, roots: &[Rc<Entry>], amount: Option<usize>, end_hash: Option<String>) -> Vec<Rc<Entry>> {
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
		let new_items = other.diff(&self);

		//something about identify provider and verification,
		//implement later
		//...
		//...

		for e in &new_items {
			if let None = self.get(e.0) {
				self.length += 1;
			}
			for n in e.1.next() {
				self.nexts.insert(n.to_owned());
			}
		}

		for e in &new_items {
			self.entries.insert(e.0.to_owned(),e.1.clone());
		}

		let mut nexts_from_new_items = HashSet::new();
		new_items.into_iter().map(|x| x.1.next().to_owned()).for_each(|n| n.iter().for_each(|n| {
			nexts_from_new_items.insert(n.to_owned());
		}));
		let all_heads = Log::find_heads(&self.heads.iter().chain(other.heads.iter()).map(|x| x.clone()).collect::<Vec<_>>()[..]);
		let merged_heads: Vec<Rc<Entry>> = all_heads.into_iter().filter(|x| !nexts_from_new_items.contains(&x.hash().to_owned())).
		filter(|x| !self.nexts.contains(&x.hash().to_owned())).collect();
		self.heads = Log::dedup(&merged_heads[..]);

		if let Some(n) = size {
			let mut vs = self.values();
			vs.reverse();
			vs = vs.into_iter().take(n).collect();

			self.entries.clear();
			for v in &vs {
				self.entries.insert(v.hash().to_owned(),v.clone());
			}

			self.heads = Log::find_heads(&Log::dedup(&vs));
			self.length = self.entries.len();
		}

		let mut t_max = 0;
		for h in &self.heads {
			t_max = max(t_max,h.clock().time());
		}
		self.clock = LamportClock::new(&self.id).set_time(t_max);

		true
	}

	pub fn diff (&self, other: &Log) -> HashMap<String,Rc<Entry>> {
		let mut stack: Vec<String> = self.heads.iter().map(|x| x.hash().to_owned()).collect();
		let mut traversed = HashSet::<&str>::new();
		let mut diff = HashMap::new();
		while !stack.is_empty() {
			let hash = stack.remove(0);
			let a = self.get(&hash);
			let b = other.get(&hash);
			if a.is_some() && b.is_none()
			&& a.unwrap().id() == other.id {
				let a = a.unwrap();
				for n in a.next() {
					if !traversed.contains(&n[..]) && other.get(n).is_none() {
						stack.push(n.to_owned());
						traversed.insert(n);
					}
				}
				traversed.insert(a.hash());
				diff.insert(a.hash().to_owned(),a.clone());
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

	pub fn buffer (&self) -> Vec<u8> {
		self.json().into_bytes()
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

impl Display for Log {
	fn fmt (&self, f: &mut Formatter) -> Result {
		let mut es = self.values();
		es.reverse();
		let mut s = String::new();
		for e in es {
			let parents = Entry::find_children(&e,&self.values());
			if parents.len() >= 1 {
				if parents.len() >= 2 {
					for _ in 0..parents.len() - 1 {
						s.push_str("  ");
					}
				}
				s.push_str("└─");
			}
			s.push_str(e.payload());
			s.push_str("\n");
		}
		write!(f,"{}",s)
	}
}

#[derive(Copy,Clone)]
pub struct AdHocAccess;

impl AdHocAccess {
	fn can_access (&self, _entry: &Entry) -> bool {
		true
	}
}
