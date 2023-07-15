use methods::{MULTIPLY_ELF, MULTIPLY_ID};
use risc0_zkvm::{
    default_executor_from_elf,
	SessionReceipt,
    serde::{from_slice, to_vec},
    ExecutorEnv,
};

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

fn main() {
     // Pick two numbers
    let (receipt, _) = multiply_factors(17, 23);

}
