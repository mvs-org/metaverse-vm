# Hyperspace Mainnet
Hyperspace Mainnet Node - Stage 1 - PoS Consensus Mode - (Hybrid PoW/Pos and Stratum protocol implementation will be available  on Stage 2)

First of all you have to get the code:
```
git clone https://github.com/mvs-org/hyperspace
```

# Setting up enviroment
Install Substrate pre-requisites (including Rust):
For Unix-based operating systems, you should run the following commands:
```
curl https://sh.rustup.rs -sSf | sh
source ~/.cargo/env

rustup default nightly
rustup target add wasm32-unknown-unknown
```
You will also need to install the following packages:

Linux:
```
sudo apt install cmake pkg-config libssl-dev git clang libclang-dev
```
Linux on ARM: rust-lld is required for linking wasm, but is missing on non Tier 1 platforms. So, use this script to build lld and create the symlink /usr/bin/rust-lld to the build binary.

Mac:
```
brew install cmake pkg-config openssl git llvm
```

# Build the corresponding binary file:
```
cd Hyperspace
cargo build --release
```
The first build takes a long time, as it compiles all the necessary libraries.

# To start the full node you just compiled
```
./target/release/hyperspace --chain=hyperspace.json --name MyNode1
```
