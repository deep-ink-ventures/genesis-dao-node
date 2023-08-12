#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract {

    #[ink(storage)]
    pub struct Contract {
        value: u128,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(init_value: u128) -> Self {
            Self { value: init_value }
        }

        #[ink(message)]
        pub fn multiply(&mut self, by: u128) -> u128 {
            self.value * by
        }

        #[ink(message)]
        pub fn get(&self) -> u128 {
            self.value
        }
    }
}
