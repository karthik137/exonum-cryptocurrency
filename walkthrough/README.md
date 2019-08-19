## Exonum cryptoCurrency Walkthrough


### 1) Create Rust project


$ cargo new crytpocurrency --lib


> Create Cargo.toml in the project directory and add following contents

```rust
	[package]
	name = "cryptocurrency"
	version = "0.1.0"
	edition = "2018"
	authors = ["Your Name <your@email.com>"]

	[dependencies]
	exonum = "0.10.0"
	exonum-derive = "0.10.0"
	failure = "0.1.5"
	serde = "1.0.0"
	serde_derive = "1.0.0"
	serde_json = "1.0.0"
	protobuf = "2.2.0"

	[build-dependencies]
	exonum-build = "0.10.0"

```

Define constants in lib.rs
> Open src/lib.rs add the following code

```rust
	// Service identifier
	const SERVICE_ID: u16 = 1;
	// Starting balance of a newly created wallet
	const INIT_BALANCE: u64 = 100;

```

### 2) Imports


> NOTE: Add the following code in src/lib.rs

```rust
#[macro_use]
extern crate exonum_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

use exonum::api::{self, ServiceApiBuilder, ServiceApiState};
use exonum::blockchain::{
    ExecutionError, ExecutionResult, Service, Transaction,
    TransactionContext, TransactionSet,
};
use exonum::crypto::{Hash, PublicKey};
use exonum::messages::RawTransaction;
use exonum::storage::{Fork, MapIndex, Snapshot};

// Service identifier
const SERVICE_ID: u16 = 1;
// Starting balance of a newly created wallet
const INIT_BALANCE: u64 = 100;
mod proto;

```


### 3) Declare Persistent Data

Exonum uses Protobuf as its serialization format for storage of data. Thus, we need to describe our structures using the Protobuf interface description language first. 

We should declare what kind of data the service will store in the blockchain. In our case we need to declare a single type – wallet. Inside the wallet we want to store:

Public key which is the address of the wallet
Name of the owner (purely for convenience reasons)
Current balance of the wallet.

As a first step we add a module named proto to our project. We add a cryptocurrency.proto file to this module and describe the Wallet structure in it in the Protobuf format. 

	a)Create proto directory.
	b)Create cryptocurrency.proto file in proto directory.


> NOTE: Add the following code in cryptocurrency.proto


```rust
	syntax = "proto3";

	// Allows to use `exonum.PublicKey` structure already described in `exonum`
	// library.
	import "helpers.proto";

	// Wallet structure used to persist data within the service.
	message Wallet {
	  exonum.PublicKey pub_key = 1;
	  string name = 2;
	  uint64 balance = 3;
	}

```

Secondly, to integrate the Protobuf-generated files into the proto module of the project, we add a mod.rs file with the following content to the proto module:


> Create mod.rs file in proto directory and add the following code:

> NOTE: Add the following code in cryptocurrency.proto


```rust
#![allow(bare_trait_objects)]
#![allow(renamed_and_removed_lints)]

include!(concat!(env!("OUT_DIR"), "/protobuf_mod.rs"));

use exonum::proto::schema::*;

```


As a third step, in the build.rs file we introduce the main function that generates Rust files from their Protobuf descriptions.

> NOTE: Add the following code in build.rs

```rust
extern crate exonum_build;

use exonum_build::{get_exonum_protobuf_files_path, protobuf_generate};

fn main() {
    let exonum_protos = get_exonum_protobuf_files_path();
    protobuf_generate(
        "src/proto",
        &["src/proto", &exonum_protos],
        "protobuf_mod.rs",
    );
}

```

Finally, we create the same structure definition of the wallet in Rust language based on the proto schema presented above. The service will use the structure for further operations with data schema and to validate the corresponding .rs Protobuf-generated file with this structure:

> NOTE: Add the following code to lib.rs

```rust
#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::cryptocurrency::Wallet")]
pub struct Wallet {
    pub pub_key: PublicKey,
    pub name: String,
    pub balance: u64,
}


```


Derive ProtobufConvert from exonum_derive helps to validate the Protobuf structure presented earlier. In this way we make sure that exonum::crypto::PublicKey corresponds to the public key in the proto format. Therefore, we can safely use it in our Wallet structure.

We need to change the wallet balance, so we add methods to the Wallet type:


> NOTE: Add the following code to lib.rs

