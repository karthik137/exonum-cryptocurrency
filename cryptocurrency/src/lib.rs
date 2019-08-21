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
use exonum::{
    crypto::{SecretKey},
    messages::{Message, Signed},
};

// Service identifier
const SERVICE_ID: u16 = 1;
// Starting balance of a newly created wallet
const INIT_BALANCE: u64 = 100;


//Declare Persistent Data



// add proto module
mod proto;

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::cryptocurrency::Wallet")]
pub struct Wallet {
    pub pub_key: PublicKey,
    pub name: String,
    pub balance: u64,
}

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


//Create Schema


pub struct CurrencySchema<T> {
    view: T,
}



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


impl<'a> CurrencySchema<&'a mut Fork> {
    pub fn wallets_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        MapIndex::new("cryptocurrency.wallets", &mut self.view)
    }
}



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



#[derive(Serialize, Deserialize, Clone, Debug, TransactionSet)]
pub enum CurrencyTransactions {
    /// Create wallet transaction.
    CreateWallet(TxCreateWallet),
    /// Transfer tokens transaction.
    Transfer(TxTransfer),
}


    impl TxCreateWallet {
        #[doc(hidden)]
        pub fn sign(name: &str, pk: &PublicKey, sk: &SecretKey) -> Signed<RawTransaction> {
            Message::sign_transaction(
                Self {
                    name: name.to_owned(),
                },
                SERVICE_ID,
                *pk,
                sk,
            )
        }
    }
    
    impl TxTransfer {
        #[doc(hidden)]
        pub fn sign(
            to: &PublicKey,
            amount: u64,
            seed: u64,
            pk: &PublicKey,
            sk: &SecretKey,
        ) -> Signed<RawTransaction> {
            Message::sign_transaction(
                Self {
                    to: *to,
                    amount,
                    seed,
                },
                SERVICE_ID,
                *pk,
                sk,
            )
        }
    }



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


// create wallet transaction
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



//API for the crypto currency

//fn my_method(state: &ServiceApiState, query: MyQuery) -> api::Result<MyResponse>
//struct CryptocurrencyApi;
pub struct CryptocurrencyApi;
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


// wire api

impl CryptocurrencyApi {
    pub fn wire(builder: &mut ServiceApiBuilder) {
        // Binds handlers to the specific routes.
        builder
            .public_scope()
            .endpoint("v1/wallet", Self::get_wallet)
            .endpoint("v1/wallets", Self::get_wallets);
    }
}


#[derive(Debug)]
pub struct CurrencyService;


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
