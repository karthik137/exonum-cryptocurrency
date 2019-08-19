# exonum-cryptocurrency


### Steps to start the blockchain

#### Cargo build

```
$ cd cryptocurrency
```

```
$ cargo build
```


#### Run blockchain node

```
$ RUST_LOG=info cargo run --example demo
```


### Execute wallet API's

#### Create wallet1

```
$ curl -H "Content-Type: application/json" -X POST -d @create-wallet-1.json \
    http://127.0.0.1:8000/api/explorer/v1/transactions


```

###### Response

```json
{"tx_hash":"75a9d95694f22823ae01a6feafb3d4e27b55b83bd6897aa581456ea5da382dde"}
```

###### Blockchain LOG

```
Create the wallet: Wallet { pub_key: PublicKey(114e49a7...), name: "Alice", balance: 100 }
```


#### Create wallet2

```
$ curl -H "Content-Type: application/json" -X POST -d @create-wallet-2.json \
    http://127.0.0.1:8000/api/explorer/v1/transactions


```

###### Response

```json
{"tx_hash":"7a09053aa590704332b7a18f552150caa8b6e4f777afa4005d169038f481b7f7"}
```

###### Blokchain LOG

```
Create the wallet: Wallet { pub_key: PublicKey(9359df92...), name: "Bob", balance: 100 }
```

#### Transfer funds between the wallets

```
$ curl -H "Content-Type: application/json" -X POST -d @transfer-funds.json \
    http://127.0.0.1:8000/api/explorer/v1/transactions
```

###### Response 

```json
{"tx_hash":"ae3afbe35f1bfd102daea2f3f72884f04784a10aabe9d726749b1188a6b9fe9b"}
```

###### Blockchain LOG

```
Transfer between wallets: Wallet { pub_key: PublicKey(114e49a7...), name: "Alice", balance: 85 } => Wallet { pub_key: PublicKey(9359df92...), name: "Bob", balance: 115 }
```


#### Info on All Wallets


```
$ curl http://127.0.0.1:8000/api/services/cryptocurrency/v1/wallets

```


###### Response

```json
[{"pub_key":"114e49a764813f2e92609d103d90f23dc5b7e94e74b3e08134c1272441614bd9","name":"Alice","balance":85},{"pub_key":"9359df9223bd4c263692a437e3d244b644c7b7f847db12cc556c2e25c73e6103","name":"Bob","balance":115}]

```

#### Info on specific wallet

```
$ http://127.0.0.1:8000/api/services/cryptocurrency/v1/wallet?pub_key=114e49a764813f2e92609d103d90f23dc5b7e94e74b3e08134c1272441614bd9
```

###### Response

```json

{
  "balance": "85",
  "name": "Alice",
  "pub_key": "114e49a764813f2e92609d103d90f23dc5b7e94e74b3e08134c1272441614bd9"
}

```