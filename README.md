### Environment Setup
1. Install Rust from https://rustup.rs/
2. Install Solana v1.6.2 or later from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool

### Build and test for program compiled natively
```
$ cargo build
$ cargo test
```

### Build and test the program compiled for BPF
```
$ cargo build-bpf
$ cargo test-bpf
```

## CLI Client commands

For each command, there is also a document. You can see it by using --help additional parameter.

1. Build the CLI Client
```
$ cargo build --relase
```

2. Show CLI Client help
```
$ ./target/release/insurance-cli --help
```

## Show information about InsuranceContract

```
$ ./target/release/insurance-cli show <InsuranceContractData pubkey>
```
