//! Options for the `Log` trait

use crate::entry::Entry;
use crate::lamport_clock::LamportClock;
use crate::log::AdHocAccess;
// use std::cmp::Ordering;
use crate::identity::Identity;
use cid::Cid;
// use multihash::Multihash;

/// Options for constructing [`Log`].
///
/// Constructing log options using `LogOptions::new()` creates default log options:
/// * no identifier,
/// * no entries (and no heads among those non-existent entries),
/// * no Lamport clock,
/// * no sorting algorithm.
///
/// Use method chaining to set additional parameters:
///
/// ```ignore
/// let opts = LogOptions::new().id("some_id").clock(LamportClock::new().set_time(128));
/// let log = Log::new(/* identity */,opts);
/// ```
///
/// [`Log`]: ./struct.Log.html

#[derive(Debug)]
pub enum SortMethod {
    /// Last write wins sorting strategy
    LastWriteWins,
}

#[derive(Debug)]

/// Log Options
pub struct LogOptions {
    id: Option<String>,
    identity: Option<Identity>,
    access: AdHocAccess,
    entries: Vec<Entry>,
    heads: Vec<Cid>,
    // TODO: Convert to enum of different clocks
    clock: Option<LamportClock>,
    // TODO: Convert to enum of different sorting strategies, don't pass a function
    strategy: SortMethod,
}

impl Default for LogOptions {
    fn default() -> Self {
        LogOptions {
            id: None,
            identity: None,
            access: AdHocAccess,
            entries: Vec::<Entry>::new(),
            heads: Vec::<Cid>::new(),
            clock: None,
            strategy: SortMethod::LastWriteWins,
        }
    }
}

impl LogOptions {
    /// Constructs default log options.
    pub fn new() -> LogOptions {
        LogOptions::default()
    }

    /// Getter for access
    pub fn access(&self) -> AdHocAccess {
        self.access
    }

    /// Getter for id
    pub fn id(&self) -> Option<String> {
        self.id.clone()
    }

    /// Getter for clock
    pub fn clock(&self) -> Option<LamportClock> {
        self.clock.clone()
    }

    /// Getter for entries
    pub fn entries(&self) -> Vec<Entry> {
        self.entries.clone()
    }

    /// Getter for heads
    pub fn heads(&self) -> Vec<Cid> {
        self.heads.clone()
    }

    /// Getter for identity
    pub fn identity(&self) -> Option<Identity> {
        self.identity.clone()
    }

    /// Sets the identifier for the constructed log options.
    ///
    /// Allows method chaining.
    pub fn set_id(mut self, id: &str) -> LogOptions {
        self.id = Some(id.to_owned());
        self
    }

    /// Sets the entries for the constructed log options.
    ///
    /// Allows method chaining.
    pub fn set_entries(mut self, es: Vec<Entry>) -> LogOptions {
        self.entries = es;
        self
    }

    /// Sets the heads for the constructed log options.
    ///
    /// Allows method chaining.
    pub fn set_heads(mut self, hs: Vec<Cid>) -> LogOptions {
        self.heads = hs;
        self
    }

    /// Sets the Lamport clock for the constructed log options.
    ///
    /// Allows method chaining.
    pub fn set_clock(mut self, clock: LamportClock) -> LogOptions {
        self.clock = Some(clock);
        self
    }

    // Sets the sorting algorithm for the constructed log options.
    //
    // Allows method chaining.
    // pub fn fn_sort<F>(mut self, fn_sort: F) -> LogOptions<'log, 'options>
    // where
    //     F: 'static + Fn(&Entry, &Entry) -> Ordering,
    // {
    //     self.fn_sort = Some(Box::new(fn_sort));
    //     self
    // }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::identity::{Identity, Signatures};
    use crate::log::{Log};
    // use multihash::Multihash;
    // use std::collections::HashSet;
    // use std::rc::Rc;

    fn identity1() -> Identity {
        Identity::new(
            "userA",
            "public",
            Signatures::new("id_signature", "public_signature"),
        )
    }

    #[test]
    fn set_id() {
        let options = LogOptions::new().set_id("ABC");
        assert_eq!(options.id(), Some("ABC".to_string()));
    }

    // #[test]
    // fn set_clock_id() {
    //     let id = identity1();
    //     // TODO: Was I drunk when I wrote this test??
    //     let options = LogOptions::new().set_id("ABC");
    //     assert_eq!(options.clock().unwrap().id(), id.pub_key());
    // }

    #[test]
    #[ignore]
    fn set_items() {
        let identity = identity1();

        let e1 = Entry::new(
            &identity,
            "A",
            b"entryA",
            Vec::<String>::new(),
            Some(LamportClock::new("A")),
        );
        let e2 = Entry::new(
            &identity,
            "B",
            b"entryB",
            Vec::<String>::new(),
            Some(LamportClock::new("B")),
        );
        let e3 = Entry::new(
            &identity,
            "C",
            b"entryC",
            Vec::<String>::new(),
            Some(LamportClock::new("C")),
        );

        let options = LogOptions::new().set_id("A").set_entries(vec![e1, e2, e3]);

        assert_eq!(options.entries.len(), 3);
        // assert_eq!(log.values()[0].payload(),b"entryA");
        // assert_eq!(log.values()[1].payload(),b"entryB");
        // assert_eq!(log.values()[2].payload(),b"entryC");
    }

    #[test]
    fn set_heads() {
        let identity = identity1();
        let e1 = Entry::new(&identity, "A", b"entryA", Vec::<String>::new(), None);
        let e2 = Entry::new(&identity, "B", b"entryB", Vec::<String>::new(), None);
        let e3 = Entry::new(&identity, "C", b"entryC", Vec::<String>::new(), None);

        let options = LogOptions::new().set_id("A").set_entries(vec![e1, e2, e3]);
        // TODO: Let's remove set_heads or make either or?
        // .set_heads(&[&e3]),
        // assert_eq!(log.heads().len(), 1);
        // assert_eq!(log.heads()[0].hash(),e3.hash());
    }
}
