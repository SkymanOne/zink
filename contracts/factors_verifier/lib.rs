#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::Environment,
    prelude::vec::Vec,
};

type DefaultAccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
type DefaultBalance = <ink::env::DefaultEnvironment as Environment>::Balance;

#[ink::contract]
mod factors_verifier {
	use scale::{Encode, Decode};
	use ink::prelude::vec::Vec;


	#[ink(storage)]
	pub struct FactorsVerifier {
		verified: bool,
		image_id: [u32; 8]
	}


	#[derive(PartialEq, Eq, Encode, Decode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub enum Error {
		PalletVerificationFailed,
		ContractVerificationFailed,
	}

	impl FactorsVerifier {
		#[ink(constructor)]
		pub fn new(image_id: [u32; 8]) -> Self {
			Self { verified: true, image_id }
		}

		#[ink(message)]
		pub fn verify_via_ext(&mut self, bytes: Vec<u32>) -> Result<(), Error> {
			Ok(())
		}

	}
}
