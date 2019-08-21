use assert_matches::assert_matches;
use exonum::{
    api::Error,
    crypto::{gen_keypair, PublicKey, SecretKey},
    messages::{to_hex_string, RawTransaction, Signed},
};
use exonum_testkit::{txvec, ApiKind, TestKit, TestKitApi, TestKitBuilder};
use serde_json::json;

// Imports datatypes from the crate where the service is defined.
use cryptocurrency::{
    CurrencySchema, 
    Wallet, 
    WalletQuery,
    CurrencyService,
    TxCreateWallet, 
    TxTransfer,
};


fn init_testkit() -> TestKit {
    TestKitBuilder::validator()
        .with_service(CurrencyService)
        .create()
}

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


