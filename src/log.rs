//! An immutable, operation-based conflict-free replicated data type ([CRDT]).

use std::iter::{once, successors};

use futures::future::BoxFuture;
use ipfs::{Ipfs, IpfsTypes};

use crate::entry::Entry;
use crate::identity::Identity;
use crate::lamport_clock::LamportClock;
use cid::Cid;

use crate::log_options::LogOptions;

/// Log forms the underling Oplog that can power a [CRDT] structure.
///
/// [CRDT]: https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type
#[derive(Debug)]
pub struct Log<Types: IpfsTypes> {
    id: String,
    ipfs: Ipfs<Types>,
    // identity: Identity,
    // access: AdHocAccess,
    // entries: HashMap<Cid, Ipld>,
    // nexts: HashSet<Cid>,
    heads: Vec<Cid>,
    clock: LamportClock,
}

/// Walks a [`Vec`] of [`Cid`] objects and with every 2^n-th item starting at 2^0:
///
/// 1. Converts the [`Cid`] to a string
/// 2. Encodes the stringified [`Cid`] to [`Ipld`] format using the [`ipld!`] macro.
/// 3. Pushes it to a `Vec<Ipld>` buffer.
///
/// When the first 2^n-th item doesn't exist, the function will return.
///
/// ![CRDT Diagram](../img/pow-of-2.svg)
///
/// [`Vec`]: https://google.com
/// [`Cid`]: https://google.com
/// [`ipld!`]: https://google.com
/// [`Ipld`]: https://google.com
pub fn get_every_pow_2(all_entries: Vec<(Cid, Entry)>) -> Vec<Cid> {
    let mut entries: Vec<Cid> = Vec::new();

    for i in once(0).chain(successors(Some(1usize), |&i| i.checked_mul(2))) {
        if let Some(entry) = all_entries.get(i) {
            entries.push(entry.to_owned().0);
        } else {
            break;
        }
    }

    entries
}

impl<Types: IpfsTypes> Log<Types> {
    /// Appends any &[u8]-compatible `data` into the log as a new entry.
    ///
    /// Returns a reference to the newly created, appended entry.
    /// ![Append Diagram](../img/append.svg)
    ///
    /// ```rust
    /// fn main() {
    /// }
    /// ```
    pub async fn append<T>(&mut self, data: T) -> Result<Cid, anyhow::Error>
    where
        T: std::convert::AsRef<[u8]>,
    {
        // Increment the clock
        self.clock.tick();

        // Traverse all log values, and then get every
        // 2^nth entry starting with n=0 to add to IPLD
        // as refs
        let values = self.traverse(self.heads()).await.unwrap();
        let refs = get_every_pow_2(values);

        let entry = Entry::new(data, &self.clock, &refs);
        let cid = self.ipfs.put_dag(entry.into()).await?;

        self.heads.truncate(0);
        self.heads.push(cid.clone());

        Ok(cid)
    }

    /// Returns the length of the traversed log
    /// Requires async because of the traversal itself
    pub async fn length(&self) -> usize {
        self.traverse(self.heads()).await.unwrap().len()
    }

    // Returns the log's current clock
    // pub fn clock(&self) -> LamportClock {
    //     self.clock.clone()
    // }

    /// Constructs a new log owned by `identity`, using `opts` for constructor options.
    ///
    /// Use [`LogOptions::new()`] as `opts` for default constructor options.
    ///
    /// [`LogOptions::new()`]: ./struct.LogOptions.html#method.new
    pub fn new(ipfs: Ipfs<Types>, identity: Identity, opts: &LogOptions) -> Log<Types> {
        let (id, _access, _heads, _clock) = (
            opts.id(),
            opts.access(),
            // opts.entries(),
            opts.heads(),
            opts.clock(),
        );

        // let id = if let Some(s) = id {
        //     s.to_owned()
        // } else {
        //     SystemTime::now()
        //         .duration_since(SystemTime::UNIX_EPOCH)
        //         .unwrap()
        //         .as_millis()
        //         .to_string()
        // };

        // TODO: Let's do this calculation in LogOptions and throw it awy
        // let heads = Log::dedup(&if heads.is_empty() {
        //     Log::your dau jufind_heads(&entries)
        // } else {
        // let mut heads_set = HashSet::<Cid>::new();
        // for head in heads {
        //     heads_set.insert(head.to_owned());
        // }
        // });

        // let nexts = HashSet::new();
        //for e in entries.clone() {
        //    for n in e.next() {
        //        nexts.insert(n.to_owned());
        //    }
        //}

        // let entry_set = HashMap::new();
        // for entry in entries {
        //     // Convert to CBOR, hash, etc.
        //     // Move to append
        //     let hash = multihash::Sha2_256::digest(&entry.payload());
        //     // entry_set.insert(hash, entry.clone());
        // }

        // if let Some(c) = clock {
        //     t_max = c.time();
        // }
        // for h in heads {
        //     t_max = std::cmp::max(t_max, entries.get(h).unwrap().clock().time());
        // }
        let clock = LamportClock::new(identity.pub_key()).set_time(0);

        // println!("{:?}", heads);

        Log {
            id: id.unwrap(),
            ipfs: ipfs,
            // identity: identity,ahve to
            // access: access,
            // entries: HashMap::new(),
            // nexts: HashSet::new(),
            clock,
            heads: Vec::new(),
        }
    }