```rust
impl Wallet {
    pub fn new(&pub_key: &PublicKey, name: &str, balance: u64) -> Self {
        Self {
            pub_key,
            name: name.to_owned(),
            balance,
        }
    }

    pub fn increase(self, amount: u64) -> Self {
        let balance = self.balance + amount;
        Self::new(&self.pub_key, &self.name, balance)
    }

    pub fn decrease(self, amount: u64) -> Self {
        debug_assert!(self.balance >= amount);
        let balance = self.balance - amount;
        Self::new(&self.pub_key, &self.name, balance)
  }
}

```



### 4) Create Schema


Schema is a structured view of the key-value storage used in Exonum. To access the storage, however, we will not use the storage directly, but rather Snapshots and Forks. Snapshot represents an immutable view of the storage, and Fork is a mutable one, where the changes can be easily rolled back. Snapshot is used in read requests, and Fork - in transaction processing.


> NOTE: Add the following code to lib.rs


```rust
pub struct CurrencySchema<T> {
    view: T,
}

```


For access to the objects inside the storage we need to declare the layout of the data. As we want to keep the wallets in the storage, we will use an instance of MapIndex, a map abstraction. Keys of the index will correspond to public keys of the wallets. Index values will be serialized Wallet structs.

Snapshot provides random access to every piece of data inside the database. To isolate the wallets map into a separate entity, we add a unique prefix to it, which is the first argument to the MapIndex::new call


> NOTE: Add the following code to lib.rs


```rust
impl<T: AsRef<Snapshot>> CurrencySchema<T> {
    pub fn new(view: T) -> Self {
        CurrencySchema { view }
    }

    // Utility method to get a list of all the wallets from the storage
    pub fn wallets(&self) -> MapIndex<&Snapshot, PublicKey, Wallet> {
        MapIndex::new("cryptocurrency.wallets", self.view.as_ref())
    }

    // Utility method to quickly get a separate wallet from the storage
    pub fn wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }
}

```


Here, we have declared a constructor and two getter methods for the schema. We wrap any type that allows interacting with the schema as a Snapshot reference (that is, implements the AsRef trait from the standard library). Fork implements this trait, which means that we can construct a CurrencySchema instance above the Fork, and use wallets and wallet getters for it.

For Fork-based schema, we declare an additional method to write to the storage:


> NOTE: Add the following code to lib.rs


```rust
impl<'a> CurrencySchema<&'a mut Fork> {
    pub fn wallets_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        MapIndex::new("cryptocurrency.wallets", &mut self.view)
    }
}

```


### 5) Define Transactions


Transaction is a kind of message which performs atomic actions on the blockchain state.

For our Cryptocurrency Tutorial we need two transaction types:

    a) Create a new wallet and add some money to it
    b) Transfer money between two different wallets.

The transaction to create a new wallet (TxCreateWallet) contains a name of the user who created this wallet. Address of the wallet will be derived from the public key that was used to sign this transaction.


> NOTE: Add the following code to cryptocurrency.proto


```rust
// Transaction type for creating a new wallet.
message TxCreateWallet {
  // UTF-8 string with the owner's name.
  string name = 1;
}


```


The transaction to transfer coins between different wallets (TxTransfer) has a public key of the receiver (to). It also contains the amount of money to move between the wallets. We add the seed field to make sure that our transaction is impossible to replay. Sender's public key will be the same key that was used to sign the transaction.


> NOTE: Add the following code to cryptocurrency.proto


```rust
// Transaction type for transferring tokens between two wallets.
message TxTransfer {
  // Public key of the receiver.
  exonum.PublicKey to = 1;
  // Number of tokens to transfer from the sender's account to the receiver's
  // account.
  uint64 amount = 2;
  // Auxiliary number to guarantee non-idempotence of transactions.
  uint64 seed = 3;
}

```


Now, just as we did with the Wallet structure above, we need to describe the same transactions in Rust:

> NOTE: Add the following code to lib.rs


```rust
#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::cryptocurrency::TxCreateWallet")]
pub struct TxCreateWallet {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::cryptocurrency::TxTransfer")]
pub struct TxTransfer {
    pub to: PublicKey,
    pub amount: u64,
    pub seed: u64,
}


```


Service transactions are defined through the enum with the derive of the TransactionSet that automatically assigns transaction IDs based on their declaration order starting from 0:

> NOTE: Add the following code to lib.rs


