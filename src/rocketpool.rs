use crate::{
    db::{add_tvl, AddTvl},
    indexer::HandlerParams,
};
use alloy::{primitives::Uint, rpc::types::eth::BlockNumberOrTag, sol};

sol!(
    #[sol(rpc)]
    RocketMinipoolManager,
    "abi/RocketMinipoolManager.json"
);

sol!(
    #[sol(rpc)]
    RocketNodeStaking,
    "abi/RocketNodeStaking.json"
);

sol!(
    #[sol(rpc)]
    RocketVault,
    "abi/RocketVault.json"
);

pub async fn handler(
    HandlerParams {
        log,
        provider,
        mut conn,
    }: HandlerParams<'_>,
) {
    println!("Log: {:?}", log);

    let blocknumber = log.block_number.unwrap();

    let rocket_minipool_manager_contract = RocketMinipoolManager::new(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
            .parse()
            .unwrap(),
        &provider,
    );

    let rocket_node_staking_contract = RocketNodeStaking::new(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
            .parse()
            .unwrap(),
        &provider,
    );

    let rocket_vault_contract = RocketVault::new(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
            .parse()
            .unwrap(),
        &provider,
    );

    let mut total_eth: i64 = 0;
    let mut total_rpl: i64 = 0;

    let mut limit = 1000;
    let mut offset = 0;

    let mut initialised_minipools: u64 = 0;
    let mut prelaunch_minipools: i64 = 0;
    let mut staking_minipools: i64 = 0;
    let mut withdrawable_minipools: i64 = 0;

    loop {
        let active_minipools = rocket_minipool_manager_contract
            .getMinipoolCountPerStatus(Uint::from(limit), Uint::from(offset))
            .block(alloy::rpc::types::eth::BlockId::Number(
                BlockNumberOrTag::Number(blocknumber),
            ))
            .call()
            .await
            .unwrap();
    }

    // add_tvl(
    //     &mut conn,
    //     AddTvl {
    //         eth: 2,
    //         rpl: 3,
    //         blocknumber,
    //     },
    // );
}
