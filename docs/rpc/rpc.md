# Hyperspace Remote Procedure Calls

Remote Procedure Calls, or RPCs, are a way for an external program (eg.
a frontend) to communicate with a node. They are used for checking
storage values, submitting transactions, and querying the current
consensus authorities with any client that speaks json RPC. One widely
available option for using RPC is curl.

Example:
```
\#!/bin/bash
"curl -H "Content-Type: application/json" -d '{"id":"1",
"jsonrpc":"2.0", "method":"state\_getRuntimeVersion", "params":\[\]}'
https://vm.mvs.org/mainnet\_rpc"
```

List of calls: * calls marked with an asterix are under development *

**account\_nextIndex** (*account: AccountId)*\
Returns the next valid index (aka nonce) for given account.
This method takes into consideration all pending transactions currently
in the pool and if no transactions are found in the pool it fallbacks
to query the index from the runtime (aka. state nonce).

**author\_hasKey** (*public\_key: Bytes,\
key\_type: String*)\
Checks if the keystore has private keys for the given public key and
key type. Returns \`true\` if a private key could be found.

**author\_hasSessionKeys** (*session\_keys: Bytes*) \
Checks if the keystore has private keys for the given session public
keys. \`session\_keys\` is the SCALE encoded session keys object from
theruntime. Returns \`true\` iff all private keys could be found.

**author\_insertKey** (*key\_type: String,suri: String,\
 public: Bytes*)\
Insert a key into the keystore.

**author\_pendingExtrinsics** ()\
Returns all pending extrinsics, potentially grouped by sender.

**author\_removeExtrinsic** (*bytes\_or\_hash:Vec&lt;hash::ExtrinsicOrHash&lt;Hash&gt;&gt;*)\
Remove given extrinsic from the pool and temporarily ban it to prevent
reimporting.

**author\_rotateKeys** ()\
Generate new session keys and returns the corresponding public keys.

**author\_submitAndWatchExtrinsic** (*metadata: Self::Metadata,*
subscriber: Subscriber&lt;TransactionStatus&lt;Hash, BlockHash&gt;&gt;,
*bytes: Bytes*) \
Submit an extrinsic to watch. See
\[\`TransactionStatus\`\](sp\_transaction\_pool::TransactionStatus) for
details on transaction life cycle.

**author\_submitExtrinsic** (*extrinsic: Bytes*) \
Submit hex-encoded extrinsic for inclusion in block.

**author\_unwatchExtrinsic** (*metadata: Option&lt;Self::Metadata&gt;,*
*id: SubscriptionId*) \
Unsubscribe from extrinsic watching.

**babe\_epochAuthorship** () \
Returns data about which slots (primary or secondary) can be claimed in
the current epoch with the keys in the keystore.

**balances\_usableBalance** (*instance: u8,*
*who: AccountId*) \
Node-specific RPC methods for interaction with balances.

**chain_getBlock** (*hash: Option&lt;Hash&gt;*) \
Get header and body of a relay chain block.

**chain\_getBlockHash** (*hash:Option&lt;ListOrValue&lt;NumberOrHex&gt;&gt;*) \
Get hash of the n-th block in the canon chain. By default returns
latest block hash.

**chain\_getFinalisedHead** () \
Get hash of the last finalized block in the canon chain.

**chain\_getHead** (*hash:Option&lt;ListOrValue&lt;NumberOrHex&gt;&gt;*) \
Get hash of the n-th block in the canon chain. By default returns
latest block hash.

**chain\_getHeader** (*hash: Option&lt;Hash&gt;*) \
Get header of a relay chain block.

**chain\_getRuntimeVersion** (*hash: Option&lt;Hash&gt;*) \
Get the runtime version.

**chain\_subscribeAllHeads** (*metadata: Self::Metadata,* *subscriber: Subscriber&lt;Header&gt;*) \
Finalized head subscription operations.

**All head subscription** (*metadata: Self::Metadata,* \
*subscriber: Subscriber&lt;Header&gt;*) \
Finalized head subscription operations.

**chain\_subscribeNewHead** (*metadata: Self::Metadata,* \
*subscriber: Subscriber&lt;Header&gt;*) \
Finalized head subscription operations.

**chain\_subscribeNewHeads** (*metadata: Self::Metadata,* \
*subscriber: Subscriber&lt;Header&gt;*) \
Finalized head subscription operations.

**chain\_subscribeRuntimeVersion** (*metadata: Self::Metadata,* \
*subscriber: Subscriber&lt;RuntimeVersion&gt;*) \
Unsubscribe runtime version.

**chain\_unsubscribeAllHeads** (*metadata: Option&lt;Self::Metadata&gt;,* \
*id: SubscriptionId*) \
Unsubscribe all heads.

**chain\_unsubscribeFinalisedHeads()** alias **chain\_unsubscribeAllHeads**

**chain\_unsubscribeNewHead** alias **chain\_unsubscribeAllHeads**

**chain\_unsubscribeNewHeads** (metadata: Option&lt;Self::Metadata&gt;, \
id: SubscriptionId*)
Unsubscribe all heads.

**chain\_unsubscribeRuntimeVersion**(*metadata:Option&lt;Self::Metadata&gt;,*
*id: SubscriptionId*) \
Finalized head and RuntimeVersion unsubscription operations.

**childstate\_getKeys** (*child\_storage\_key: PrefixedStorageKey,* \
prefix: StorageKey, \
*hash: Option&lt;Hash&gt;*) \
Returns the keys with prefix from a child storage, leave empty to get
all the keys

**childstate\_getStorage** (*child\_storage\_key: PrefixedStorageKey,* \
key: StorageKey, \
*hash: Option&lt;Hash&gt;*) \
Returns a child storage entry at a specific block's state

**childstate\_getStorageSize** (*child\_storage\_key:
PrefixedStorageKey,* \
key: StorageKey, \
*hash: Option&lt;Hash&gt;*) \
Returns the size of a child storage entry at a block's state.

**eth\_accounts** ()\* \
Returns EVM accounts list.

**eth\_blockNumber** () \
Returns highest block number from EVM perspective.

**eth\_call** (*\_: CallRequest,* \
*\_: Option&lt;BlockNumber&gt;*) \
Call contract, returning the output data.

**eth\_chainId** () \
Returns the chain ID used for transaction signing at the current best
block. None is returned if not.

**eth\_estimateGas** (*\_: CallRequest,* \
*\_: Option&lt;BlockNumber&gt;*) \
Estimate gas needed for execution of given contract.

**eth\_gasPrice** () \
Returns current gas\_price.

**eth\_getBalance** (*\_: H160,* \
*\_: Option&lt;BlockNumber&gt;*) \
Returns balance of the given account.

**eth\_getBlockByHash** (*\_: H256,* \
*\_: bool*) \
Returns block with given hash.

**eth\_getBlockByNumber** (*\_: BlockNumber,* \
*\_: bool*) \
Returns block with given number.

**eth\_getBlockTransactionCountByHash** (*\_: H256*) \
Returns the number of transactions in a block with given hash. 

**eth\_getBlockTransactionCountByNumber** (*\_: BlockNumber*) \
Returns the number of transactions in a block with given block number. 

**eth\_getCode** (*\_: H160,* \
*\_: Option&lt;BlockNumber&gt;*) \
Returns the code at given address at given time (block number).

**eth\_getLogs** (*\_: Filter*) \
Returns logs matching given filter object.

**eth\_getStorageAt** (*\_: H160,* \
\_: U256, \
*\_: Option &lt;BlockNumber&gt;*) \
Returns content of the storage at given address.

**eth\_getTransactionByBlockHashAndIndex** ( *\_: H256,* \
*\_: Index*) \
Returns transaction at given block hash and index. 

**eth\_getTransactionByBlockNumberAndIndex**( *v\_: BlockNumber,* \
*\_: Index*) \
Returns transaction by given block number and index. 

**eth\_getTransactionByHash** (*\_: H256*) \
Get transaction by its hash.

**eth\_getTransactionCount** (*\_: H160,* \
*\_: Option&lt;BlockNumber&gt;*) \
Returns the number of transactions sent from given address at given
time (block number).

**eth\_getTransactionReceipt** (*\_: H256*) \
Returns transaction receipt by transaction hash.

**eth\_getUncleByBlockHashAndIndex** (*\_: H256,* \
*\_: Index*) \
Returns Unlce by block hash and index

**eth\_getUncleByBlockNumberAndIndex** (*\_: BlockNumber,* \
*\_: Index*) \
Returns an uncle at given block and index.

**eth\_getUncleCountByBlockHash** (*\_: H256*) \
Returns the number of uncles in a block with given hash.

**eth\_getUncleCountByBlockNumber** (*\_: BlockNumber*) \
Returns the number of uncles in a block with given block number.

**eth\_getWork** ()\* \
Returns the hash of the current block, the seedHash, and the boundary
condition to be met.

**eth\_hashrate** () \
Returns the number of hashes per second that the node is mining with.

**eth\_mining** () \
Returns true if client is actively mining new blocks.

**eth\_protocolVersion** () \
Returns protocol version encoded as a string (quotes are necessary
here).

**eth\_sendRawTransaction** (*\_: Bytes*) \
Sends signed transaction, returning its hash.

**eth\_sendTransaction** (*\_: TransactionRequest*) \
Sends transaction; will block waiting for signer to return the
transaction hash.

**eth\_submitHashrate** (*\_: U256,* \
*\_: H256*) \
Used for submitting mining hashrate.

**eth\_submitWork** ( *\_: H64,* \
\_: H256, \
*\_: H256*) \
Used for submitting a proof-of-work solution.

**eth\_subscribe** (*\_: Self::Metadata,* \
\_: typed::Subscriber&lt;pubsub::Result&gt;, \
\_: pubsub::Kind, \
*\_: Option&lt;pubsub::Params&gt;*) \
Subscribe to Eth subscription.

**eth\_syncing** () \
Returns an object with data about the sync status or false.

**eth\_unsubscribe** (*\_: Option&lt;Self::Metadata&gt;,* \
*\_: SubscriptionId*) \
Unsubscribe from existing Eth subscription.

**grandpa\_proveFinality** ( *begin: Hash,* \
end: Hash, \
*authorities\_set\_id: u64*) \
Prove finality for the given block number by returning the
justification for the last block in the set and all the intermediary
headers to link them together.

**grandpa\_roundState** () \
Returns the state of the current best round state as well as the
ongoing background rounds.

**grandpa\_subscribeJustifications** (*metadata: Self::Metadata,* \
*subscriber: Subscriber&lt;Notification&gt;*) \
Returns the block most recently finalized by Grandpa, alongside side
its justification.

**grandpa\_unsubscribeJustifications** (*metadata: \
Option&lt;Self::Metadata&gt;,* \
*id: SubscriptionId*) \
Unsubscribe from receiving notifications about recently finalized
blocks.

**headerMMR\_genProof** (*block\_number\_of\_member\_leaf: u64,* \
*block\_number\_of\_last\_leaf: u64*) \
Get the MMR proof for a certain height, block number of member leaf,
block number of the lastest leafnet\_listening,
Returns true if client is actively listening for network connections.
Otherwise false.

**net\_peerCount** () \=
Returns number of peers connected to node.

**net\_version** () \
Returns protocol version.

**offchain\_localStorageGet** (*kind: StorageKind,* \
*key: Bytes*) \
Get offchain local storage under given key and prefix.

**offchain\_localStorageSet** (*kind: StorageKind,* \
key: Bytes, \
*value: Bytes*) \
Set offchain local storage under given key and prefix.

**payment\_queryFeeDetails** (*encoded\_xt: Bytes,* \
*at: Option&lt;BlockHash&gt;*) \
Query the fee of a payment

**payment\_queryInfo** (*encoded\_xt: Bytes,* \
*at: Option&lt;BlockHash&gt;*) \
Get details regarding payment fee.

**staking\_powerOf** (*who: AccountId*) \
Retrunt the power on a certain AccountId in a staking context.

**state\_call** (*name: String,* \
bytes: Bytes, \
*hash: Option&lt;Hash&gt;*) \
Call a contract's block state

**state\_callAt** (*name: String,* \
bytes: Bytes, \
*hash: Option&lt;Hash&gt;*) \
Call a contract at a block's state.

**state\_getKeys** (*prefix: StorageKey,* \
*hash: Option&lt;Hash&gt;*) \
Returns the keys with prefix, leave empty to get all the keys.

**state\_getKeysPaged** (*prefix: Option&lt;StorageKey&gt;,* \
count: u32, \
start\_key: Option&lt;StorageKey&gt;, \
*hash: Option&lt;Hash&gt;*) \
Returns the keys with prefix with pagination support.
Up to \`count\` keys will be returned.
If \`start\_key\` is passed, return next keys in storage in
lexicographic order.

**state\_getKeysPagedAt** (*prefix: Option&lt;StorageKey&gt;,* \
count: u32, \
start\_key: Option&lt;StorageKey&gt;, \
*hash: Option&lt;Hash&gt;*) \
Returns the keys with prefix with pagination support.
Up to \`count\` keys will be returned.
If \`start\_key\` is passed, return next keys in storage in
lexicographic order.

**state\_getMetadata** () \
Returns the runtime metadata as an opaque blob.state\_getPairs,
Returns the keys with prefix, leave empty to get all the keys

**state\_getReadProof** (*keys: Vec&lt;StorageKey&gt;,* \
*hash: Option&lt;Hash&gt;*) \
Returns proof of storage entries at a specific block's state.

**state\_getRuntimeVersion** (*hash: Option&lt;Hash&gt;*) \
Get the runtime version.

**state\_getStorage** (*key: StorageKey,* \
*hash: Option&lt;Hash&gt;*) \
Returns a storage entry at a specific block's state.

**state\_getStorageAt** (*key: StorageKey,* \
*hash: Option&lt;Hash*&gt;) \
Returns the hash of a storage entry at a block's state.

**state\_getStorageHash** (*key: StorageKey,* \
*hash: Option&lt;Hash&gt;*) \
Returns the hash of a storage entry at a block's state.

**state\_getStorageHashAt** (*key: StorageKey,* \
*hash: Option&lt;Hash*&gt;) \
Returns the hash of a storage entry at a block's state.

**state\_getStorageSize** (*key: StorageKey,* \
 *hash: Option&lt;Hash*) \
Returns the size of a storage entry at a block's state.

**state\_queryStorage** (*keys: Vec&lt;StorageKey&gt;,* \
block: Hash, \
*hash: Option&lt;Hash&gt;*) \
Query historical storage entries (by key) starting from a block given as
the second parameter.
NOTE: This first returned result contains the initial state of storage
for all keys.
Subsequent values in the vector represent changes to the previous state
(diffs).

**state\_queryStorageAt** (*keys: Vec&lt;StorageKey&gt;,* \
*at: Option&lt;Hash&gt;*) \
Query storage entries (by key) starting at block hash given as the
second parameter.

**state\_subscribeRuntimeVersion** (*metadata: Self::Metadata,* \
*subscriber: Subscriber&lt;RuntimeVersion*) \
New runtime version subscription.

**state\_subscribeStorage** (*metadata: Self::Metadata,* \
subscriber: Subscriber&lt;StorageChangeSet&lt;Hash&gt;&gt;, \
*keys: Option&lt;Vec&lt;StorageKey&gt;*) \
New storage subscription.

**state\_unsubscribeRuntimeVersion** (*metadata: Option&lt;Self::Metadata&gt;,* \
*id: SubscriptionId*) \
Unsubscribe from runtime subscription

**state\_unsubscribeStorage** (*metadata: Option&lt;Self::Metadata&gt;,* \
*id: SubscriptionId*) \
Unsubscribe from storage subscription

**subscribe\_newHead** (*metadata: Option&lt;Self::Metadata&gt;,* \
*id: SubscriptionId*) \
New head subscription

**sync\_state\_genSyncSpec** (*raw: bool*) \
Returns the json-serialized chainspec running the node, with a sync
state.

**system\_accountNextIndex** (*account: AccountId*) \
Returns the next valid index (aka nonce) for given account. This method
takes into consideration all pending transactions currently in the pool
and if no transactions are found in the pool it fallbacks to query the
index from the runtime (aka. state nonce).

**system\_addLogFilter** (*directives: String*) \
Adds the supplied directives to the current log filter. The syntax is
identical to the CLI \`&lt;target&gt;=&lt;level&gt;\`:\`sync=debug,state=trace\`

**system\_addReservedPeer** (peer: String) \
Adds a reserved peer. Returns the empty string or an error. The string
parameter should encode a \`p2p\` multiaddr.

**system\_chain** () \
Get the chain's name. Given as a string identifier.

**system\_chainType** () \
Get the chain's type.

**system\_dryRun** (*extrinsic: Bytes*, \
at: Option&lt;BlockHash&gt;*) \
Dry run an extrinsic at a given block. Return SCALE encoded
ApplyExtrinsicResult.

**system\_dryRunAt** (*extrinsic: Bytes,* \
*at: Option&lt;BlockHash&gt;*) \
Dry run an extrinsic at a given block. Return SCALE encoded
ApplyExtrinsicResult

**system\_health** () \
Return health status of the node. Node is considered healthy if it is:
- connected to some peers (unless running in dev mode)
- not performing a major sync

**system\_localListenAddresses** () \
Returns the multiaddresses that the local node is listening on. The
addresses include a trailing \`/p2p/\` with the local PeerId, and are
thus suitable to be passed to \`system\_addReservedPeer\` or as a
bootnode address for example.

**system\_localPeerId** () \
Returns the base58-encoded PeerId of the node.

**system\_name** () \
Get the node's implementation name. Plain old string.

**system\_networkState** () \
Return the current network state

**system\_nodeRoles** () \
Returns the roles the node is running as.

**system\_peers** () \
Returns currently connected peers

**system\_properties** () \
Get a custom set of properties as a JSON object, defined in the chain
spec.

**system\_removeReservedPeer** (*peer\_id: String*) \
Remove a reserved peer. Returns the empty string or an error. The
string should encode only the PeerId.

**system\_resetLogFilter** () \
Resets the log filter to defaults.

**system\_syncState**() \
Returns the state of the syncing of the node: starting block, current
best block, highest known block.

**system\_version**() \
Get the node implementation's version. Should be a semver string.

**unsubscribe\_newHead** (*metadata: Option&lt;Self::Metadata&gt;,* \
*id: SubscriptionId*) 

**web3\_clientVersion** () \
Returns current client version.

**web3\_sha3** (*\_: Bytes*) \
Returns sha3 of the given data.


