//! This crate implements the underlying Operations Log (Oplog) that powers
//! a CRDT.

#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]

/// Entries are IPLD structures that form a graph by their hashes
pub mod entry;
pub mod identity;
pub mod lamport_clock;
pub mod log;
pub mod log_options;

mod util;

#[cfg(test)]
mod tests {
    use super::identity::Identity;
    use super::identity::Signatures;
    use super::log::Log;
    use super::log_options::LogOptions;

    /*********************/
    /* Utility Functions */
    /*********************/

    // Generates a test identity
    fn identity(user: &str, acl: &str) -> Identity {
        Identity::new(
            user,
            acl,
            Signatures::new("id_signature", "public_signature"),
        )
    }

    // Spwans a test in-memory instance of IPFS
    async fn spawn_ipfs() -> ipfs::Ipfs<ipfs::TestTypes> {
        let options = ipfs::IpfsOptions::inmemory_with_generated_keys(false);

        let (ipfs, task) = ipfs::UninitializedIpfs::new(options)
            .await
            .start()
            .await
            .unwrap();
        tokio::spawn(task);

        ipfs
    }

    #[tokio::test]
    async fn append() {
        let ipfs = spawn_ipfs().await;

        let identity = identity("A", "public");
        let options = LogOptions::new().set_id("A");
        let mut log = Log::new(ipfs, identity, &options);

        let _cid = log.append("one").await;
        let _traversal = log.traverse(log.heads()).await;
    }

    #[test]
    #[ignore]
    fn find_heads() {
        // let identity = identity();
        // // let e1 = Entry::new(&identity(), "A", b"entryA", Vec::<String>::new(), None);
        // // let e2 = Entry::new(&identity2(), "B", b"entiiiiiiiiikkio;'ikkookkryB", Vec::<String>::new(), None);
        // // let e3 = Entry::new(&identity3(), "C", b"entryC", Vec::<String>::new(), None);

        // let options = LogOptions::new().set_id("A");

        // let mut log = Log::new(identity, &options);

        // log.append("one").unwrap();
        // assert_eq!(log.heads().len(), 1);
        // log.append("two").unwrap();
        // assert_eq!(log.heads().len(), 1);
        // log.append("three").unwrap();
        // assert_eq!(log.heads().len(), 1);

        // // TODO: Test hashes
        // // println!("{:?}", &log.heads()[0].to_string());
    }

    #[tokio::test]
    async fn traverse() {
        let ipfs = spawn_ipfs().await;

        let identity = identity("A", "public");
        let options = LogOptions::new().set_id("A");
        let mut log = Log::new(ipfs, identity, &options);

        log.append("one").await.unwrap();
        log.append("two").await.unwrap();
        log.append("three").await.unwrap();
        log.append("four").await.unwrap();
        log.append("five").await.unwrap();

        let _values = log.traverse(log.heads());
    }

    #[tokio::test]
    async fn length() {
        let options = ipfs::IpfsOptions::inmemory_with_generated_keys(false);

        let (ipfs, task) = ipfs::UninitializedIpfs::new(options)
            .await
            .start()
            .await
            .unwrap();
        tokio::spawn(task);

        let mut log = Log::new(
            ipfs,
            identity("A", "public"),
            &LogOptions::new().set_id("A"),
        );
        log.append("one").await.unwrap();
        log.append("two").await.unwrap();
        log.append("three").await.unwrap();
        log.append("four").await.unwrap();
        log.append("five").await.unwrap();
        // assert_eq!(log.length().await, 5);
    }

    // //fix comparison after implementing genuine hashing
    // #[test]
    // #[ignore]
    // fn get () {
    // let mut log = Log::new(identity(),LogOptions::new().id("AAA"));
    // log.append("one",None);
    // assert_eq!(log.get(log.values()[0].hash()).unwrap().hash(),"QmUMWpQmAqh4Uws3eSWkELeic1eHTnwzZq3p3VGt1D5Cy9");
    // assert_eq!(log.get("zero"),None);
    // }

