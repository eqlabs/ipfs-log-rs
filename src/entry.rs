use crate::lamport_clock::LamportClock;
use crate::identity::Identity;

//very much ad hoc
pub type Data = String;

pub enum EntryOrHash {
	Entry(Entry),
	Hash(String),
}

pub struct Entry {
	hash: String,
	id: String,
	payload: Data,
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

	pub fn new (identity: Identity, log_id: &str, data: Data,
	next: &[EntryOrHash], clock: Option<LamportClock>) -> Entry {
		//None filtering required?
		let next = next.iter().map(|n| match n {
			EntryOrHash::Entry(e)	=>	e.hash.to_owned(),
			EntryOrHash::Hash(h)	=>	h.to_owned(),
		}).collect();
		Entry {
			//very much ad hoc
			hash: "12345678".to_owned(),
			id: log_id.to_owned(),
			payload: data,
			next: next,
			v: 1,
			clock: clock.unwrap_or(LamportClock::new(identity.public_key())),
		}
	}

	pub fn next (&self) -> &Vec<String> {
		&self.next
	}

	pub fn clock (&self) -> &LamportClock {
		&self.clock
	}

	pub fn hash (&self) -> &str {
		&self.hash
	}
}
