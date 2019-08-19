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
