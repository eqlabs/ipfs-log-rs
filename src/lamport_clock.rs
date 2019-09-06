use serde::Serialize;
use std::cmp::Ordering;

#[derive(Clone,Serialize)]
pub struct LamportClock {
	id: String,
	time: u64,
}

impl LamportClock {
	pub fn new (id: &str) -> LamportClock {
		LamportClock {
			id: id.to_owned(),
			time: 0,
		}
	}

	pub fn set_time (mut self, time: u64) -> LamportClock {
		self.time = time;
		self
	}

	pub fn time (&self) -> u64 {
		self.time
	}

	pub fn id (&self) -> &str {
		&self.id
	}

	pub fn tick (&mut self) {
		self.time += 1;
	}

	pub fn merge (&mut self, o: &LamportClock) {
		if self.time < o.time {
			self.time = o.time;
		}
	}
}

impl PartialEq for LamportClock {
	fn eq (&self, other: &Self) -> bool {
		self.time == other.time && self.id == other.id
	}
}

impl Eq for LamportClock {}

impl Ord for LamportClock {
	fn cmp (&self, other: &Self) -> Ordering {
		let delta = self.time as i64 - other.time as i64;
		if delta == 0 && self.id != other.id {
			if self.id < other.id {
				Ordering::Less
			}
			else {
				Ordering::Greater
			}
		}
		else {
			if delta < 0 {
				Ordering::Less
			}
			else if delta > 0 {
				Ordering::Greater
			}
			//is this necessary/hoped for?
			else {
				Ordering::Equal
			}
		}
	}
}

impl PartialOrd for LamportClock {
	fn partial_cmp (&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}
