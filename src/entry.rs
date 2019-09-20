use std::cmp::Ordering;
use std::rc::Rc;
use serde::Serialize;
use crate::lamport_clock::LamportClock;
use crate::identity::Identity;
use crate::identity::IdAndKey;

pub enum EntryOrHash<'a> {
	Entry(&'a Entry),
	Hash(String),
}

/// An entry containing data payload, a hash to locate it in [`IPFS`],
/// and pointers to its parents.
///
/// [`IPFS`]: https://ipfs.io
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
	#[doc(hidden)]
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

	#[doc(hidden)]
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
			clock: clock.unwrap_or(LamportClock::new(identity.pub_key())),
		}
	}

	/// Locally creates an entry owned by `identity` .
	///
	///  The created entry is part of the [log] with the id `log_id`,
	/// holds payload of `data` and can be assigned to point to
	/// at most two parents with their hashes in `nexts`. Providing a
	/// [Lamport clock] via `clock` is optional.
	///
	/// Returns a [reference-counting pointer] to the created entry.
	///
	/// [log]: ../log/struct.Log.html
	/// [Lamport clock]: ../lamport_clock/struct.LamportClock.html
	/// [reference-counting pointer]: https://doc.rust-lang.org/std/rc/struct.Rc.html
	pub fn create (identity: Identity, log_id: &str, data: &str,
	nexts: &[EntryOrHash], clock: Option<LamportClock>) -> Rc<Entry> {
		Rc::new(Entry::new(identity,log_id,data,nexts,clock))
	}

	/// Returns the hash of the entry.
	pub fn hash (&self) -> &str {
		&self.hash
	}

	/// Returns the id of the entry that is the same as of the containing log.
	pub fn id (&self) -> &str {
		&self.id
	}

	/// Returns the data payload of the entry.
	pub fn payload (&self) -> &str {
		&self.payload
	}

	/// Returns the hashes of the parents.
	///
	/// The length of the returned slice is either:
	/// * 0 &mdash; no parents
	/// * 2 &mdash; two identical strings for one parent, two distinct strings for two different parents
	pub fn next (&self) -> &[String] {
		&self.next
	}

	/// Returns the Lamport clock of the entry.
	pub fn clock (&self) -> &LamportClock {
		&self.clock
	}

	/// Returns `true` if `e1` is the parent of `e2`, otherwise returns `false`.
	pub fn is_parent (e1: &Entry, e2: &Entry) -> bool {
		e2.next().iter().any(|x| x == e1.hash())
	}

	/// Returns a vector of pointers to all direct and indirect children of `entry`.
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
