# Factor prover

A prover that proves that it "knows" two factors of the number. It also incorporates automatic transaction submission to the local node.

## How to use

1. Run
``` bash
cargo run
```
If you run on MacOS
```
cargo run -F metal
```
To compile the project, prove locally. It will output the image id vector.

2. Copy paste the image id and use it to instantiate the ink! contract

3. Run
```
cargo run -- --contract-address <address>
```

To submit the transaction with the proof encoded

