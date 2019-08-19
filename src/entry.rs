use crate::lamport_clock::LamportClock;
use crate::lamport_clock::Identity;

//very much ad hoc
pub struct Data {
	data: String,
}

pub enum EntryOrHash {
	IsEntry(Entry),
	IsHash(String),
}

pub struct Entry{
	hash: String,
	id: String,
	payload: Data,
	next: Vec<String>,
	v: u32,
	clock: LamportClock,
}

impl Entry {
	pub fn new (id: Identity, log_id: &String, data: Data,
	next: &Vec<EntryOrHash>, clock: Option<LamportClock>) -> Entry {
		//None filtering required?
		let nexts = next.iter().map(|n| match n {
			EntryOrHash::IsEntry(e)	=>	e.hash.clone(),
			EntryOrHash::IsHash(s)	=>	s.to_owned(),
		}).collect();
		Entry {
			//very much ad hoc
			hash: "12345678".to_owned(),
			id: log_id.to_owned(),
			payload: data,
			next: nexts,
			v: 1,
			clock: clock.unwrap_or(LamportClock::new(id)),
		}
	}

	pub fn clock (&self) -> &LamportClock {
		&self.clock
	}

	pub fn hash (&self) -> &String {
		&self.hash
	}
}
