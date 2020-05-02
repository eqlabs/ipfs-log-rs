//! An immutable, operation-based conflict-free replicated data type ([CRDT]).

use std::collections::HashMap;
use std::iter::{once, successors};

use ipfs::{ Ipfs, IpfsTypes };
use libipld::{ipld, Ipld};

use crate::entry::Entry;
use crate::identity::Identity;
use crate::lamport_clock::LamportClock;
use cid::Cid;

use crate::log_options::LogOptions;


/// [CRDT]: https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type
/// ![CRDT Diagram](./doc/img/pow-of-2.svg)
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

/// Gets every pow 2
fn get_every_pow_2(all_entries: Vec<Cid>) -> Vec<Ipld> {
    let mut entries = Vec::new();

    for i in once(0).chain(successors(Some(1usize), |&i| i.checked_mul(2))) {
        if let Some(entry) = all_entries.get(i) {
            let ipld = ipld!(entry.to_string());
            entries.push(ipld);
        }
    }

    entries
}

impl<Types: IpfsTypes> Log<Types> {
    /// Appends `data` into the log as a new entry.
    ///
    /// Returns a reference to the newly creTypesated, appended entry.
    pub async fn append<T>(&mut self, data: T) -> Result<(), anyhow::Error>
    where
        T: std::convert::AsRef<[u8]>,
    {
        // Increment the clock
        self.clock.tick();

        let refs = get_every_pow_2(self.traverse(self.heads()).await);

        let ipld = ipld!({
            "clock": {
                "id": self.clock.id(),
                "time": self.clock.time()
            },
            "refs": refs,
            "payload": data.as_ref()
        });
        // let bytes = DagCborCodec::encode(&ipld)?;
        // let hash = multihash::Sha2_256::digest(&bytes);
        // let cid = Cid::new(cid::Version::V1, cid::Codec::DagCBOR, hash)?;



        println!("{:?}", &ipld);
        let cid = self.ipfs.put_dag(ipld).await?;
        println!("{:?}", &cid.to_string());

        self.heads.truncate(0);
        self.heads.push(cid);

        Ok(())
    }

    fn length(&self) -> usize {
        //self.entries.len()
        0
    }

    /// Returns the log's current clock
    pub fn clock(&self) -> LamportClock {
        self.clock.clone()
    }

