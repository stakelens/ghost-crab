use alloy::{providers::ProviderBuilder, sol};
use db::{add_tvl, establish_connection, AddTvl};
use indexer::{process_logs, HandlerParams, ProcessLogsParams};
mod db;
mod indexer;
mod models;
mod schema;

sol!(
    #[sol(rpc)]
    IERC20,
    "abi/IERC20.json"
);

fn handler(
    HandlerParams {
        log,
        client,
        mut conn,
    }: HandlerParams,
) {
    println!("Log: {:?}", log);
    add_tvl(&mut conn, AddTvl { eth: 2 });
}

#[tokio::main]
async fn main() {
    let conn = establish_connection();

    let rpc_url = "https://lb.nodies.app/v1/eda527f40f4c48698a739e2dfae256b5"
        .parse()
        .unwrap();

    let provider = ProviderBuilder::new().on_http(rpc_url);

    process_logs(ProcessLogsParams {
        from_block: 19_796_144,
        to_block: 19_796_144 + 10,
        event: "Transfer(address,address,uint256)".parse().unwrap(),
        handler: handler,
        provider: provider.clone(),
        conn,
    })
    .await;

    let contract = IERC20::new(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
            .parse()
            .unwrap(),
        &provider,
    );

    let IERC20::totalSupplyReturn { _0 } = contract.totalSupply().call().await.unwrap();
    println!("WETH total supply is {_0}");
}
