use std::collections::BTreeSet;
use std::borrow::Borrow;

pub struct GSet<T: Eq + Ord + Clone> {
	set: BTreeSet<T>,
}

impl<T: Eq + Ord + Clone> GSet<T> {
	pub fn new () -> GSet<T> {
		GSet {
			set: BTreeSet::new(),
		}
	}

	pub fn insert (&mut self, value: T) -> bool {
		self.set.insert(value)
	}

	pub fn get<Q> (&self, value: &Q) -> Option<&T>
	where Q: Ord + ?Sized, T: Borrow<Q> {
		self.set.get(value)
	}

	pub fn contains<Q> (&self, value: &Q) -> bool
	where Q: Ord + ?Sized, T: Borrow<Q> {
		self.set.contains(value)
	}

	pub fn len (&self) -> usize {
		self.set.len()
	}

	pub fn is_empty (&self) -> bool {
		self.set.is_empty()
	}

	pub fn is_subset (&self, o: &GSet<T>) -> bool {
		self.set.is_subset(&o.set)
	}

	pub fn union (a: &GSet<T>, b: &GSet<T>) -> GSet<T> {
		let mut c = GSet::new();
		c.set = a.set.union(&b.set).cloned().collect();
		c
	}
}
