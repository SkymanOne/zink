#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{env::Environment, prelude::vec::Vec};
use scale::{Decode, Encode};

type DefaultAccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
type DefaultBalance = <ink::env::DefaultEnvironment as Environment>::Balance;

#[derive(PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
	PalletVerificationFailed,
	ContractVerificationFailed,
}

#[derive(PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Debug))]
pub struct ProofData {
	slice: Vec<u32>,
	image_id: [u32; 8],
}

impl From<scale::Error> for Error {
	fn from(_: scale::Error) -> Self {
		panic!("encountered unexpected invalid SCALE encoding")
	}
}

impl ink::env::chain_extension::FromStatusCode for Error {
	fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
		match status_code {
			0 => Ok(()),
			1 => Err(Error::PalletVerificationFailed),
			_ => panic!("encountered unknown status code"),
		}
	}
}

#[ink::chain_extension]
pub trait VerifierExtension {
	type ErrorCode = Error;

	#[ink(extension = 1)]
	fn verify(proof_data: ProofData) -> Result<(), Error>;
}

/// An environment using default ink environment types, with PSP-22 extension included
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
	const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

	type AccountId = DefaultAccountId;
	type Balance = DefaultBalance;
	type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
	type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;
	type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;

	type ChainExtension = crate::VerifierExtension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod factors_verifier {
	use crate::Error;
	use crate::Vec;

	#[ink(storage)]
	pub struct FactorsVerifier {
		verified: bool,
		image_id: [u32; 8],
	}

	impl FactorsVerifier {
		#[ink(constructor)]
		pub fn new(image_id: [u32; 8]) -> Self {
			Self { verified: false, image_id }
		}

		#[ink(message)]
		pub fn verify_via_ext(&mut self, bytes: Vec<u32>) -> Result<(), Error> {
			let proof_data = crate::ProofData { slice: bytes, image_id: self.image_id };
			let res = self.env().extension().verify(proof_data);
			if res.is_ok() {
				self.verified = true;
				Ok(())
			} else {
				Err(Error::PalletVerificationFailed)
			}
		}

		#[ink(message)]
		pub fn get_status(&self) -> bool {
			self.verified
		}
	}
}
