use std::cmp::Ordering;
use std::rc::Rc;
use serde::Serialize;
use crate::lamport_clock::LamportClock;
use crate::identity::Identity;

/// A wrapper containing either a reference to an entry
/// or a hash as a string.
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

	/// Returns the identifier of the entry that is the same as of the containing log.
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

	/// Returns a vector of pointers to all direct and indirect children of `entry` in `entries`.
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

	/// A sorting function to pick the more recently written entry.
	///
	/// Uses [`sort_step_by_step`], resolving unsorted cases in the manner defined in it.
	///
	/// Returns an ordering.
	///
	/// [`sort_step_by_step`]: #method.sort_step_by_step
	pub fn last_write_wins (a: &Entry, b: &Entry) -> Ordering {
		Entry::sort_step_by_step(|_,_| Ordering::Less)(a,b)
	}

	/// A sorting function to pick the entry with the greater hash.
	///
	/// Uses [`sort_step_by_step`], resolving unsorted cases in the manner defined in it.
	///
	/// Returns an ordering.
	///
	/// [`sort_step_by_step`]: #method.sort_step_by_step
	pub fn sort_by_entry_hash (a: &Entry, b: &Entry) -> Ordering {
		Entry::sort_step_by_step(|a,b| a.hash().cmp(&b.hash()))(a,b)
	}

	/// A sorting helper function to
	/// 1. first try to sort the two entries using `resolve`,
	/// 2. if still unsorted (equal), try to sort based on the Lamport clock identifiers of the respective entries,
	/// 3. sort by the Lamport clocks of the respective entries.
	///
	/// Returns a closure that can be used as a sorting function.
	pub fn sort_step_by_step<F> (resolve: F) -> impl Fn(&Entry,&Entry) -> Ordering
	where F: 'static + Fn(&Entry,&Entry) -> Ordering {
		Entry::sort_by_clocks(Entry::sort_by_clock_ids(resolve))
	}

	/// A sorting helper function to sort by the Lamport clocks of the respective entries.
	/// In the case the Lamport clocks are equal, tries to sort using `resolve`.
	///
	/// Returns a closure that can be used as a sorting function.
	pub fn sort_by_clocks<F> (resolve: F) -> impl Fn(&Entry,&Entry) -> Ordering
	where F: 'static + Fn(&Entry,&Entry) -> Ordering {
		move |a,b| {
			let mut diff = a.clock().cmp(&b.clock());
			if diff == Ordering::Equal {
				diff = resolve(a,b);
			}
			diff
		}
	}

	/// A sorting helper function to sort by the Lamport clock identifiers of the respective entries.
	/// In the case the Lamport clocks identifiers are equal, tries to sort using `resolve`.
	///
	/// Returns a closure that can be used as a sorting function.
	pub fn sort_by_clock_ids<F> (resolve: F) -> impl Fn(&Entry,&Entry) -> Ordering
	where F: 'static + Fn(&Entry,&Entry) -> Ordering {
		move |a,b| {
			let mut diff = a.clock().id().cmp(&b.clock().id());
			if diff == Ordering::Equal {
				diff = resolve(a,b);
			}
			diff
		}
	}

	/// A sorting helper function that forbids the sorting function `fn_sort` from
	/// producing unsorted (equal) cases.
	///
	/// Returns a closure that behaves in the same way as `fn_sort`
	/// but panics if the two entries given as input are equal.
	pub fn no_zeroes<F> (fn_sort: F) -> impl Fn(&Entry,&Entry) -> Ordering
	where F: 'static + Fn(&Entry,&Entry) -> Ordering {
		move |a,b| {
			let diff = fn_sort(a,b);
			if diff == Ordering::Equal {
				panic!("Your log's tiebreaker function {}",
				"has returned zero and therefore cannot be");
			}
			diff
		}
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
