#![allow(dead_code)]

mod log;
mod gset;
mod identity;
mod lamport_clock;
mod entry;

#[cfg(test)]
mod tests {
	use std::rc::Rc;
	use std::io::Cursor;

	use ipfs_api::IpfsClient;
	use hyper::rt::Future;

	use super::gset::GSet;
	use super::lamport_clock::LamportClock;
	use super::identity::Identity;
	use super::log::Log;
	use super::log::AdHocAccess;
	use super::entry::Entry;
	use super::entry::EntryOrHash;

	fn identity () -> Identity {
		Identity::new("identity","public","id_signature","public_signature")
	}

	#[test]
	fn set_id () {
		let log = Log::new(identity(),Some("ABC"),AdHocAccess,&[],&[],None,None);
		assert_eq!(log.id(),"ABC");
	}

	#[test]
	fn set_clock_id () {
		let id = identity();
		let log = Log::new(id.clone(),Some("ABC"),AdHocAccess,&[],&[],None,None);
		assert_eq!(log.clock().id(),id.public_key());
	}

	#[test]
	fn set_items () {
		let id = identity();
		let e1 = Entry::create(id.clone(),"A","entryA",&[],Some(LamportClock::new("A")));
		let e2 = Entry::create(id.clone(),"A","entryB",&[],Some(LamportClock::new("B")));
		let e3 = Entry::create(id.clone(),"A","entryC",&[],Some(LamportClock::new("C")));
		let log = Log::new(id.clone(),Some("A"),AdHocAccess,&[e1,e2,e3],&[],None,None);
		assert_eq!(log.len(),3);
		assert_eq!(log.values()[0].payload(),"entryA");
		assert_eq!(log.values()[1].payload(),"entryB");
		assert_eq!(log.values()[2].payload(),"entryC");
	}

	#[test]
	fn set_heads () {
		let id = identity();
		let e1 = Entry::create(id.clone(),"A","entryA",&[],None);
		let e2 = Entry::create(id.clone(),"A","entryB",&[],None);
		let e3 = Entry::create(id.clone(),"A","entryC",&[],None);
		let log = Log::new(id.clone(),Some("B"),AdHocAccess,&[e1,e2,e3.clone()],&[e3.clone()],None,None);
		assert_eq!(log.heads().len(),1);
		assert_eq!(log.heads()[0].hash(),e3.hash());
	}

	#[test]
	#[ignore]
	fn test_gset () {
		let mut x: GSet<i32> = GSet::new();
		assert!(x.is_empty());
		x.insert(2);
		x.insert(3);
		x.insert(5);
		x.insert(8);
		assert!(!x.is_empty());
		assert_eq!(x.len(),4);
		let mut y: GSet<i32> = GSet::new();
		y.insert(4);
		y.insert(5);
		y.insert(10);
		y.insert(12);
		assert!(!x.is_subset(&y));
		assert!(!y.is_subset(&x));
		let z = GSet::union(&x,&y);
		assert_eq!(z.len(),7);
		let mut w = GSet::new();
		w.insert(2);
		w.insert(4);
		w.insert(8);
		assert!(w.is_subset(&z));
		assert!(!z.is_subset(&w));
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
		let acc = AdHocAccess;
		let mut x = Log::new(id.clone(),Some(log_id),acc,&[],&[],None,None);
		x.append("to",None);
		x.append("set",None);
		x.append("your",None);
		x.append("global",None);

		let e2 = Entry::new(id.clone(),log_id,"second",&[],None);
		let e3 = Entry::new(id.clone(),log_id,"third",&[],None);
		let e1 = Entry::new(id.clone(),log_id,"first",&[EntryOrHash::Entry(&e2),EntryOrHash::Entry(&e3)],None);
		let es = &[Rc::new(e1),Rc::new(e2),Rc::new(e3)];
		let mut y = Log::new(id.clone(),Some(log_id),acc,es,&[],None,None);
		y.append("fifth",None);
		y.append("seventh",None);

		let mut z = Log::new(id.clone(),Some(log_id),acc,es,&[],None,None);
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
}
