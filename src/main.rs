use alloy::{providers::ProviderBuilder, sol};
use indexer::{process_logs, HandlerParams, ProcessLogsParams};
mod indexer;

sol!(
    #[sol(rpc)]
    IERC20,
    "abi/IERC20.json"
);

#[tokio::main]
async fn main() {
    let rpc_url = ""
        .parse()
        .unwrap();

    let provider = ProviderBuilder::new().on_http(rpc_url);

    process_logs(ProcessLogsParams {
        from_block: 19_796_144,
        to_block: 19_796_144 + 10,
        event: "Transfer(address,address,uint256)".parse().unwrap(),
        handler: |HandlerParams { log, client }| {
            println!("Log: {:?}", log);
        },
        provider: provider.clone(),
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
