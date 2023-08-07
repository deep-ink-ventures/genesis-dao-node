#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract {

    #[ink(storage)]
    pub struct Contract {
        multiplier: u32,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(multiplier: u32) -> Self {
            Self { multiplier }
        }

        #[ink(message)]
        pub fn multiply(&mut self, value: u32) -> u32{
            value * self.multiplier
        }
    }
}