```rust
#[derive(Serialize, Deserialize, Clone, Debug, TransactionSet)]
pub enum CurrencyTransactions {
    /// Create wallet transaction.
    CreateWallet(TxCreateWallet),
    /// Transfer tokens transaction.
    Transfer(TxTransfer),
}

```


### 6) Reporting Errors


The execution of the transaction may be unsuccessful for some reason. For example, the transaction TxCreateWallet will not be executed if the wallet with such public key already exists. There are also three reasons why the transaction TxTransfer cannot be executed:

    a) There is no sender with the given public key
    b) There is no recipient with the given public key
    c) The sender has insufficient currency amount.

Let’s define the codes of the above errors:


> NOTE: Add the following code to lib.rs

```rust
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    #[fail(display = "Wallet already exists")]
    WalletAlreadyExists = 0,

    #[fail(display = "Sender does not exist")]
    SenderNotFound = 1,

    #[fail(display = "Receiver does not exist")]
    ReceiverNotFound = 2,

    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 3,

    #[fail(display = "Sender same as receiver")]
    SenderSameAsReceiver = 4,
}

// Conversion between service-specific errors and the standard error type
// that can be emitted by transactions.
impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

```



### 7) Transaction Execution



Every transaction in Exonum has business logic of the blockchain attached, which is encapsulated in the Transaction trait. This trait has execute method which contains logic applied to the storage when a transaction is executed.

In our case execute method gets the reference to the TransactionContext. It includes Fork of the storage (can be accessed with .fork()) and the public key which was used to sign the transaction (can be accessed with .author()). We wrap Fork with our CurrencySchema to access our data layout.

For creating a wallet, we check that the wallet does not exist and add a new wallet if so:


> NOTE: Add the following code to lib.rs

```rust
impl Transaction for TxCreateWallet {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let author = context.author();
        let view = context.fork();
        let mut schema = CurrencySchema::new(view);
        if schema.wallet(&author).is_none() {
            let wallet = Wallet::new(&author, &self.name, INIT_BALANCE);
            println!("Create the wallet: {:?}", wallet);
            schema.wallets_mut().put(&author, wallet);
            Ok(())
        } else {
            Err(Error::WalletAlreadyExists)?
        }
    }
}

```


TxTransfer transaction gets two wallets for both sides of the transfer transaction. If they are found, we check the balance of the sender. If the sender has enough coins, then we decrease the sender’s balance and increase the receiver’s balance.

We also need to check that the sender does not send the coins to himself. Otherwise, if the sender is equal to the receiver, the implementation below will create money out of thin air.


> NOTE: Add the following code to lib.rs


```rust

impl Transaction for TxTransfer {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let author = context.author();
        let view = context.fork();

        if author == self.to {
            Err(Error::SenderSameAsReceiver)?
        }

        let mut schema = CurrencySchema::new(view);

        let sender = match schema.wallet(&author) {
            Some(val) => val,
            None => Err(Error::SenderNotFound)?,
        };

        let receiver = match schema.wallet(&self.to) {
            Some(val) => val,
            None => Err(Error::ReceiverNotFound)?,
        };

        let amount = self.amount;
        if sender.balance >= amount {
            let sender = sender.decrease(amount);
            let receiver = receiver.increase(amount);
            println!("Transfer between wallets: {:?} => {:?}", sender, receiver);
            let mut wallets = schema.wallets_mut();
            wallets.put(&author, sender);
            wallets.put(&self.to, receiver);
            Ok(())
        } else {
            Err(Error::InsufficientCurrencyAmount)?
        }
    }
}



```




### 8) Implement API


Next, we need to implement the node API. With this aim we declare a blank struct

> NOTE: Add the following code to lib.rs

``` rust
pub struct CryptocurrencyApi;
```


API for Transactions

The core processing logic is essentially the same for all types of transactions and is implemented by exonum. Therefore, there is no need to implement a separate API for transactions management within the service. To send a transaction you have to create a transaction message according to the uniform structure developed by exonum.

The transaction ID is a transaction number in the enum with #[derive(TransactionSet)]. As we mentioned earlier, transactions count starts with 0.
API for Read Requests

We want to implement 2 read requests:

    a) Return the information about all wallets in the system
    b) Return the information about a specific wallet identified by the public key.

To accomplish this, we define a couple of corresponding methods in CryptocurrencyApi that use state to read information from the blockchain storage.