    /// Constructs a new log owned by `identity`, using `opts` for constructor options.
    ///
    /// Use [`LogOptions::new()`] as `opts` for default constructor options.
    ///
    /// [`LogOptions::new()`]: ./struct.LogOptions.html#method.new
    pub fn new (
        ipfs: &Ipfs<Types>,
        identity: Identity,
        opts: &LogOptions,
    ) -> Log<Types> {
        let (id, _access, _entries, _heads, _clock) = (
            opts.id(),
            opts.access(),
            opts.entries(),
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
        //     Log::find_heads(&entries)
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
            ipfs: ipfs.clone(),
            // identity: identity,
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
    // pub fn from_multihash (ipfs: Rc<IpfsClient>, identity: Identity, opts: LogOptions, hash: &str) -> Log {
    // 	let es = Entry::fetch_entries(&ipfs,&[hash.to_owned()]).into_iter().map(|x| Rc::new(x)).collect::<Vec<Rc<Entry>>>();
    // 	Log::new(ipfs,identity,opts.entries(&es).heads(&[]))
    // }

    // Joins the log `other` into this log. `other` is kept intact through and after the process.
    //
    // Optionally truncates the log into `size` after joining.
    //
    // Returns a reference to this log.
    // pub fn join (&mut self, other: &Log, size: Option<usize>) -> Option<&Log> {
    // 	if self.id != other.id {
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

    /// Returns the identifier of the log.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the identity of the owner of the log
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

    /// TODO: Document
    pub fn values(&self) -> Vec<&Entry> {
        Vec::<&Entry>::new()
        // let mut values: Vec<Rc<Entry>> = self
        //     .entries
        //     .iter()
        //     .map(|(_cid, entry)| entry.to_owned())
        //     .collect();

        // let mut es = self.traverse(&self.heads(), None, None);
        // es.reverse();
        // es
    }

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

    /// Traverse the oplog by nexts / refs
    /// TODO: Utilize multithreading here
    pub async fn traverse(
        &self,
        // Increment the clock
        heads: Vec<Cid>,
        // _amount: Option<usize>,
        // _end_hash: Option<String>,
    ) -> Vec<Cid> {
        let mut entries = Vec::new();

        // Perhaps not naive by not utilizing multithreading
        // but also perhaps getting it for free via tokio executor
        for head in heads {
            entries.push(head.clone());
            let ipld = self.ipfs.get_dag(head.into()).await;
            println!("{:?}", ipld);
        }

        entries
        // self.entries.iter().collect()
        // let mut stack = roots.to_owned();
        // stack.sort_by(|a, b| (self.sort())(a, b));
        // stack.reverse();

        // 	let mut traversed = HashSet::<&str>::new();
        // 	let mut result = Vec::new();
        // 	let mut count = 0;

        // 	while !stack.is_empty() && (amount.is_none() || count < amount.unwrap()) {
        // 		let e = stack.remove(0);
        // 		let hash = e.hash().to_owned();
        // 		count += 1;
        // 		for h in e.next() {
        // 			// if let Some(e) = self.get(h) {
        // 			// 	if !traversed.contains(e.hash()) {
        // 			// 		stack.insert(0,e.clone());
        // 			// 		stack.sort_by(|a,b| (self.sort())(a,b));
        // 			// 		stack.reverse();
        // 			// 		traversed.insert(e.hash());
        // 			// 	}
        // 			// }
        // 		}
        // 		result.push(e);

        // 		// if let Some(ref eh) = end_hash {
        // 		// 	if eh == &hash {
        // 		// 		break;
        // 		// 	}
        // 		// }
        // 	}

        // 	result
    }

    // pub fn json (&self) -> String {
    // 	let mut hs = self.heads().to_owned();
    // 	hs.sort_by(|a,b| (self.sort())(a,b));
    // 	hs.reverse();
    // 	json!({
    // 		"id": self.id,
    // 		"heads": hs.into_iter().map(|x| x.hash().to_owned()).collect::<Vec<_>>(),
    // 	}).to_string()
    // }

    // pub fn snapshot (&self) -> String {
    // 	let hs = self.heads().to_owned();
    // 	let vs = self.values().to_owned();
    // 	json!({
    // 		"id": self.id,
    // 		//"heads": hs.into_iter().map(|x| serde_json::to_string(&*x).unwrap()).collect::<Vec<_>>(),
    // 		//"values": vs.into_iter().map(|x| serde_json::to_string(&*x).unwrap()).collect::<Vec<_>>(),
    // 	}).to_string()
    // }

    // pub fn buffer (&self) -> Vec<u8> {
    // 	self.json().into_bytes()
    // }

    // Fetches all the entries with the hashes in `hashes` and all their parents from the IPFS client `ipfs`.
    //
    // Returns a vector of entries.
    // pub fn fetch_entries (ipfs: &IpfsClient, hashes: &[String]) -> Vec<Entry> {
    // 	let hashes = Arc::new(Mutex::new(hashes.to_vec()));
    // 	let mut es = Vec::new();
    // 	loop {
    // 		let mut result = Vec::new();
    // 		while !hashes.lock().unwrap().is_empty() {
    // 			let h = hashes.lock().unwrap().remove(0);
    // 			let hashes_clone = hashes.clone();
    // 			result.push(Entry::from_multihash(ipfs,&h).
    // 			map(move |x| {
    // 				for n in &x.next {
    // 					hashes_clone.lock().unwrap().push(n.to_owned());
    // 				}
    // 				x
    // 			}));
    // 		}
    // 		es = es.into_iter().chain(Runtime::new().unwrap().block_on(join_all(result)).
    // 		unwrap().into_iter()).collect::<Vec<Entry>>();
    // 		if hashes.lock().unwrap().is_empty() {
    // 			break;
    // 		}
    // 	}
    // 	es
    // }
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
