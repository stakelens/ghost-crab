use crate::{
    db::{add_swell_tvl, AddSwellTVL},
    indexer::{Handleable, HandlerParams},
};
use alloy::{rpc::types::eth::BlockNumberOrTag, sol, sol_types::SolEvent};
use async_trait::async_trait;

sol!(
    #[sol(rpc)]
    swETH,
    "abis/swell/swETH.json"
);

#[derive(Clone)]
pub struct SwellHandler;

impl SwellHandler {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

#[async_trait]
impl Handleable for SwellHandler {
    // TODO: we should update multiple events to trigger a handler
    fn get_event(&self) -> String {
        swETH::ETHDepositReceived::SIGNATURE.to_string()
    }

    async fn handle(&self, params: HandlerParams) {
        let blocknumber = params.log.block_number.unwrap();

        let swETH_contract = swETH::new(
            "0xf951E335afb289353dc249e82926178EaC7DEd78"
                .parse()
                .unwrap(),
            &params.provider,
        );

        let total_supply = swETH_contract
            .totalSupply()
            .block(alloy::rpc::types::eth::BlockId::Number(
                BlockNumberOrTag::Number(blocknumber),
            ))
            .call()
            .await
            .unwrap();

        let rate = swETH_contract
            .getRate()
            .block(alloy::rpc::types::eth::BlockId::Number(
                BlockNumberOrTag::Number(blocknumber),
            ))
            .call()
            .await
            .unwrap();

        let blocknumber = blocknumber as i64;
        let mut conn = params.conn.lock().unwrap();

        add_swell_tvl(
            &mut conn,
            AddSwellTVL {
                sweth: total_supply._0.to_string(),
                rate: rate._0.to_string(),
                blocknumber,
            },
        );
    }
}
