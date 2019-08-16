use std::cmp::Ordering;

use crate::lamport_clock::LamportClock;

pub struct Entry {
	clock: LamportClock,
	hash: String,
}

pub enum SortStrategy {
	LastWriteWins,
	SortByEntryHash,
}

pub struct Log {
	sort_strategy: SortStrategy,
}

impl Log {
	fn pick_later<'a> (&self, a: &'a Entry, b: &'a Entry) -> &'a Entry {
		let mut diff = a.clock.cmp(&b.clock);
		if diff == Ordering::Equal {
			a.clock.id().cmp(&b.clock.id());
		}
		if diff == Ordering::Equal {
			diff = match self.sort_strategy {
				SortStrategy::LastWriteWins		=>	Ordering::Less,
				SortStrategy::SortByEntryHash	=>	a.hash.cmp(&b.hash),
			}
		}
		match diff {
			Ordering::Less		=>	a,
			Ordering::Greater	=>	b,
			Ordering::Equal		=>	{
				let strategy = match self.sort_strategy {
					SortStrategy::LastWriteWins		=>	"LastWriteWins",
					SortStrategy::SortByEntryHash	=>	"SortByEntryHash",
				};
				panic!("Your log's tiebreaker function, {},{}",strategy,
				"has returned zero and therefore cannot be");
			},
		}
	}
}
