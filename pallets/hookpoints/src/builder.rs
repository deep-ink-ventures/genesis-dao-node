use codec::{Encode};
use frame_support::sp_io::hashing::blake2_256;
use sp_std::prelude::*;

pub struct HookPoint<AccountId> {
    pub owner: AccountId,
    pub origin: AccountId,
    pub callback: Vec<u8>,
    pub data: Vec<u8>
}

impl<AccountId> HookPoint<AccountId> {
   pub fn new(callback: &str, owner: AccountId, origin: AccountId) -> Self {
      let hash = blake2_256(callback.as_bytes());
      HookPoint {
          owner, origin,
          callback: callback.into(),
          data: [hash[0], hash[1], hash[2], hash[3]].to_vec()
      }
   }

    pub fn add_arg<T>(mut self, arg: T) -> Self where T: Encode {
        self.data.append(&mut arg.encode());
        self
    }
}

