use std::cmp::Ordering;

#[derive(Eq,PartialEq,Clone)]
struct Signatures {
	id: String,
	public_key: String,
}

#[derive(Eq,PartialEq,Clone)]
pub struct Identity {
	id: String,
	public_key: String,
	signatures: Signatures,
	//type,
	//provider,
}

impl Identity {
	pub fn new (id: String, public_key: String,
	id_signature: String, public_key_id_signature: String) -> Identity {
		Identity {
			id: id,
			public_key: public_key,
			signatures: Signatures {
				id: id_signature,
				public_key: public_key_id_signature,
			},
		}
	}

	pub fn id (&self) -> &String {
		&self.id
	}

	pub fn public_key (&self) -> &String {
		&self.public_key
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

//very much ad hoc
pub struct IdentityBuilder {
	id: u64,
}

impl IdentityBuilder {
	pub fn new () -> IdentityBuilder {
		IdentityBuilder {
			id: 0,
		}
	}

	pub fn build (&mut self) -> Identity {
		let gen = |a| format!("{}{}",a,self.id).to_owned();
		let i = Identity::new(gen("000"),gen("111"),gen("222"),gen("333"));
		self.id += 1;
		i
	}
}
