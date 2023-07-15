use crate::Runtime;
use codec::{Decode, Encode};
use frame_support::log::{error, trace};
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use risc0_zkvm::{serde::from_slice, SessionReceipt};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::DispatchError;
use sp_std::prelude::Vec;

/// Contract extension for `FetchRandom`
#[derive(Default)]
pub struct Risc0VerifierExtension;

type DispatchResult = Result<(), DispatchError>;

fn convert_err(err_msg: &'static str) -> impl FnOnce(DispatchError) -> DispatchError {
	move |err| {
		trace!(
			target: "runtime",
			"Risc0 error:{:?}",
			err
		);
		DispatchError::Other(err_msg)
	}
}

enum FuncId {
	Verify,
	Deserialize
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct ProofData {
	slice: Vec<u32>,
	image_id: [u32; 8],
}

impl TryFrom<u16> for FuncId {
	type Error = DispatchError;

	fn try_from(func_id: u16) -> Result<Self, Self::Error> {
		let id = match func_id {
			1 => FuncId::Verify,
			2 => FuncId::Deserialize,
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			},
		};
		Ok(id)
	}
}

/// Chain Extension function for the verification
/// Note! Not weight is charged, but it should be calculated
fn verify<E: Ext>(env: Environment<E, InitState>) -> DispatchResult
where
	<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
{
	let mut buffer = env.buf_in_buf_out();
	let proof_data: ProofData = buffer.read_as_unbounded(buffer.in_len())?;
	let receipt: SessionReceipt = from_slice(&proof_data.slice)
		.map_err(|_| DispatchError::Other("Error during proof deserialization"))?;
	receipt.verify(proof_data.image_id).map_err(|_| DispatchError::Other("Proof is invalid"))
}

/// An experiment to just deserialize the input from the slice and encode it using SCALE
fn deserialize_proof<E: Ext>(env: Environment<E, InitState>) -> DispatchResult
where
	<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
{
	let mut buffer = env.buf_in_buf_out();
	let proof_data: ProofData = buffer.read_as_unbounded(buffer.in_len())?;
	let receipt: SessionReceipt = from_slice(&proof_data.slice)
		.map_err(|_| DispatchError::Other("Error during proof deserialization"))?;
	let receipt_wrapped = ReceiptWrapper(receipt);
	let bytes = receipt_wrapped.encode();
	buffer.write(&bytes, false, None).map_err(
		convert_err("Error writing receipt to buffer")
	)
}

#[derive(Debug)]
struct ReceiptWrapper(SessionReceipt);

impl Encode for ReceiptWrapper {}

impl ChainExtension<Runtime> for Risc0VerifierExtension {
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		let func_id = FuncId::try_from(env.func_id())?;
		match func_id {
			FuncId::Verify => verify::<E>(env)?,
			FuncId::Deserialize => deserialize_proof::<E>(env)?,
		}

		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