For parsing a public key of a specific wallet we define a helper structure.


> NOTE: Add the following code to lib.rs


```rust
/// The structure describes the query parameters for the `get_wallet` endpoint.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct WalletQuery {
    /// Public key of the requested wallet.
    pub pub_key: PublicKey,
}

impl CryptocurrencyApi {
    /// Endpoint for getting a single wallet.
    pub fn get_wallet(
        state: &ServiceApiState,
        query: WalletQuery
    ) -> api::Result<Wallet> {
        let snapshot = state.snapshot();
        let schema = CurrencySchema::new(snapshot);
        schema
            .wallet(&query.pub_key)
            .ok_or_else(|| api::Error::NotFound("\"Wallet not found\"".to_owned()))
    }

    /// Endpoint for dumping all wallets from the storage.
    pub fn get_wallets(
        state: &ServiceApiState,
        _query: ()
    ) -> api::Result<Vec<Wallet>> {
        let snapshot = state.snapshot();
        let schema = CurrencySchema::new(snapshot);
        let idx = schema.wallets();
        let wallets = idx.values().collect();
        Ok(wallets)
    }
}

```



### 9) Wire API

As the final step of the API implementation, we need to tie the request processing logic to the specific endpoints. We do this in the CryptocurrencyApi::wire() method:


> NOTE: Add the following code to lib.rs

```rust
impl CryptocurrencyApi {
    pub fn wire(builder: &mut ServiceApiBuilder) {
        // Binds handlers to the specific routes.
        builder
            .public_scope()
            .endpoint("v1/wallet", Self::get_wallet)
            .endpoint("v1/wallets", Self::get_wallets);
    }
}


```


### 10) Define Service

> NOTE: Add the following code to lib.rs

```rust
#[derive(Debug)]
pub struct CurrencyService;

```


The remaining method, wire_api, binds APIs defined by the service. We will use it to receive requests via REST API applying the logic we defined in CryptocurrencyApi earlier:

> NOTE: Add the following code to lib.rs

```rust
impl Service for CurrencyService {
    fn service_name(&self) -> &'static str {
        "cryptocurrency"
    }

    fn service_id(&self) -> u16 {
        SERVICE_ID
    }

    // Implements a method to deserialize transactions coming to the node.
    fn tx_from_raw(
        &self,
        raw: RawTransaction
    ) -> Result<Box<dyn Transaction>, failure::Error> {
        let tx = CurrencyTransactions::tx_from_raw(raw)?;
        Ok(tx.into())
    }

    fn state_hash(&self, _: &dyn Snapshot) -> Vec<Hash> {
        vec![]
    }

    // Links the service API implementation to Exonum.
    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        CryptocurrencyApi::wire(builder);
    }
}


```


11) Create Demo Blockchain


The service is ready. You can verify that the library code compiles by running cargo build in the shell. However, we do not have the means of processing requests to the service. To fix this, let us create a minimalistic blockchain network with one node and a single service we’ve just finished creating.

The code we are going to write is logically separate from the service itself. The service library could be connected to an Exonum-powered blockchain together with other services, while the demo blockchain is a specific example of its usage. For this reason, we will position the blockchain code as an example and place it into examples/demo.rs.


> Create demo.rs file in examples folder.


> NOTE: Add the following code in demo.rs


```rust
use exonum::{
    blockchain::{GenesisConfig, ValidatorKeys},
    node::{Node, NodeApiConfig, NodeConfig},
    storage::MemoryDB,
};
use cryptocurrency::CurrencyService;
fn node_config() -> NodeConfig {
    // Code goes here

    let (consensus_public_key, consensus_secret_key) =
        exonum::crypto::gen_keypair();
    let (service_public_key, service_secret_key) =
        exonum::crypto::gen_keypair();

    let validator_keys = ValidatorKeys {
        consensus_key: consensus_public_key,
        service_key: service_public_key,
    };
    let genesis = GenesisConfig::new(vec![validator_keys].into_iter());


    let api_address = "0.0.0.0:8000".parse().unwrap();
    let api_cfg = NodeApiConfig {
        public_api_address: Some(api_address),
        ..Default::default()
    };


    let peer_address = "0.0.0.0:2000";

    // Returns the value of the `NodeConfig` object from the `node_config` function
    NodeConfig {
        listen_address: peer_address.parse().unwrap(),
        service_public_key,
        service_secret_key,
        consensus_public_key,
        consensus_secret_key,
        genesis,
        external_address: peer_address.to_owned(),
        network: Default::default(),
        connect_list: Default::default(),
        api: api_cfg,
        mempool: Default::default(),
        services_configs: Default::default(),
        database: Default::default(),
        thread_pool_size: Default::default(),
    }
}




fn main() {
    exonum::helpers::init_logger().unwrap();
    let node = Node::new(
        MemoryDB::new(),
        vec![Box::new(CurrencyService)],
        node_config(),
        None,
    );
    node.run().unwrap();
}


```



