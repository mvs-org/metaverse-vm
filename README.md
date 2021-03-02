# Hyperspace Mainnet
Hyperspace Mainnet Node - Stage 1 - PoS Consensus Mode

First of all you have to get the code:
```
git clone https://github.com/mvs-org/Hyperspace
```

# Setting up enviroment
Install Substrate pre-requisites (including Rust):
For Unix-based operating systems, you should run the following commands:
```
curl https://sh.rustup.rs -sSf | sh

rustup default nightly
rustup target add wasm32-unknown-unknown
```
# Build the corresponding binary file:
```
cargo build --release
```
The first build takes a long time, as it compiles all the necessary libraries.

# To start the node you just compiled
```
./target/release/hyperspace --chain=hyperspace.json --name MyNode1
```
