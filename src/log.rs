use std::cmp::Ordering;

use crate::lamport_clock::LamportClock;

pub struct Entry {
	clock: LamportClock,
	hash: String,
}

pub enum SortStrategy {
	LastWriteWins,
	SortByEntryHash,
	SortByClocks(Box<dyn Fn(&Entry,&Entry) -> Ordering>),
	SortByClockIds(Box<dyn Fn(&Entry,&Entry) -> Ordering>),
}

pub struct Log {
	sort_strategy: SortStrategy,
}

impl Log {
	fn sort (&self, a: &Entry, b: &Entry) -> Ordering {
		let diff = match self.sort_strategy {
			SortStrategy::LastWriteWins			=>	self.sort_step_by_step(a,b,|_,_| Ordering::Less),
			SortStrategy::SortByEntryHash		=>	self.sort_step_by_step(a,b,|a,b| a.hash.cmp(&b.hash)),
			SortStrategy::SortByClocks(ref f)	=>	self.sort_by_clocks(a,b,f),
			SortStrategy::SortByClockIds(ref f)	=>	self.sort_by_clock_ids(a,b,f),
		};
		if diff == Ordering::Equal {
			let strategy = match self.sort_strategy {
				SortStrategy::LastWriteWins		=>	"LastWriteWins",
				SortStrategy::SortByEntryHash	=>	"SortByEntryHash",
				SortStrategy::SortByClocks(_)	=>	"SortByClocks",
				SortStrategy::SortByClockIds(_)	=>	"SortByClockIds",
			};
			panic!("Your log's tiebreaker function, {},{}",strategy,
			"has returned zero and therefore cannot be");
		}
		diff

		//this is arguably more readable and understandable,
		//yet implements only LastWriteWins and SortByEntryHash
		/*
		let mut diff = a.clock.cmp(&b.clock);
		if diff == Ordering::Equal {
			diff = a.clock.id().cmp(&b.clock.id());
		}
		if diff == Ordering::Equal {
			diff = match self.sort_strategy {
				SortStrategy::LastWriteWins		=>	Ordering::Less,
				SortStrategy::SortByEntryHash	=>	a.hash.cmp(&b.hash),
				_								=>	unreachable!(),
			}
		}
		match diff {
			Ordering::Less		=>	a,
			Ordering::Greater	=>	b,
			Ordering::Equal		=>	{
				let strategy = match self.sort_strategy {
					SortStrategy::LastWriteWins		=>	"LastWriteWins",
					SortStrategy::SortByEntryHash	=>	"SortByEntryHash",
					_								=>	unreachable!(),
				};
				panic!("Your log's tiebreaker function, {},{}",strategy,
				"has returned zero and therefore cannot be");
			},
		}*/
	}

	fn sort_step_by_step<F: Fn(&Entry,&Entry) -> Ordering> (&self, a: &Entry, b: &Entry,
	resolve: F) -> Ordering {
		self.sort_by_clocks(a,b,|a,b| self.sort_by_clock_ids(a,b,&resolve))
	}

	fn sort_by_clocks<F: Fn(&Entry,&Entry) -> Ordering> (&self, a: &Entry, b: &Entry,
	resolve: F) -> Ordering {
		let mut diff = a.clock.cmp(&b.clock);
		if diff == Ordering::Equal {
			diff = resolve(a,b);
		}
		diff
	}

	fn sort_by_clock_ids<F: Fn(&Entry,&Entry) -> Ordering> (&self, a: &Entry, b: &Entry,
	resolve: F) -> Ordering {
		let mut diff = a.clock.id().cmp(&b.clock.id());
		if diff == Ordering::Equal {
			diff = resolve(a,b);
		}
		diff
	}
}
