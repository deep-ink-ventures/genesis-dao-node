use codec::{Encode};
use sp_core::{Blake2Hasher, Hasher};

pub struct HookPoint<AccountId> {
    pub owner: AccountId,
    pub origin: AccountId,
    pub name: String,
    pub callback: Callback
}

impl<AccountId> HookPoint<AccountId> {
   pub fn new(owner: AccountId, origin: AccountId, mod_name: &str, callback: Callback) -> Self {
      HookPoint {
          owner, origin,
          callback: callback.prefix_selector(mod_name),
          name: mod_name.into()
      }
   }
}

pub struct Callback {
    pub name: String,
    pub data: Vec<u8>
}

impl Callback {
    pub fn new(func_name: &str) -> Self {
        Callback { name: func_name.into(), data: vec![] }
    }

    pub fn add_arg<T>(mut self, arg: T) -> Self where T: Encode {
        self.data.append(&mut arg.encode());
        self
    }

    pub fn prefix_selector(mut self, mod_name: &str) -> Self {
        let mut mod_str = mod_name.to_string();
        mod_str.push_str("::");
        mod_str.push_str(self.name.as_str());
        let hash = Blake2Hasher::hash(mod_str.as_str().as_bytes());
	    let bytes = hash.as_bytes();
	    let mut data = [bytes[0], bytes[1], bytes[2], bytes[3]].to_vec();
        data.append(&mut self.data);
        self.data = data;
        return self
    }

}
