use std::cmp::Ordering;

pub struct LamportClock {
	id: Identity,
	time: u64,
}

impl LamportClock {
	pub fn new (id: Identity) -> LamportClock {
		LamportClock {
			id: id,
			time: 0,
		}
	}

	pub fn time (mut self, time: u64) -> LamportClock {
		self.time = time;
		self
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
			else {
				Ordering::Greater
			}
		}
	}
}

impl PartialOrd for LamportClock {
	fn partial_cmp (&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Ord,PartialOrd,Eq,PartialEq)]
pub struct Identity {
	id: u64,
}

impl Identity {
	fn new (id: u64) -> Identity {
		Identity {
			id: id,
		}
	}
}

//very much ad hoc
pub struct IdentityBuilder {
	id: u64,
}

impl IdentityBuilder {
	pub fn new () -> IdentityBuilder {
		IdentityBuilder {
			id: 0,
		}
	}

	pub fn build (&mut self) -> Identity {
		let i = Identity::new(self.id);
		self.id += 1;
		i
	}
}
