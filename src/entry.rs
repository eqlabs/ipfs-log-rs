use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;
// use std::sync::Arc;
// use std::sync::Mutex;

// TODO: JSON should be feature flagged
// use serde::{Deserialize, Serialize};
// use serde_json::json;

// use futures::{future::{Future,join_all}};
//
use libipld::DagCbor;

use crate::identity::Identity;
use crate::lamport_clock::LamportClock;

use multihash::Multihash;

use std::convert::From;

/// An entry containing data payload, a hash to locate it in [`IPFS`],
/// and pointers to its parents.
///
/// [`IPFS`]: https://ipfs.io

// TODO, make this a simple builder pattern

#[derive(Clone, DagCbor)] // ,Debug,Serialize,Deserialize)]
pub struct Entry {
    identity: Identity,
    log_id: String,
    payload: Vec<u8>,
    next: Vec<String>,
    v: u32,
    clock: LamportClock,
}

impl Entry {
    #[doc(hidden)]
    pub fn new(
        identity: &Identity,
        log_id: &str,
        data: &[u8],
        next: Vec<String>,
        clock: Option<LamportClock>,
    ) -> Entry {
        // TODO: 'log lifetime
        Entry {
            identity: identity.to_owned(),
            log_id: log_id.to_owned(),
            payload: Vec::<u8>::from(data), // data.to_owned(),
            next: next.to_owned(),
            v: 1,
            clock: clock.unwrap_or(LamportClock::new(identity.pub_key())),
        }
    }

    // Locally creates an entry owned by `identity` .
    //
    //  The created entry is part of the [log] with the id `log_id`,
    // holds payload of `data` and can be assigned to point to
    // at most two parents with their hashes in `nexts`. Providing a
    // [Lamport clock] via `clock` is optional.
    //
    // Returns a [reference-counting pointer] to the created entry.
    //
    // [log]: ../log/struct.Log.html
    // [Lamport clock]: ../lamport_clock/struct.LamportClock.html
    // [reference-counting pointer]: https://doc.rust-lang.org/std/rc/struct.Rc.html

    // Stores `entry` in the IPFS client `ipfs` and returns a future containing its multihash.
    //
    // **N.B.** *At the moment stores the entry as JSON, not CBOR DAG.*

    /// Returns the future containing the entry stored in the IPFS client `ipfs` with the multihash `hash`.
    ///
    /// **N.B.** *At the moment converts the entry from JSON, not CBOR DAG.*
    /// pub fn from_multihash (ipfs: &IpfsClient, hash: &str) -> impl Future<Item = Entry,Error = Error> + Send {
    /// 	let h = hash.to_owned();
    /// 	ipfs.cat(hash).concat2().map(|x| {
    /// 		let mut e: Entry = serde_json::from_str(std::str::from_utf8(&x).unwrap()).unwrap();
    /// 		e.hash = h;
    /// 		e
    /// 	})
    /// }
    /// Returns the data payload of the entry.
    pub fn payload(&self) -> Vec<u8> {
        self.payload.clone()
        // let payload_0 = std::str::from_utf8(log.values()[0].payload()).unwrap();
    }

    /// Returns the hashes of the parents.
    ///
    /// The length of the returned slice is either:
    /// * 0 &mdash; no parents
    /// * 2 &mdash; two identical strings for one parent, two distinct strings for two different parents
    pub fn next(&self) -> Vec<String> {
        self.next.clone()
    }

    /// Returns the Lamport clock of the entry.
    pub fn clock(&self) -> &LamportClock {
        &self.clock
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.payload == other.payload
            && self.next == other.next
            && self.v == other.v
            && self.clock == other.clock
    }
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        // TODO: Figure out a better way to format these, including a multihash call
        // here, maybe? Since it's Debug
        write!(formatter, "{:?}\n", self.payload)
    }
}

impl Eq for Entry {}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        let diff = self.clock().cmp(other.clock());
        if diff == Ordering::Equal {
            self.clock().id().cmp(other.clock().id())
        } else {
            diff
        }
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


