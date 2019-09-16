#![allow(dead_code)]

mod log;
mod identity;
mod lamport_clock;
mod entry;

#[cfg(test)]
mod tests {
	use std::rc::Rc;
	use std::io::Cursor;

	use ipfs_api::IpfsClient;
	use hyper::rt::Future;
	use serde_json::json;
	use sha2::{Sha256,Digest};
	use secp256k1::{Secp256k1,Message};
	use rand::rngs::OsRng;

	use super::lamport_clock::LamportClock;
	use super::identity::Identity;
	use super::log::Log;
	use super::log::LogOptions;
	use super::entry::Entry;
	use super::entry::EntryOrHash;

	fn identity1 () -> Identity {
		Identity::new("userA","public","id_signature","public_signature")
	}

	fn identity2 () -> Identity {
		Identity::new("userB","public","id_signature","public_signature")
	}

	fn identity3 () -> Identity {
		Identity::new("userC","public","id_signature","public_signature")
	}

	#[test]
	fn set_id () {
		let log = Log::new(identity1(),LogOptions::new().id("ABC"));
		assert_eq!(log.id(),"ABC");
	}

	#[test]
	fn set_clock_id () {
		let id = identity1();
		let log = Log::new(id.clone(),LogOptions::new().id("ABC"));
		assert_eq!(log.clock().id(),id.public_key());
	}

	#[test]
	fn set_items () {
		let id = identity1();
		let e1 = Entry::create(id.clone(),"A","entryA",&[],Some(LamportClock::new("A")));
		let e2 = Entry::create(id.clone(),"A","entryB",&[],Some(LamportClock::new("B")));
		let e3 = Entry::create(id.clone(),"A","entryC",&[],Some(LamportClock::new("C")));
		let log = Log::new(id,LogOptions::new().id("A").entries(&[e1,e2,e3]));
		assert_eq!(log.len(),3);
		assert_eq!(log.values()[0].payload(),"entryA");
		assert_eq!(log.values()[1].payload(),"entryB");
		assert_eq!(log.values()[2].payload(),"entryC");
	}

	#[test]
	fn set_heads () {
		let id = identity1();
		let e1 = Entry::create(id.clone(),"A","entryA",&[],None);
		let e2 = Entry::create(id.clone(),"A","entryB",&[],None);
		let e3 = Entry::create(id.clone(),"A","entryC",&[],None);
		let log = Log::new(id,LogOptions::new().id("B").entries(&[e1,e2,e3.clone()]).heads(&[e3.clone()]));
		assert_eq!(log.heads().len(),1);
		assert_eq!(log.heads()[0].hash(),e3.hash());
	}

	#[test]
	fn find_heads () {
		let id = identity1();
		let e1 = Entry::create(id.clone(),"A","entryA",&[],None);
		let e2 = Entry::create(id.clone(),"A","entryB",&[],None);
		let e3 = Entry::create(id.clone(),"A","entryC",&[],None);
		let log = Log::new(id,LogOptions::new().id("A").entries(&[e1.clone(),e2.clone(),e3.clone()]));
		assert_eq!(log.heads().len(),3);
		assert_eq!(log.heads()[2].hash(),e1.hash());
		assert_eq!(log.heads()[1].hash(),e2.hash());
		assert_eq!(log.heads()[0].hash(),e3.hash());
	}

	#[test]
	fn to_string () {
		let expected = "five\n└─four\n  └─three\n    └─two\n      └─one\n";
		let mut log = Log::new(identity1(),LogOptions::new().id("A"));
		log.append("one",None);
		log.append("two",None);
		log.append("three",None);
		log.append("four",None);
		log.append("five",None);
		assert_eq!(log.to_string(),expected);
	}

	//fix comparison after implementing genuine hashing
	#[test]
	fn get () {
		let mut log = Log::new(identity1(),LogOptions::new().id("AAA"));
		log.append("one",None);
		assert_eq!(log.get(log.values()[0].hash()).unwrap().hash(),"one");
		assert_eq!(log.get("zero"),None);
	}

	#[test]
	fn set_identity () {
		let id1 = identity1();
		let mut log = Log::new(id1.clone(),LogOptions::new().id("AAA"));
		log.append("one",None);
		assert_eq!(log.values()[0].clock().id(),id1.public_key());
		assert_eq!(log.values()[0].clock().time(),1);
		let id2 = identity2();
		log.set_identity(id2.clone());
		log.append("two",None);
		assert_eq!(log.values()[1].clock().id(),id2.public_key());
		assert_eq!(log.values()[1].clock().time(),2);
		let id3 = identity3();
		log.append("three",None);
		assert_eq!(log.values()[2].clock().id(),id3.public_key());
		assert_eq!(log.values()[2].clock().time(),3);
	}

	//implement later
	#[test]
	fn has () {
	}

	//fix comparisons after implementing genuine hashing
	#[test]
	fn serialize () {
		let expected = json!({
			"id": "AAA",
			"heads": ["three"],
		}).to_string();
		let mut log = Log::new(identity1(),LogOptions::new().id("AAA"));
		log.append("one",None);
		log.append("two",None);
		log.append("three",None);
		assert_eq!(log.json(),expected);
		//...
	}

