## Exonum cryptocurrency teskit walkthrough


###### This tutorial describes testing of RUST services.

#### Preparing for integration testing


1) Exonum is typically packed as a rust crate. Correspondingly. service testing could be performed with the help of integration tests, in which the service is treated like a block box.


2) Exonum provides a handy tool called exonum-testkit crate.



> We will continue with cryptocurrency tutorial.

Add the following code to Cargo.toml

```rust

[dev-dependencies]
assert_matches = "1.3.0"
exonum-testkit = "0.10.2"
```

#### Testing kinds

There are two major kinds of testing enabled by exonum-testkit.


1) Transaction logic testing

Transaction logic testing treats the service as a gray box.  It uses the service schema to read information from the storage, and executes transactions by sending them directly to the Rust API of the testkit. 

2) API testing

It treats the service as a black box, using its HTTP APIs to process transactions and read requests. A good idea is to use this kind of testing to verify the API-specific code.


So both test cases follow the same pattern:

i) Initialize the testkit.

ii) Introduce changes to the blockchain via transactions.

iii) Use the service schema or read requests to check that the changes are as expected.


#### Testing transaction logic


Let's create tests/tx_logic.rs, a file which will contain the tests for transaction business logic.

> NOTE: Create test/tx_logic.rs file

> NOTE: Add the following code to tx_logic.rs file


```rust
use assert_matches::assert_matches;
use exonum::{
    api::Error,
    crypto::{gen_keypair, PublicKey, SecretKey},
    messages::{to_hex_string, RawTransaction, Signed},
};
use exonum_testkit::{txvec, ApiKind, TestKit, TestKitApi, TestKitBuilder};
use serde_json::json;
// Imports datatypes from the crate where the service is defined.
use exonum_cryptocurrency::{
    schema::{CurrencySchema, Wallet, WalletQuery},
    service::CurrencyService,
    transactions::{TxCreateWallet, TxTransfer},
};


fn init_testkit() -> TestKit {
    TestKitBuilder::validator()
        .with_service(CurrencyService)
        .create()
}


```


##### Wallet creation

This test is very simple: we want to create a single wallet with the help of the corresponding API call and make sure that the wallet is actually persisted by the blockchain.


```rust

#[test]
fn test_create_wallet() {
    let mut testkit = init_testkit();
    let (pub_key, sec_key) = gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::sign("Alice", &pub_key, &sec_key),
    ]);
    let snapshot = testkit.snapshot();
    let wallet = CurrencySchema::new(snapshot)
        .wallet(&pub_key)
        .expect("No wallet");

    assert_eq!(wallet.pub_key, pub_key);
    assert_eq!(wallet.name, "Alice");
    assert_eq!(wallet.balance, 100);
}
```


To run the test execure cargo test in the shell:


```
$ cargo test

```


##### Successful Transfer

Let’s test a transfer between two wallets. To do this, first, we need to initialize the testkit. Then we need to create the wallets and transfer funds between them. As per the code of the Cryptocurrency Service, the wallets are created with the initial balance set to 100.


> NOTE: Add the following code to tx_logic.rs file

```rust

#[test]
fn test_transfer(){
    let mut testkit = init_testkit();
    let (alice_pubkey, alice_key) = gen_keypair();
    let (bob_pubkey, bob_key) = gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::sign("Alice", &alice_pubkey, &alice_key),
        TxCreateWallet::sign("Bob", &bob_pubkey, &bob_key),
        TxTransfer::sign(
            &bob_pubkey,    // receiver
            10,             // amount
            0,              // seed
            &alice_pubkey,  // sender
            &alice_key,     // private key used to sign the transaction
        ),
    ]);

    let wallets = {
        let snapshot = testkit.snapshot();
        let schema = CurrencySchema::new(&snapshot);
        (schema.wallet(&alice_pubkey), schema.wallet(&bob_pubkey))
    };

    if let (Some(alice_wallet), Some(bob_wallet)) = wallets {
        assert_eq!(alice_wallet.balance, 90);
        assert_eq!(bob_wallet.balance, 110);
    } else {
        panic!("Wallets not persisted");
    }
    
}

```


##### Transfer to Non-Existing Wallet


Unlike in a real Exonum network, you can control which transactions the testkit will include into the next block. This allows testing different orderings of transactions, even those that would be hard (but not impossible) to reproduce in a real network.

Let’s test the case when Alice sends a transaction to Bob while Bob’s wallet is not committed. The test is quite similar to the previous one, with the exception of how the created transactions are placed into the block.


> NOTE: Add the following code to tx_logic.rs file


```rust
#[test]
fn test_transfer_to_nonexisting_wallet(){
    let mut testkit = init_testkit();
    let (alice_pubkey, alice_key) = gen_keypair();
    let (bob_pubkey, bob_key) = gen_keypair();
    testkit.create_block_with_transactions(txvec![
        TxCreateWallet::sign("Alice", &alice_pubkey, &alice_key),
        TxTransfer::sign(&bob_pubkey, 10, 0, &alice_pubkey, &alice_key),
        TxCreateWallet::sign("Bob", &bob_pubkey, &bob_key),
    ]);

    let wallets = {
        let snapshot = testkit.snapshot();
        let schema = CurrencySchema::new(&snapshot);
        (schema.wallet(&alice_pubkey), schema.wallet(&bob_pubkey))
    };

    if let (Some(alice_wallet), Some(bob_wallet)) = wallets {
        assert_eq!(alice_wallet.balance, 100);
        assert_eq!(bob_wallet.balance, 100);
    } else {
        panic!("Wallets not persisted");
    }


}
```

### Run cargo test

```
$ cargo test

```





