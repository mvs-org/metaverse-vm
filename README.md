# betelgeuse - Alpha TestNet
Hybrid PoW/PoS consensus node

[![Discord](https://img.shields.io/discord/586902457053872148.svg)](https://discord.gg/qKMMzcx8)
## Build

### Prerequisites

First let's clone this git repository:
```bash
git clone https://github.com/mvs-org/betelgeuse
cd betelgeuse
```
Install Rust Developer Environment and all required tools:
```bash
curl https://sh.rustup.rs -sSf | sh
source ~/.cargo/env
```
Install necessary dependencies. On Ubuntu, run the following:
```bash
sudo apt update
sudo apt install -y cmake pkg-config libssl-dev git gcc build-essential clang libclang-dev
```
Install this specific Rust nightly version that is known to be compatible with the version of Substrate we are using:
```bash
rustup install nightly-2020-10-06
```
Now, configure the nightly version to work with the Wasm compilation target:
```bash
rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-06
```
Now you can start building your node:
```bash
cargo +nightly-2020-10-06 build --release
```
### Mining

#### Step 1 - create your wallet
Access Web Wallet using URL: http://95.111.250.21:2028/ and create a new wallet account and write down your mnemonic seed.
 
You’ll use the mnemonic it for accessing your account and for signed mining. 
For example supposing you've finished build your node earlier and your current directory is betelgeuse: 
```bash
./target/release/betelgeuse import-mining-key "razor limb price drum rifle robust stock cake coral chase pioneer shoot" --chain ./betelgeuse.json
```
You will get the following output: 2021-01-14 04:11:48  Registered one mining key (public key 0x80b9fcd9a5ff3bb238a9c084803eedc194f0592749bfb88d14f82baffaf67f7).

#### Step 2 - import mining key
```bash
./target/release/betelgeuse import-mining-key "razor limb price drum rifle robust stock cake coral chase pioneer shoot" --chain ./betelgeuse.json
```
You will get the following output: 2021-01-14 04:11:48  Registered one mining key (public key 0x80b9fcd9a5ff3bb238a9c084803eedc194f0592749bfb88d14f82baffaf67f7).

#### Step 3 - start mining
```bash
./target/release/betelgeuse --author "0x80b9fcd9a5ff3bb238a9c084803eedc194f0592749bfb88d14f82baffaf67f7b" --validator --name AdrianVPS --chain ./betelgeuse.json
```

The syntax of the command to run a node and start mining is:
./betelgeuse --author "<your public key>" --validator --name <node name> --chain ./betelgeuse.json 
  
*You only have to change your node name and your public key obtained in the previous step and you should start mining.

#### Step 3 - check mining results
You can check the node synchronization and other data using the Web wallet located at the following URL:  
http://95.111.250.21:2028/

