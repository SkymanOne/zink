use methods::{MULTIPLY_ELF, MULTIPLY_ID};
use risc0_zkvm::{
    default_executor_from_elf,
	SessionReceipt,
    serde::{from_slice, to_vec},
    ExecutorEnv,
};

use clap::Parser;
use codec::{Decode, Encode};
use sp_core::blake2_256;
use std::str::FromStr;
use subxt::{
	blocks::ExtrinsicEvents,
	config::WithExtrinsicParams,
	ext::{
		sp_core::{
			sr25519::{Pair as SubxtPair},
			Pair as SubxtPairT,
		},
		sp_runtime::AccountId32,
	},
	tx::{BaseExtrinsicParams, PairSigner, PlainTip},
	Error, OnlineClient, PolkadotConfig, SubstrateConfig,
};
// Runtime types, etc
#[subxt::subxt(runtime_metadata_path = "./metadata.scale")]
pub mod substrate_node {}

use substrate_node::runtime_types::sp_weights::weight_v2::Weight;


async fn send_receipt(
	contract: AccountId32,
	receipt: &SessionReceipt,
) -> Result<
	ExtrinsicEvents<
		WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
	>,
	Error,
> {
	let receipt_scale_encoded = to_vec(receipt).unwrap().encode();
	let receipt_recreated: SessionReceipt =
		from_slice(&Vec::<u32>::decode(&mut &receipt_scale_encoded[..]).unwrap()).unwrap();

	// If fails, the contract won't be able to verify the proof. If passed, all should be good for the contract to verify it
	assert_eq!(&receipt_recreated, receipt);

	let mut call_data = Vec::<u8>::new();
	//append the selector
	call_data.append(&mut (blake2_256("verify_via_ext".as_bytes())[0..4]).to_vec());
	//append the arguments
	call_data.append(&mut to_vec(receipt).unwrap().encode());

	let call_tx = substrate_node::tx().contracts().call(
		// MultiAddress::Id(contract)
		contract.into(),
		0, // value
		// Both need checking, or values from estimates. These ones come from contracts ui
		Weight { ref_time: 160_106_502_144, proof_size: 40_898_144 }, // gas_limit
		None,                                                  // storage_deposit_limit
		// To zkvm's serialization, then to SCALE encoding
		call_data,
	);

	let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();

	// This is just Alice, which is okay for an example
	let restored_key = SubxtPair::from_string(
		"0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		None,
	)
	.unwrap();
	let signer = PairSigner::new(restored_key);

	let result = api
		.tx()
		.sign_and_submit_then_watch_default(&call_tx, &signer)
		.await?
		.wait_for_in_block()
		.await?
		.fetch_events()
		.await?;

	Ok(result)
}

// Multiply them inside the ZKP
pub fn multiply_factors(a: u64, b: u64) -> (SessionReceipt, u64) {
    let env = ExecutorEnv::builder()
        // Send a & b to the guest
        .add_input(&to_vec(&a).unwrap())
        .add_input(&to_vec(&b).unwrap())
        .build()
        .unwrap();

    // First, we make an executor, loading the 'multiply' ELF binary.
    let mut exec = default_executor_from_elf(env, MULTIPLY_ELF).unwrap();

    // Run the executor to produce a session.
    let session = exec.run().unwrap();

    // Prove the session to produce a receipt.
    let receipt = session.prove().unwrap();

    // Extract journal of receipt (i.e. output c, where c = a * b)
    let c: u64 = from_slice(&receipt.journal).expect(
        "Journal output should deserialize into the same types (& order) that it was written",
    );

    // Report the product
    println!("I know the factors of {}, and I can prove it!", c);

    (receipt, c)
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Contract address
	#[arg(short, long)]
	contract_address: Option<String>,
}

#[tokio::main]
async fn main() {
	let args = Args::parse();
	// Pick two numbers
	let (receipt, _) = multiply_factors(17, 23);

	println!("IMAGE_ID {:?}", MULTIPLY_ID);

	// Verify receipt, panic if it's wrong
	receipt.verify(MULTIPLY_ID).expect(
		"Code you have proven should successfully verify; did you specify the correct image ID?",
	);

	let r_vec = to_vec(&receipt).unwrap();
	let receipt2: Result<SessionReceipt, _> = from_slice(&r_vec);
	let receipt2 = receipt2.unwrap();

	assert_eq!(receipt, receipt2);
	if let Some(contract_account) = &args.contract_address {
		let address = AccountId32::from_str(&contract_account).unwrap();
		let res = send_receipt(address, &receipt).await;
		println!("Submission result: {}", res.is_ok());
	}
}
