#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod genesis_dao {
    #[ink(storage)]
    pub struct GenesisDao {}

    impl GenesisDao {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }
    }

    impl genesis_dao_contract_trait::GenesisDao for GenesisDao {
        /// hook point for `empty_func` pallet
        #[ink(message)]
        fn empty_func(&self) -> u32 {
            0
        }

        /// hook point for `t` pallet
        #[ink(message)]
        fn t(&self, _whatever: AccountId) {
            // do nothing
        }

        /// hook point for `on_vote` pallet
        #[ink(message)]
        fn on_vote(&self, _voter: AccountId, foo: u128) -> u128 {
            foo
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use genesis_dao_contract_trait::GenesisDao as Trait;

    }
}