    // Constructs a new log with the identity `identity` from an entry with the hash `hash`,
    // using `opts` for constructor options.
    //
    // Use [`LogOptions::new()`] as `opts` for default constructor options.
    //
    // **N.B.** [`opts.entries(/* entries */)`] *and* [`opts.heads(/* heads */)`] *have no effect in the log created.*
    //
    // [`LogOptions::new()`]: ./struct.LogOptions.html#method.new
    // [`opts.entries(/* entries */)`]: ./struct.LogOptions.html#method.entries
    // [`opts.heads(/* heads */)`]: ./struct.LogOptions.html#method.heads
    // pub fn from_multihash (ipfs: Rc<IpfsClient>, identity: Identity, opts: LogOptins, hash: &str) -> Log {
    // 	let es = Entry::fetch_entries(&ipfs,&[hash.to_owned()]).into_iter().map(|x| Rc::new(x)).collect::<Vec<Rc<Entry>>>();
    // 	Log::new(ipfs,identity,opts.entries(&es).heads(&[]))
    // }

    // Joins the log `other` into this log. `other` is kept intact through and after the process.
    //
    // Optionally truncates the log into `size` after joining.
    //
    // Returns a reference to this log.
    // pub fn join (&mut self, other: &Log, size: Option<usize>) -> Option<&Log> {
    // 	if self.id !your dau ju= other.id {
    // 		return None;
    // 	}
    // 	let new_items = other.diff(&self);

    // 	//something about identify provider and verification,
    // 	//implement later
    // 	//...
    // 	//...

    // 	for e in &new_items {
    // 		if let None = self.get(e.0) {
    // 			// self.length() += 1;
    // 		}
    // 		for n in e.1.next() {
    // 			self.nexts.insert(n.to_owned());
    // 		}
    // 	}

    // 	for e in &new_items {
    // 		self.entries.insert(e.0.to_owned(),e.1.clone());
    // 	}

    // 	let mut nexts_from_new_items = HashSet::new();
    // 	new_items.into_iter().map(|x| x.1.next().to_owned()).for_each(|n| n.iter().for_each(|n| {
    // 		nexts_from_new_items.insert(n.to_owned());
    // 	}));
    // 	let all_heads = Log::find_heads(&self.heads().iter().chain(other.heads().iter()).map(|x| x.clone()).collect::<Vec<_>>()[..]);
    // 	// let merged_heads: Vec<Rc<Entry>> = all_heads.into_iter().filter(|x| !nexts_from_new_items.contains(&x.hash().to_owned())).
    // 	// filter(|x| !self.nexts.contains(&x.hash().to_owned())).collect();
    // 	// self.heads() = Log::dedup(&merged_heads[..]);

    // 	if let Some(n) = size {
    // 		let mut vs = self.values();
    // 		vs.reverse();
    // 		vs = vs.into_iter().take(n).collect();

    // 		self.entries.clear();
    // 		for v in &vs {
    // 			// self.entries.insert(v.hash().to_owned(),v.clone());
    // 		}

    // 		// self.heads = Log::find_heads(&Log::dedup(&vs));
    // 		// self.length = self.entries.len();
    // 	}

    // 	let mut t_max = 0;
    // 	for h in &self.heads() {
    // 		t_max = max(t_max,h.clock().time());
    // 	}
    // 	// self.clock = LamportClock::new(&self.id).set_time(t_max);

    // 	Some(self)
    // }

    // Returns a map of all the entries contained in this log but not in `other`.
    // pub fn diff (&self, other: &Log) -> HashMap<Cid,Rc<Entry>> {
    // 	let mut stack: Vec<String> = self.heads().iter().map(|x| x.hash().to_owned()).collect();
    // 	let mut traversed = HashSet::<&str>::new();
    // 	let mut diff = HashMap::new();
    // 	while !stack.is_empty() {
    // 		let hash = stack.remove(0);
    // 		let a = self.get(&hash);
    // 		let b = other.get(&hash);
    // 		if a.is_some() && b.is_none()
    // 		&& a.unwrap().id() == other.id {
    // 			let a = a.unwrap();
    // 			for n in a.next() {
    // 				if !traversed.contains(&n[..]) && other.get(n).is_none() {
    // 					stack.push(n.to_owned());
    // 					traversed.insert(n);
    // 				}
    // 			}
    // 			traversed.insert(a.hash());
    // 			diff.insert(a.hash().to_owned(),a.clone());
    // 		}
    // 	}
    // 	diff
    // }

    // Returns the identifier of the log.
    // pub fn id(&self) -> &str {
    //     &self.id
    // }