    // #[test]
    // #[ignore]
    // fn set_identity () {
    // 	let id1 = identity();
    // 	let mut log = Log::new(id1.clone(),LogOptions::new().id("AAA"));
    // 	log.append("one",None);
    // 	assert_eq!(log.values()[0].clock().id(),id1.pub_key());
    // 	assert_eq!(log.values()[0].clock().time(),1);
    // 	let id2 = identity2();
    // 	// log.set_identity(id2.clone());
    // 	log.append("two",None);
    // 	assert_eq!(log.values()[1].clock().id(),id2.pub_key());
    // 	assert_eq!(log.values()[1].clock().time(),2);
    // 	let id3 = identity3();
    // 	log.append("three",None);
    // 	assert_eq!(log.values()[2].clock().id(),id3.pub_key());
    // 	assert_eq!(log.values()[2].clock().time(),3);
    // }

    // //implement later
    // #[test]
    // fn has () {
    // }

    // //fix comparisons after implementing genuine hashing
    // #[test]
    // #[ignore]
    // fn serialize () {
    // 	let expected = json!({
    // 		"id": "AAA",
    // 		"heads": ["QmREuiyqTuJrcWr5BLrT9d9p8dcvdWvwc4JJMHpKcei4Em"],
    // 	}).to_string();
    // 	let mut log = Log::new(identity(),LogOptions::new().id("AAA"));
    // 	log.append("one",None);
    // 	log.append("two",None);
    // 	log.append("three",None);
    // 	assert_eq!(log.json(),expected);
    // 	//...

    // 	//extra
    // 	// let log2 = Log::from_multihash(ipfs.clone(),identity(),LogOptions::new().id("AAA"),"QmREuiyqTuJrcWr5BLrT9d9p8dcvdWvwc4JJMHpKcei4Em");
    // 	// assert_eq!(log.snapshot(),log2.snapshot());
    // 	// let log3 = Log::from_multihash(identity(),LogOptions::new().id("AAA"),"QmQyM8vsbzs6ibi6DFRhXVFurR1AaFyJkPnnvQTeNEdbZu");
    // 	// assert_ne!(log.snapshot(),log3.snapshot());
    // }

    #[test]
    #[ignore]
    fn values() {
        // let mut log = Log::new(identity(), &LogOptions::new());

        // // Accepts anything that can be represented as a [u8]
        // log.append(b"hello1").unwrap();
        // log.append([100]).unwrap();
        // log.append("hello3").unwrap();

        // assert_eq!(log.length(), 3);

        // println!("{:?}", log.values());

        // // TODO: Reverse order
        // // FIXME: Unreliable ordering in hashmap iter
        // let payload_0 = log.values()[0].payload();
        // let payload_1 = log.values()[1].payload();
        // let payload_2 = log.values()[2].payload();

        // assert_eq!(&payload_0, b"hello1");
        // assert_eq!(payload_1, vec![100]);
        // assert_eq!(&payload_2, b"hello3");
    }

    // #[test]
    // #[ignore]
    // fn log_join () {
    // 	let id = Identity::new("0","1",Signatures::new("2","3"));
    // 	let log_id = "xyz";
    // 	let mut x = Log::new(id.clone(),LogOptions::new().id(log_id));
    // 	x.append("to",None);
    // 	x.append("set",None);
    // 	x.append("your",None);
    // 	x.append("global",None);

    // 	let e2 = Entry::new(id.clone(),log_id,"second",&[],None);
    // 	let e3 = Entry::new(id.clone(),log_id,"third",&[],None);
    // 	let e1 = Entry::new(id.clone(),log_id,"first",&[EntryOrHash::Entry(&e2),EntryOrHash::Entry(&e3)],None);
    // 	let es = &[Rc::new(e1),Rc::new(e2),Rc::new(e3)];
    // 	let mut y = Log::new(id.clone(),LogOptions::new().id(log_id).entries(es));
    // 	y.append("fifth",None);
    // 	y.append("seventh",None);