// Required for multihash digest
// What can we accomplish here? Compression?
impl AsRef<[u8]> for Entry {
    fn as_ref(&self) -> &[u8] {
        self.payload.as_ref()
    }
}

// A sorting function to pick the more recently written entry.
//
// Uses [`sort_step_by_step`], resolving unsorted cases in the manner defined in it.
//
// Returns an ordering.
//
// [`sort_step_by_step`]: #method.sort_step_by_step
// pub fn last_write_wins (a: &Entry, b: &Entry) -> Ordering {
// 	Entry::sort_step_by_step(|_,_| Ordering::Less)(a,b)
// }

// A sorting function to pick the entry with the greater hash.
//
// Uses [`sort_step_by_step`], resolving unsorted cases in the manner defined in it.
//
// Returns an ordering.
//
// [`sort_step_by_step`]: #method.sort_step_by_step
// pub fn sort_by_entry_hash (a: &Entry, b: &Entry) -> Ordering {
// 	Entry::sort_step_by_step(|a,b| a.hash().cmp(&b.hash()))(a,b)
// }

// A sorting helper function to
// 1. first try to sort the two entries using `resolve`,
// 2. if still unsorted (equal), try to sort based on the Lamport clock identifiers of the respective entries,
// 3. sort by the Lamport clocks of the respective e   ntries.
//
// Returns a closure that can be used as a sorting function.
// pub fn sort_step_by_step<F> (resolve: F) -> impl Fn(&Entry,&Entry) -> Ordering
// where F: 'static + Fn(&Entry,&Entry) -> Ordering {
// 	Entry::sort_by_clocks(Entry::sort_by_clock_ids(resolve))
// }

// A sorting helper function to sort by the Lamport clocks of the respective entries.
// In the case the Lamport clocks are equal, tries to sort using `resolve`.
//
// Returns a closure that can be used as a sorting function.
// pub fn sort_by_clocks<F> (resolve: F) -> impl Fn(&Entry,&Entry) -> Ordering
// where F: 'static + Fn(&Entry,&Entry) -> Ordering {
// 	move |a,b| {
// 		//ulet mut diff = a.clock().cmp(&b.clock());
// 		if diff == Ordering::Equal {
// 			diff = resolve(a,b);
// 		}
// 		diff
// 	}
// }

// A sorting helper function to sort by the Lamport clock identifiers of the respective entries.
// In the case the Lamport clocks identifiers are equal, tries to sort using `resolve`.
//
// Returns a closure that can be used as a sorting function.
// pub fn sort_by_clock_ids<F> (resolve: F) -> impl Fn(&Entry,&Entry) -> Ordering
// where F: 'static + Fn(&Entry,&Entry) -> Ordering {
// 	move |a,b| {
// 		let mut diff = a.clock().id().cmp(&b.clock().id());
// 		if diff == Ordering::Equal {
// 			diff = resolve(a,b);
// 		}
// 		diff
// 	}
// }

// A sorting helper function that forbids the sorting function `fn_sort` from
// producing unsorted (equal) cases.
//
// Returns a closure that behaves in the same way as `fn_sort`
// but panics if the two entries given as input are equal.
// pub fn no_zeroes<F>(fn_sort: F) -> impl Fn(&Entry, &Entry) -> Ordering
// where
//     F: 'static + Fn(&Entry, &Entry) -> Ordering,
// {
//     move |a, b| {
//         let diff = fn_sort(a, b);
//         if diff == Ordering::Equal {
//             panic!(
//                 "Your log's tiebreaker function {}",
//                 "has returned zero and therefore cannot be"
//             );
//         }
//         diff
//     }
// }
// Box::new(Entry::no_zeroes(Box::new(Entry::last_write_wins)))

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::identity::{Identity, Signatures};

    //#[test]
    //fn test_multihash_entry() {
    //    let identity = Identity::new(
    //        "bench_multihash_entry",
    //        "public",
    //        Signatures::new("id_signature", "public_signature"),
    //    );

    //    let entry = Entry::new(
    //        &identity,
    //        "A",
    //        b"entryA",
    //        &HashSet::<Multihash>::new(),
    //        None,
    //    );

    //    multihash::Sha2_256::digest(entry);
    //}
}