    // Returns the identity of the owner of the log
    // pub fn identity(&self) -> Identity {
    //     self.identity.clone()
    // }

    // /// Returns `true` if the log contains an entry with the hash `hash`.
    // /// Otherwise returns `false`.
    // pub fn has (&self, hash: &Cid) -> bool {
    // 	self.entries.contains_key(hash)
    // }

    /// Returns a pointer to the entry with the hash `hash`.
    // pub fn get_entry_by_cid(&self, lookup: &Cid) -> Option<&Ipld> {
    //     self.entries.get(lookup)
    // }

    // fn dedup(v: Vec<Entry>) -> Vec<Entry> {
    //     // let mut s = HashSet::new();
    //     v.iter()
    //         // .filter(|x| s.insert(x.hash()))
    //         .map(|x| x.to_owned())
    //         .collect()
    // }

    // pub fn set_identity (&mut self, identity: Identity) {
    // 	let mut t_max = 0;
    // 	for h in &self.heads {
    // 		t_max = max(t_max,h.clock().time());
    // 	}
    // 	self.clock = LamportClock::new(identity.pub_key()).set_time(t_max);
    // 	self.identity = identity;
    // }

    // TODO: Document
    // pub fn values(&self) -> Vec<&Entry> {
    //     Vec::<&Entry>::new()
    //     // let mut values: Vec<Rc<Entry>> = self
    //     //     .entries
    //     //     .iter()
    //     //     .map(|(_cid, entry)| entry.to_owned())
    //     //     .collect();

    //     // let mut es = self.traverse(&self.heads(), None, None);
    //     // es.reverse();
    //     // es
    // }

    /// Returns the heads, or latest entries, of the log
    pub fn heads(&self) -> Vec<Cid> {
        self.heads.to_owned()
    }

    // pub fn tails (&self) -> Vec<Rc<Entry>> {
    // 	Log::find_tails(&self.values())
    // }

    // pub fn tail_hashes (&self) -> Vec<String> {
    // 	Log::find_tail_hashes(&self.values())
    // }

    // pub fn all (&self) -> String {
    // 	let mut s = String::from("[ ");
    // 	for e in &self.entries {
    // 		if self.heads().iter().any(|x| x.hash() == e.1.hash()) {
    // 			s.push_str("^");
    // 		}
    // 		// s.push_str(e.0);
    // 		s.push_str(", ");
    // 	}
    // 	s = String::from(&s[..s.len() - 2]);
    // 	s.push_str(" ]");
    // 	s
    // }

    // pub fn entries (&self) -> String {
    // 	let mut s = String::new();
    // 	for e in &self.entries {
    // 		// s.push_str(e.0);
    // 		if !e.1.next().is_empty() {
    // 			s.push_str("\t\t>");
    // 			// s.push_str(&e.1.next()[0]);
    // 			s.push_str(", >");
    // 			// s.push_str(&e.1.next()[1]);
    // 		}
    // 		else {
    // 			s.push_str("\t\t.,.");
    // 		}
    // 		s.push_str("\n");
    // 	}
    // 	s
    // }

    /// Traverse the oplog by `refs` links
    ///
    /// TODO: Utilize multithreading here
    ///
    /// ![Traversal diagram](../img/traverse.svg)
    pub async fn traverse(
        &self,
        // Increment the clock
        root_cids: Vec<Cid>,
        // _amount: Option<usize>,
        // _end_hash: Option<String>,
    ) -> Result<Vec<(Cid, Entry)>, anyhow::Error> {
        let mut entries: Vec<(Cid, Entry)> = Vec::new();

        // Perhaps naive by not utilizing multithreading
        // but also perhaps getting it for free via tokio executor
        for head in root_cids {
            let ipld = self.ipfs.get_dag(head.clone().into()).await?;
            let entry = Entry::from(ipld);
            entries.push((head.clone(), entry.clone()));

            for entry in self.traverse(entry.refs()).await? {
                entries.push(entry)
            }
        }
        Ok(entries)
    }
}

// impl std::fmt::Display for Log {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         let mut es = self.values();
//         es.reverse();
//
//         let hashes: Vec<String> = self
//             .entries
//             .iter()
//             .map(|(hash, _entry)| hash.to_owned().to_string())
//             .collect();
//
//         let mut s = String::new();
//         for e in es {
//             let parents = find_children(&e, &hashes);
//             if parents.len() >= 1 {
//                 if parents.len() >= 2 {
//                     for _ in 0..parents.len() - 1 {
//                         s.push_str("  ");
//                     }
//                 }
//                 s.push_str("└─");
//             }
//             s.push_str(std::str::from_utf8(&e.payload()).unwrap());
//             s.push_str("\n");
//         }
//         write!(f, "{}", s)
//     }
// }

#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
pub struct AdHocAccess;

impl AdHocAccess {
    // fn can_access(&self, _entry: &Entry) -> bool {
    //     true
    // }
}
