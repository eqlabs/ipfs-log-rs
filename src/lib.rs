//! This crate implements the underlying Operations Log (Oplog) that powers
//! a CRDT.

#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]

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
    use super::log::{Log};
    use super::log_options::LogOptions;

    fn identity1() -> Identity {
        Identity::new(
            "userA",
            "public",
            Signatures::new("id_signature", "public_signature"),
        )
    }

    // fn identity2() -> Identity {
    //     Identity::new(
    //         "userB",
    //         "public",
    //         Signatures::new("id_signature", "public_signature"),
    //     )
    // }

    // fn identity3() -> Identity {
    //     Identity::new(
    //         "userC",
    //         "public",
    //         Signatures::new("id_signature", "public_signature"),
    //     )
    // }

    #[test]
    #[ignore]
    fn find_heads() {
        // let identity = identity1();
        // // let e1 = Entry::new(&identity1(), "A", b"entryA", Vec::<String>::new(), None);
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

    use std::path::PathBuf;

    #[tokio::test]
    async fn traverse() {
       let options = ipfs::IpfsOptions::inmemory_with_generated_keys(false);

       let (ipfs, task) = ipfs::UninitializedIpfs::new(options)
           .await
           .start()
           .await
           .unwrap();
       tokio::spawn(task);

       let identity = identity1();
       let options = LogOptions::new().set_id("A");
       let mut log = Log::new(&ipfs, identity, &options);

       log.append("one").await.unwrap();
       log.append("two").await.unwrap();
       log.append("three").await.unwrap();
       log.append("four").await.unwrap();
       log.append("five").await.unwrap();

       let _values = log.traverse(log.heads());
    }

    #[test]
    #[ignore]
    fn to_string() {
        // let _expected = "five\n└─four\n  └─three\n    └─two\n      └─one\n";
        // let mut log = Log::new(identity1(), &LogOptions::new().set_id("A"));
        // log.append("one").unwrap();
        // log.append("two").unwrap();
        // log.append("three").unwrap();
        // log.append("four").unwrap();
        // log.append("five").unwrap();
        // assert_eq!(log.length(), 5);
    }

    // //fix comparison after implementing genuine hashing
    // #[test]
    // #[ignore]
    // fn get () {
    // let mut log = Log::new(identity1(),LogOptions::new().id("AAA"));
    // log.append("one",None);
    // assert_eq!(log.get(log.values()[0].hash()).unwrap().hash(),"QmUMWpQmAqh4Uws3eSWkELeic1eHTnwzZq3p3VGt1D5Cy9");
    // assert_eq!(log.get("zero"),None);
    // }

    // #[test]
    // #[ignore]
    // fn set_identity () {
    // 	let id1 = identity1();
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
    // 	let mut log = Log::new(identity1(),LogOptions::new().id("AAA"));
    // 	log.append("one",None);
    // 	log.append("two",None);
    // 	log.append("three",None);
    // 	assert_eq!(log.json(),expected);
    // 	//...

    // 	//extra
    // 	// let log2 = Log::from_multihash(ipfs.clone(),identity1(),LogOptions::new().id("AAA"),"QmREuiyqTuJrcWr5BLrT9d9p8dcvdWvwc4JJMHpKcei4Em");
    // 	// assert_eq!(log.snapshot(),log2.snapshot());
    // 	// let log3 = Log::from_multihash(identity1(),LogOptions::new().id("AAA"),"QmQyM8vsbzs6ibi6DFRhXVFurR1AaFyJkPnnvQTeNEdbZu");
    // 	// assert_ne!(log.snapshot(),log3.snapshot());
    // }

    #[test]
    #[ignore]
    fn values() {
        // let mut log = Log::new(identity1(), &LogOptions::new());

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