Run the blockchain:

```
$ cargo build

$ RUST_LOG=info cargo run --example demo
```

This will start the blockchain node.



### Blockchain Transactions


###### 1) Create first wallet

> Request

API --> http://127.0.0.1:8000/api/explorer/v1/transactions

TYPE --> POST

Body --> 

```json
{
  "tx_body": "114e49a764813f2e92609d103d90f23dc5b7e94e74b3e08134c1272441614bd90000010000000a05416c69636587b54e335ef652ccae5112388d128e5162326f60d25196b34ad431e394ee2f77cfe72d201d7ba12db9b9ddd278235493dc444a3671a4710e87bad53411a45a0c"
}
```

> Response


{"tx_hash":"75a9d95694f22823ae01a6feafb3d4e27b55b83bd6897aa581456ea5da382dde"}


LOG --> 'Create the wallet: Wallet { pub_key: PublicKey(114e49a7...), name: "Alice", balance: 100 }'



###### 2) Create the Second Wallet


> Request

API -->

http://127.0.0.1:8000/api/explorer/v1/transactions

TYPE --> POST

Body --> 

```json
{
  "tx_body": "9359df9223bd4c263692a437e3d244b644c7b7f847db12cc556c2e25c73e61030000010000000a03426f62583236ff2afe268d31ca93ab0258cb3fea944551975d95888dbec88787fb5b1e23a044c4e674c6fbbb239ff7de83e8d3ba8ca57dc7e47a3eb52572f9dbd9df02"
}

```

> Response --> 


```json
{"tx_hash":"7a09053aa590704332b7a18f552150caa8b6e4f777afa4005d169038f481b7f7"}
```

LOG --> 'Create the wallet: Wallet { pub_key: PublicKey(9359df92...), name: "Bob", balance: 100 }'


###### 3) Transfer money between wallets

> Request

http://127.0.0.1:8000/api/explorer/v1/transactions

TYPE --> POST

Body -->

```json
{
  "tx_body": "114e49a764813f2e92609d103d90f23dc5b7e94e74b3e08134c1272441614bd90000010001000a220a209359df9223bd4c263692a437e3d244b644c7b7f847db12cc556c2e25c73e6103100f7611ddb5d15e4b77894fae770e5b15f19c07e0f7c7472e31fabe850f0067fb3ab4702130ba6325448d53516a8897a1d9228ba6a87b0e1224143c1b629c4d180b"
}
```

> Response -->


```json
{"tx_hash":"ae3afbe35f1bfd102daea2f3f72884f04784a10aabe9d726749b1188a6b9fe9b"}
```

LOG --> Transfer between wallets: Wallet { pub_key: PublicKey(114e49a7...), name: "Alice", balance: 85 } => Wallet { pub_key: PublicKey(9359df92...), name: "Bob", balance: 115 }



###### 4) Get all wallets

> Request

http://127.0.0.1:8000/api/services/cryptocurrency/v1/wallets

TYPE --> GET

> Response

```json
[{"pub_key":"114e49a764813f2e92609d103d90f23dc5b7e94e74b3e08134c1272441614bd9","name":"Alice","balance":85},{"pub_key":"9359df9223bd4c263692a437e3d244b644c7b7f847db12cc556c2e25c73e6103","name":"Bob","balance":115}]
```

###### 5) Info on specific wallet

> Request

http://127.0.0.1:8000/api/services/cryptocurrency/v1/wallet?pub_key=114e49a764813f2e92609d103d90f23dc5b7e94e74b3e08134c1272441614bd9


TYPE --> GET

> Response

```json

{
  "balance": "85",
  "name": "Alice",
  "pub_key": "114e49a764813f2e92609d103d90f23dc5b7e94e74b3e08134c1272441614bd9"
}

```

