use std::cmp::Ordering;
use std::rc::Rc;
use serde::Serialize;
use crate::lamport_clock::LamportClock;
use crate::identity::Identity;

pub enum EntryOrHash<'a> {
	Entry(&'a Entry),
	Hash(String),
}

#[derive(Clone,Debug,Serialize)]
pub struct Entry {
	hash: String,
	id: String,
	payload: String,
	next: Vec<String>,
	v: u32,
	clock: LamportClock,
}

impl Entry {
	//very ad hoc
	pub fn empty () -> Entry {
		let s = "0000";
		Entry {
			hash: s.to_owned(),
			id: s.to_owned(),
			payload: s.to_owned(),
			next: Vec::new(),
			v: 0,
			clock: LamportClock::new(s),
		}
	}

	pub fn new (identity: Identity, log_id: &str, data: &str,
	next: &[EntryOrHash], clock: Option<LamportClock>) -> Entry {
		//None filtering required?
		let next = next.iter().map(|n| match n {
			EntryOrHash::Entry(e)	=>	e.hash.to_owned(),
			EntryOrHash::Hash(h)	=>	h.to_owned(),
		}).collect();
		Entry {
			//very much ad hoc
			hash: data.to_owned(),
			id: log_id.to_owned(),
			payload: data.to_owned(),
			next: next,
			v: 1,
			clock: clock.unwrap_or(LamportClock::new(identity.public_key())),
		}
	}

	pub fn create (identity: Identity, log_id: &str, data: &str,
	next: &[EntryOrHash], clock: Option<LamportClock>) -> Rc<Entry> {
		Rc::new(Entry::new(identity,log_id,data,next,clock))
	}

	pub fn hash (&self) -> &str {
		&self.hash
	}

	pub fn id (&self) -> &str {
		&self.id
	}

	pub fn payload (&self) -> &str {
		&self.payload
	}

	pub fn next (&self) -> &Vec<String> {
		&self.next
	}

	pub fn clock (&self) -> &LamportClock {
		&self.clock
	}

	pub fn is_parent (e1: &Entry, e2: &Entry) -> bool {
		e2.next().iter().any(|x| x == e1.hash())
	}

	pub fn find_children (entry: &Entry, entries: &[Rc<Entry>]) -> Vec<Rc<Entry>> {
		let mut stack = Vec::new();
		let mut parent = entries.iter().find(|e| Entry::is_parent(entry,e));
		while let Some(p) = parent {
			stack.push(p.clone());
			let prev = p;
			parent = entries.iter().find(|e| Entry::is_parent(prev,e));
		}
		stack.sort_by(|a,b| a.clock().time().cmp(&b.clock().time()));
		stack
	}
}

impl PartialEq for Entry {
	fn eq (&self, other: &Self) -> bool {
		self.hash == other.hash
	}
}

impl Eq for Entry {}

impl Ord for Entry {
	fn cmp (&self, other: &Self) -> Ordering {
		let diff = self.clock().cmp(other.clock());
		if diff == Ordering::Equal {
			self.clock().id().cmp(other.clock().id())
		}
		else {
			diff
		}
	}
}

impl PartialOrd for Entry {
	fn partial_cmp (&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}
