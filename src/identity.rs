use std::collections::HashMap;
use std::cmp::Ordering;

use sha2::{Sha256,Digest};
use secp256k1::{Secp256k1,Message};
use rand::rngs::OsRng;

pub trait IdAndKey {
	fn id (&self) -> &str;
	fn pub_key (&self) -> &str;
}

#[derive(Eq,PartialEq,Clone)]
pub struct Signatures {
	id: String,
	pub_key: String,
}

impl IdAndKey for Signatures {
	fn id (&self) -> &str {
		&self.id
	}

	fn pub_key (&self) -> &str {
		&self.pub_key
	}
}

#[derive(Eq,PartialEq,Clone)]
pub struct Identity {
	id: String,
	pub_key: String,
	signatures: Signatures,
	//type,
	//provider,
}

impl Identity {
	pub fn new (id: &str, pub_key: &str, id_sign: &str, pub_sign: &str) -> Identity {
		Identity {
			id: id.to_owned(),
			pub_key: pub_key.to_owned(),
			signatures: Signatures {
				id: id_sign.to_owned(),
				pub_key: pub_sign.to_owned(),
			},
		}
	}

	pub fn signatures (&self) -> &Signatures {
		&self.signatures
	}
}

impl IdAndKey for Identity {
	fn id (&self) -> &str {
		&self.id
	}

	fn pub_key (&self) -> &str {
		&self.pub_key
	}
}

impl Ord for Identity {
	fn cmp (&self, other: &Self) -> Ordering {
		self.id.cmp(&other.id)
	}
}

impl PartialOrd for Identity {
	fn partial_cmp (&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub trait Identificator {
	fn create (&mut self, id: &str) -> Identity;
}

pub struct OrbitDbIdentificator {
	keystore: HashMap<String,String>,
}

impl OrbitDbIdentificator {
	pub fn new () -> OrbitDbIdentificator {
		OrbitDbIdentificator {
			keystore: HashMap::new(),
		}
	}

	fn put (&mut self, k: &str, v: &str) {
		self.keystore.insert(k.to_owned(),v.to_owned());
	}

	pub fn get (&self, k: &str) -> Option<&String> {
		self.keystore.get(k)
	}
}

impl Identificator for OrbitDbIdentificator {
	fn create (&mut self, id: &str) -> Identity {
		let secp = Secp256k1::new();
		let mut rng = OsRng::new().unwrap();
		let (secret_key,id_hash) = secp.generate_keypair(&mut rng);
		let (sk,ih) = (&secret_key.to_string(),&id_hash.serialize_uncompressed().iter().map(|&x| format!("{:02x}",x)).collect::<String>());

		self.put(id,sk);
		self.put(sk,ih);

		let (middle_key,public_key) = secp.generate_keypair(&mut rng);
		let (mk,pk) = (&middle_key.to_string(),&public_key.serialize_uncompressed().iter().map(|&x| format!("{:02x}",x)).collect::<String>());
		self.put(ih,mk);
		self.put(mk,pk);

		let mut hasher = Sha256::new();
		hasher.input(ih.as_bytes());
		let mut dig = hasher.result();
		let id_sign = secp.sign(&Message::from_slice(&dig).unwrap(),&middle_key);
		let mut pkis = pk.to_owned();
		pkis.push_str(&id_sign.to_string());
		let mut hasher = Sha256::new();
		hasher.input(pkis.as_bytes());
		dig = hasher.result();
		let pub_sign = secp.sign(&Message::from_slice(&dig).unwrap(),&secret_key);

		Identity::new(ih,pk,&id_sign.to_string(),&pub_sign.to_string())
	}
}