	#[test]
	fn values () {
		let mut log = Log::new(identity1(),LogOptions::new());
		assert_eq!(log.len(),0);
		log.append("hello1",None);
		log.append("hello2",None);
		log.append("hello3",None);
		assert_eq!(log.len(),3);
		assert_eq!(log.values()[0].payload(),"hello1");
		assert_eq!(log.values()[1].payload(),"hello2");
		assert_eq!(log.values()[2].payload(),"hello3");
	}

	#[test]
	#[ignore]
	fn test_clock () {
		let mut x = LamportClock::new("0000");
		let y = LamportClock::new("0001");
		let mut z = LamportClock::new("0002");
		assert!(x < y);
		assert!(y < z);
		z.tick();
		x.merge(&z);
		assert!(x > y);
		let w = LamportClock::new("0003").set_time(4);
		assert!(x < w);
		for _ in 0..3 {
			x.tick();
		}
		assert!(x < w);
		x.tick();
		assert!(x > w);
	}

	#[test]
	#[ignore]
	fn log_join () {
		let id = Identity::new("0","1","2","3");
		let log_id = "xyz";
		let mut x = Log::new(id.clone(),LogOptions::new().id(log_id));
		x.append("to",None);
		x.append("set",None);
		x.append("your",None);
		x.append("global",None);

		let e2 = Entry::new(id.clone(),log_id,"second",&[],None);
		let e3 = Entry::new(id.clone(),log_id,"third",&[],None);
		let e1 = Entry::new(id.clone(),log_id,"first",&[EntryOrHash::Entry(&e2),EntryOrHash::Entry(&e3)],None);
		let es = &[Rc::new(e1),Rc::new(e2),Rc::new(e3)];
		let mut y = Log::new(id.clone(),LogOptions::new().id(log_id).entries(es));
		y.append("fifth",None);
		y.append("seventh",None);

		let mut z = Log::new(id.clone(),LogOptions::new().id(log_id).entries(es));
		z.append("fourth",None);
		z.append("sixth",None);
		z.append("eighth",None);

		println!("x:\t\t{}\ny:\t\t{}\nz:\t\t{}",x.all(),y.all(),z.all());

		println!("diff z-y\t{:?}",z.diff(&y).iter().map(|x| x.1.hash().to_owned()).collect::<Vec<_>>());
		y.join(&z,None);
		println!("\n<<join z+y = y>\n{}>\n",y);
		println!("----\t\ty\t\t----\n{}",y.entries());
		//println!("diff y-z\t{:?}",y.diff(&z).iter().map(|x| x.1.hash().to_owned()).collect::<Vec<_>>());
		//z.join(&y,None);
		//println!("\n<<join y+z = z>\n{}>",z);
		//println!("----\t\tz\t\t----\n{}",z.entries());

		println!("diff z-y\t{:?}",z.diff(&y).iter().map(|x| x.1.hash().to_owned()).collect::<Vec<_>>());
		y.join(&z,None);
		println!("\n<<join z+y = y>\n{}>\n",y);
		println!("----\t\ty\t\t----\n{}",y.entries());

		println!("y (json)\t{}\n",y.json());
		println!("y (snapshot)\t{}\n",y.snapshot());
		println!("y (buffer)\t{:?}\n",y.buffer());
		assert_eq!(y.json(),String::from_utf8(y.buffer()).unwrap());

		println!("diff y-x\t{:?}",y.diff(&x).iter().map(|x| x.1.hash().to_owned()).collect::<Vec<_>>());
		x.join(&y,Some(10));
		println!("\n<<join y+x = x>\n{}>\n",x);
		println!("----\t\tx\t\t----\n{}",x.entries());
	}

	#[test]
	#[ignore]
	fn ipfs () {
		let client = IpfsClient::default();
		let data = Cursor::new("tinamämmi");
		let request = client.add(data).map(|r| println!("ipfs/{}",r.hash)).map_err(|e| eprintln!("{}",e));
		hyper::rt::run(request);
	}

	#[test]
	fn keys () {
		let secp = Secp256k1::new();
		let mut rng = OsRng::new().expect("OsRng");
		let (secret,public) = secp.generate_keypair(&mut rng);

		/*
		let msg = Message::from_slice(&[0x64;32]).expect("32 bytes");
		let sig = secp.sign(&msg,&secret);
		println!("{}\n*\n{:?}\n=\n{}",&secret,&msg,&sig);
		assert!(secp.verify(&msg,&sig,&public).is_ok());*/

		let id_key = ("myid",secret.clone());
		let mut hasher = Sha256::new();
		hasher.input(id_key.0.as_bytes());
		let mut dig = hasher.result();
		let id_sign = secp.sign(&Message::from_slice(&dig).unwrap(),&secret);
		let mut pub_id = public.clone().to_string();
		pub_id.push_str(&id_sign.clone().to_string());
		let mut hasher = Sha256::new();
		hasher.input(pub_id.as_bytes());
		dig = hasher.result();
		let pub_sign = secp.sign(&Message::from_slice(&dig).unwrap(),&secret);
		let id = Identity::new(&secret.to_string(),&public.to_string(),&id_sign.to_string(),&pub_sign.to_string());
		println!("\n({},\n{},\n{},\n{})\n",secret,public,id_sign,pub_sign);
	}
}
