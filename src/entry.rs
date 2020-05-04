use crate::lamport_clock::LamportClock;
use cid::Cid;
use libipld::{ipld, Ipld};

/// In `ipfs_log` an Entry is a serialization / deserialization
/// interface between Rust types and IPLD
#[derive(Debug, Clone)]
pub struct Entry {
    ipld: Ipld,
}

impl Entry {
    /// Takes a clock value, data, and refs and creates
    /// an IPLD structure.
    pub fn new<T>(data: T, clock: &LamportClock, ref_cids: &Vec<Cid>) -> Entry
    where
        T: std::convert::AsRef<[u8]>,
    {
        let refs: Vec<Ipld> = ref_cids.into_iter().map(|cid| ipld!(cid)).collect();

        let ipld: Ipld = ipld!({
            "clock": {
                "id": clock.id(),
                "time": clock.time()
            },
            "refs": refs,
            "payload": data.as_ref()
        });

        Entry { ipld }
    }

    /// Constructs a new [`LamportClock`] from the Entry IPLD's "clock" path
    pub fn clock(&self) -> LamportClock {
        let clock = self.ipld.get("clock").unwrap();

        let id = match clock.get("id").unwrap() {
            Ipld::String(id) => id,
            _ => "",
        };

        let time = match clock.get("time").unwrap() {
            Ipld::Integer(time) => time.to_owned() as u64,
            _ => 0u64,
        };

        LamportClock::new(id).set_time(time as u64)
    }

    /// Gets a Vec<Cid> from the Entry IPLD "refs" path
    pub fn refs(&self) -> Vec<Cid> {
        let refs: Vec<Ipld> = match self.ipld.get("refs").unwrap() {
            Ipld::List(cid_vec) => cid_vec.to_owned(),
            _ => Vec::new(),
        };

        refs.into_iter()
            .map(|ipld| match ipld {
                Ipld::Link(cid) => cid,
                _ => panic!("unknown cid format"),
            })
            .collect()
    }

    /// Gets a Box<[u8]> from the Entry IPLD "payload" path
    pub fn payload(&self) -> Box<[u8]> {
        match self.ipld.get("payload").unwrap() {
            Ipld::Bytes(bytes) => bytes.to_owned().into_boxed_slice(),
            _ => Vec::new().into_boxed_slice(),
        }
    }
}

impl From<Entry> for Ipld {
    fn from(entry: Entry) -> Ipld {
        entry.ipld
    }
}

// TODO: validate that IPLD has all the required bits with TryFrom
impl From<Ipld> for Entry {
    fn from(ipld: Ipld) -> Entry {
        Entry { ipld }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cid::{Cid, Codec, Version};
    use multihash::Sha2_256;

    #[test]
    fn new_entry() {
        let entry = Entry::new(b"", &LamportClock::new("A"), &Vec::new());
        if let Ipld::Map(_ipld) = entry.ipld {
            assert!(true)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn ipld_from() {
        let entry = Entry::new(b"", &LamportClock::new("A"), &Vec::new());
        let ipld = Ipld::from(entry.clone());
        assert_eq!(entry.ipld, ipld);
    }

    #[test]
    fn get_clock() {
        let entry = Entry::new(b"", &LamportClock::new("A"), &Vec::new());
        assert_eq!(entry.clock().id(), "A")
    }

    #[test]
    fn get_refs() {
        let h1 = Sha2_256::digest(b"1");
        let cid1 = Cid::new(Version::V1, Codec::DagProtobuf, h1).unwrap();

        let h2 = Sha2_256::digest(b"2");
        let cid2 = Cid::new(Version::V1, Codec::DagProtobuf, h2).unwrap();

        let h3 = Sha2_256::digest(b"3");
        let cid3 = Cid::new(Version::V1, Codec::DagProtobuf, h3).unwrap();

        let entry = Entry::new(
            b"",
            &LamportClock::new(""),
            &vec![cid1.clone(), cid2.clone(), cid3.clone()],
        );
        assert_eq!(entry.refs()[0], cid1);
        assert_eq!(entry.refs()[1], cid2);
        assert_eq!(entry.refs()[2], cid3);
    }
}
