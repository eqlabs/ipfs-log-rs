// use std::rc::Rc;
// use std::collections::{ HashSet, HashMap };

// use multihash::Multihash;

// use crate::entry::Entry;

// Returns `true` if the entry at `hash` is the parent of `e1`, otherwise returns `false`.
// pub fn _is_parent(e1: &Entry, hash: &String) -> bool {
//     e1.next().iter().any(|x| x == hash)
// }

// Returns a vector of pointers to all direct and indirect children of `entry` in `entries`.
// pub fn _find_children(entry: &Entry, hashes: &[String]) -> Vec<String> {
//     let mut stack = Vec::new();
//     let mut parent = hashes.iter().find(|hash| _is_parent(entry, hash));
//     while let Some(p) = parent {
//         stack.push(p.clone());
//         let _prev = p;
//         parent = hashes.iter().find(|hash| _is_parent(entry, hash));
//     }
//     // stack.sort_by(|a, b| a.clock().time().cmp(&b.clock().time()));
//     stack
// }

// pub fn find_heads(entries: &[Rc<Entry>]) -> Vec<Multihash> {
//     let mut parents = HashMap::<Multihash, ()>::new();
//     for e in entries {
//         for n in e.next() {
//             parents.insert(n,e.hash());
//         }
//     }
//     let mut heads = Vec::<Rc<Entry>>::new();
//     for e in entries {
//         if !parents.contains_key(&e.hash()) {
//             heads.push(e.clone());
//         }
//     }
//     heads.sort_by(|a, b| {
//         let diff = a.clock().id().cmp(b.clock().id());
//         if diff == std::cmp::Ordering::Equal {
//             std::cmp::Ordering::Less
//         } else {
//             diff
//         }
//     });
//     heads
// }
//
// pub fn find_tails (entries: &[Rc<Entry>]) -> Vec<Rc<Entry>> {
// 	let mut no_nexts = Vec::new();
// 	let mut reverses = HashMap::new();
// 	let mut nexts = HashSet::new();
// 	let mut hashes: HashSet<&str> = HashSet::new();
// 	for e in entries {
// 		if e.next().is_empty() {
// 			no_nexts.push(e.clone());
// 		}
// 		for n in e.next() {
// 			reverses.insert(n,e.clone());
// 			nexts.insert(n);
// 		}
// 		hashes.insert(e.hash());
// 	}
// 	//correct order?
// 	let mut tails = Log::dedup(&nexts.iter().filter(|&&x| !hashes.contains(&x[..])).
// 	map(|x| reverses[x].clone()).chain(no_nexts.into_iter()).collect::<Vec<_>>()[..]);
// 	tails.sort();
// 	tails
// }
//
// pub fn find_tail_hashes (entries: &[Rc<Entry>]) -> Vec<String> {
// 	let mut hashes: HashSet<&str> = HashSet::new();
// 	for e in entries {
// 		hashes.insert(e.hash());
// 	}
// 	let mut ths = Vec::new();
// 	for e in entries {
// 		for i in e.next().len() - 1..0 {
// 			let n = &e.next()[i];
// 			if !hashes.contains(&n[..]) {
// 				ths.push(n.to_owned());
// 			}
// 		}
// 	}
// 	ths.reverse();
// 	ths
// }
//
