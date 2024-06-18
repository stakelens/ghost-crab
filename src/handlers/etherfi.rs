use crate::{
    db::{add_etherfi_tvl, AddEtherfiTVL},
    indexer::{Handleable, HandlerParams},
};
use alloy::{rpc::types::eth::BlockNumberOrTag, sol, sol_types::SolEvent};
use async_trait::async_trait;

sol!(
    #[sol(rpc)]
    TVLOracle,
    "abis/etherfi/TVLOracle.json"
);

#[derive(Clone)]
pub struct EtherfiHandler;

impl EtherfiHandler {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

#[async_trait]
impl Handleable for EtherfiHandler {
    fn get_event_signature(&self) -> String {
        TVLOracle::TVLUpdated::SIGNATURE.to_string()
    }

    async fn handle(&self, params: HandlerParams) {
        let blocknumber = params.log.block_number.unwrap();

        let tvl_oracle_contract = TVLOracle::new(
            "0x6329004E903B7F420245E7aF3f355186f2432466"
                .parse()
                .unwrap(),
            &params.provider,
        );

        let tvl = tvl_oracle_contract
            .getTvl()
            .block(alloy::rpc::types::eth::BlockId::Number(
                BlockNumberOrTag::Number(blocknumber),
            ))
            .call()
            .await
            .unwrap();

        println!("Blocknumber: {}", blocknumber);
        println!("TVL: {}", tvl._0);

        let blocknumber = blocknumber as i64;
        let mut conn = params.conn.lock().unwrap();

        add_etherfi_tvl(
            &mut conn,
            AddEtherfiTVL {
                eth: tvl._0.to_string(),
                blocknumber,
            },
        );
    }
}
