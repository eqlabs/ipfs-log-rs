use std::cmp::Ordering;

use crate::lamport_clock::LamportClock;

pub struct Entry {
	clock: LamportClock,
	hash: String,
}

pub enum SortStrategy<'a> {
	LastWriteWins,
	SortByEntryHash,
	SortByClocks(Box<&'a dyn Fn(&Entry,&Entry) -> Ordering>),
	SortByClockIds(Box<&'a dyn Fn(&Entry,&Entry) -> Ordering>),
}

pub struct Log<'a> {
	sort_strategy: SortStrategy<'a>,
}

impl<'a> Log<'a> {
	fn sort (&self, a: &Entry, b: &Entry) -> Ordering {
		let diff = match self.sort_strategy {
			SortStrategy::LastWriteWins			=>	self.sort_step_by_step(a,b,&Box::new(&|a,b| Ordering::Less)),
			SortStrategy::SortByEntryHash		=>	self.sort_step_by_step(a,b,&Box::new(&|a,b| a.hash.cmp(&b.hash))),
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

	fn sort_step_by_step (&self, a: &Entry, b: &Entry,
	resolve: &Box<&dyn Fn(&Entry,&Entry) -> Ordering>) -> Ordering {
		self.sort_by_clocks(a,b,&Box::new(&|a,b| self.sort_by_clock_ids(a,b,resolve)))
	}

	fn sort_by_clocks (&self, a: &Entry, b: &Entry,
	resolve: &Box<&dyn Fn(&Entry,&Entry) -> Ordering>) -> Ordering {
		let mut diff = a.clock.cmp(&b.clock);
		if diff == Ordering::Equal {
			diff = resolve(a,b);
		}
		diff
	}

	fn sort_by_clock_ids (&self, a: &Entry, b: &Entry,
	resolve: &Box<&dyn Fn(&Entry,&Entry) -> Ordering>) -> Ordering {
		let mut diff = a.clock.id().cmp(&b.clock.id());
		if diff == Ordering::Equal {
			diff = resolve(a,b);
		}
		diff
	}
}
