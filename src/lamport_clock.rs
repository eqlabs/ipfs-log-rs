use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use libipld::DagCbor;

/// A [Lamport clock] for partial chronological ordering of unconnected events.
///
/// [Lamport clock]: https://en.wikipedia.org/wiki/Lamport_clock
#[derive(Clone, DagCbor, Debug, Serialize, Deserialize)]
pub struct LamportClock {
    id: String,
    time: u64,
}

impl LamportClock {
    /// Constructs a new Lamport clock with the given identifier.
    pub fn new(id: &str) -> LamportClock {
        LamportClock {
            id: id.to_owned(),
            time: 0,
        }
    }

    /// Sets the time of the (newly constructed) Lamport clock.
    ///
    /// ```ignore
    /// let clock = LamportClock::new("some_id").set_time(128);
    /// ```
    pub fn set_time(mut self, time: u64) -> LamportClock {
        self.time = time;
        self
    }

    /// Returns the current time of the Lamport clock.
    pub fn time(&self) -> u64 {
        self.time
    }

    /// Returns the identifier of the Lamport clock.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Advances the time of the Lamport clock.
    pub fn tick(&mut self) {
        self.time += 1;
    }

    /// Merges `o` to `self` in the following manner:
    /// * if `self.time < o.time`, set `self.time = o.time`,
    /// otherwise do nothing
    /// * `o` is never modified
    pub fn merge(&mut self, o: &LamportClock) {
        if self.time < o.time {
            self.time = o.time;
        }
    }
}

impl PartialEq for LamportClock {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.id == other.id
    }
}

impl Eq for LamportClock {}

impl Ord for LamportClock {
    fn cmp(&self, other: &Self) -> Ordering {
        let delta = self.time as i64 - other.time as i64;
        if delta == 0 && self.id != other.id {
            if self.id < other.id {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        } else {
            if delta < 0 {
                Ordering::Less
            } else if delta > 0 {
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn assert_new_clock(id: &str, time: u64) -> LamportClock {
        let clock = LamportClock::new(id).set_time(time);
        assert_eq!(id, clock.id());
        assert_eq!(time, clock.time());

        clock
    }

    #[test]
    fn test_new() {
        assert_new_clock("test_new", 0);
    }

    #[test]
    fn test_tick() {
        let mut clock = assert_new_clock("test_tick", 0);

        for i in 1..100 {
            clock.tick();
            assert_eq!(i, clock.time());
        }
    }

    #[test]
    fn test_set_time() {
        let clock = assert_new_clock("test_set_time", 5);
    }

    #[test]
    fn test_merge() {
        let mut clock_1 = assert_new_clock("test_merge_1", 3);
        let clock_2 = assert_new_clock("test_merge_2", 50);

        clock_1.merge(&clock_2);
        assert_eq!("test_merge_1", clock_1.id());
        assert_eq!(50, clock_1.time());
    }

    #[test]
    fn test_eq() {
        let clock = assert_new_clock("test_eq", 5);
        assert_eq!(clock, clock);
    }

    #[test]
    fn test_partial_eq() {
        let clock_1 = assert_new_clock("test_partial_eq_1", 10);
        let clock_2 = assert_new_clock("test_partial_eq_1", 10);

        assert_eq!(clock_1, clock_2);
    }
}