    // 	let mut z = Log::new(id.clone(),LogOptions::new().id(log_id).entries(es));
    // 	z.append("fourth",None);
    // 	z.append("sixth",None);
    // 	z.append("eighth",None);

    // 	println!("x:\t\t{}\ny:\t\t{}\nz:\t\t{}",x.all(),y.all(),z.all());

    // 	println!("diff z-y\t{:?}",z.diff(&y).iter().map(|x| x.1.hash().to_owned()).collect::<Vec<_>>());
    // 	y.join(&z,None);
    // 	println!("\n<<join z+y = y>\n{}>\n",y);
    // 	println!("----\t\ty\t\t----\n{}",y.entries());
    // 	//println!("diff y-z\t{:?}",y.diff(&z).iter().map(|x| x.1.hash().to_owned()).collect::<Vec<_>>());
    // 	//z.join(&y,None);
    // 	//println!("\n<<join y+z = z>\n{}>",z);
    // 	//println!("----\t\tz\t\t----\n{}",z.entries());

    // 	println!("diff z-y\t{:?}",z.diff(&y).iter().map(|x| x.1.hash().to_owned()).collect::<Vec<_>>());
    // 	y.join(&z,None);
    // 	println!("\n<<join z+y = y>\n{}>\n",y);
    // 	println!("----\t\ty\t\t----\n{}",y.entries());

    // 	println!("y (json)\t{}\n",y.json());
    // 	println!("y (snapshot)\t{}\n",y.snapshot());
    // 	println!("y (buffer)\t{:?}\n",y.buffer());
    // 	assert_eq!(y.json(),String::from_utf8(y.buffer()).unwrap());

    // 	println!("diff y-x\t{:?}",y.diff(&x).iter().map(|x| x.1.hash().to_owned()).collect::<Vec<_>>());
    // 	x.join(&y,Some(10));
    // 	println!("\n<<join y+x = x>\n{}>\n",x);
    // 	println!("----\t\tx\t\t----\n{}",x.entries());
    // }

    // /*
    // #[test]
    // fn ipfs () {
    // 	let client = IpfsClient::default();

    // 	/*
    // 	let data = Cursor::new("tinamämmi");
    // 	let request = client.add(data).map(|r| println!("put {}",r.hash)).map_err(|e| eprintln!("{}",e));
    // 	run(request);

    // 	let mut idpr = DefaultIdentificator::new();
    // 	let id = idpr.create("local_id");
    // 	let mut log = Log::new(id.clone(),LogOptions::new().id("log_id"));
    // 	log.append("first",None);
    // 	log.append("second",None);
    // 	log.append("third",None);
    // 	run(client.add(Cursor::new(log.snapshot())).map(|r| println!("put {}",r.hash)).map_err(|e| eprintln!("{}",e)));
    // 	run(client.object_get("QmQJxSCHs1e3NRSXZeHg86yhHWCTHd26Lx1HFsmqQHkF4R").map(|r| println!("get {}:\n{}","QmQJxSCHs1e3NRSXZeHg86yhHWCTHd26Lx1HFsmqQHkF4R",r.data)).map_err(|e| eprintln!("{}",e)));
    // 	run(client.object_get("QmekwsuyWM853FXJ5SzUW6eQG2LXjp6L8a7xSJf9ZWZW4U").map(|r| println!("get {}:\n{}","QmekwsuyWM853FXJ5SzUW6eQG2LXjp6L8a7xSJf9ZWZW4U",r.data)).map_err(|e| eprintln!("{}",e)));*/

    // 	/*
    // 	let request = client.object_new(Some(ObjectTemplate::UnixFsDir)).map(|r| println!("object: {}",r.hash)).map_err(|e| eprintln!("error: {}",e));
    // 	run(request);*/
    // }*/
}
