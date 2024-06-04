use crate::{
    db::{add_tvl, AddTvl},
    indexer::{Handleable, HandlerParams},
};
use alloy::{primitives::Uint, rpc::types::eth::BlockNumberOrTag, sol, sol_types::SolEvent};
use async_trait::async_trait;

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

#[derive(Clone)]
pub struct RocketPoolHandler;

impl RocketPoolHandler {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

#[async_trait]
impl Handleable for RocketPoolHandler {
    fn get_event(&self) -> String {
        RocketMinipoolManager::MinipoolCreated::SIGNATURE.to_string()
    }

    async fn handle(&self, params: HandlerParams) {
        let blocknumber = params.log.block_number.unwrap();

        let rocket_vault_contract = RocketVault::new(
            "0x3bdc69c4e5e13e52a65f5583c23efb9636b469d6"
                .parse()
                .unwrap(),
            &params.provider,
        );

        let rocket_minipool_manager_contract = RocketMinipoolManager::new(
            "0x6d010c43d4e96d74c422f2e27370af48711b49bf"
                .parse()
                .unwrap(),
            &params.provider,
        );

        let rocket_node_staking_contract = RocketNodeStaking::new(
            "0x0d8d8f8541b12a0e1194b7cc4b6d954b90ab82ec"
                .parse()
                .unwrap(),
            &params.provider,
        );

        let mut total_eth: Uint<256, 4> = Uint::from(0);
        let mut total_rpl: Uint<256, 4> = Uint::from(0);

        let limit = 400;
        let mut offset = 0;

        let mut initialised_minipools: Uint<256, 4> = Uint::from(0);
        let mut prelaunch_minipools: Uint<256, 4> = Uint::from(0);
        let mut staking_minipools: Uint<256, 4> = Uint::from(0);
        let mut withdrawable_minipools: Uint<256, 4> = Uint::from(0);

        loop {
            println!("Get minipools: {} {}", offset, limit);
            let active_minipools = rocket_minipool_manager_contract
                .getMinipoolCountPerStatus(Uint::from(offset), Uint::from(limit))
                .block(alloy::rpc::types::eth::BlockId::Number(
                    BlockNumberOrTag::Number(blocknumber),
                ))
                .call()
                .await
                .unwrap();

            initialised_minipools += active_minipools.initialisedCount;
            prelaunch_minipools += active_minipools.prelaunchCount;
            staking_minipools += active_minipools.stakingCount;
            withdrawable_minipools += active_minipools.withdrawableCount;

            let mut total: u64 = 0;

            total += active_minipools.initialisedCount.to::<u64>();
            total += active_minipools.prelaunchCount.to::<u64>();
            total += active_minipools.stakingCount.to::<u64>();
            total += active_minipools.withdrawableCount.to::<u64>();
            total += active_minipools.dissolvedCount.to::<u64>();

            if total < limit {
                break;
            }

            offset += limit;
        }

        let mut eth_locked_in_minipools: Uint<256, 4> = Uint::from(0);

        eth_locked_in_minipools += initialised_minipools * Uint::from(16);
        eth_locked_in_minipools += prelaunch_minipools * Uint::from(32);
        eth_locked_in_minipools += staking_minipools * Uint::from(32);
        eth_locked_in_minipools += withdrawable_minipools * Uint::from(32);
        eth_locked_in_minipools = eth_locked_in_minipools * Uint::from(1e18);

        total_eth += eth_locked_in_minipools;

        let rocket_deposit_pool_eth = rocket_vault_contract
            .balanceOf(String::from("rocketDepositPool"))
            .block(alloy::rpc::types::eth::BlockId::Number(
                BlockNumberOrTag::Number(blocknumber),
            ))
            .call()
            .await
            .unwrap();

        total_eth += rocket_deposit_pool_eth._0;

        let total_rpl_stacked = rocket_node_staking_contract
            .getTotalRPLStake()
            .block(alloy::rpc::types::eth::BlockId::Number(
                BlockNumberOrTag::Number(blocknumber),
            ))
            .call()
            .await
            .unwrap();

        total_rpl += total_rpl_stacked._0;

        let rocket_dao_node_trusted_actions_rpl_balance = rocket_vault_contract
            .balanceOf(String::from("rocketDAONodeTrustedActions"))
            .block(alloy::rpc::types::eth::BlockId::Number(
                BlockNumberOrTag::Number(blocknumber),
            ))
            .call()
            .await
            .unwrap();

        total_rpl += rocket_dao_node_trusted_actions_rpl_balance._0;

        let rocket_auction_manager_rpl_balance = rocket_vault_contract
            .balanceOf(String::from("rocketAuctionManager"))
            .block(alloy::rpc::types::eth::BlockId::Number(
                BlockNumberOrTag::Number(blocknumber),
            ))
            .call()
            .await
            .unwrap();

        total_rpl += rocket_auction_manager_rpl_balance._0;

        println!("Blocknumber: {}", blocknumber);
        println!("Total ETH: {}", total_eth);
        println!("Total RPL: {}", total_rpl);

        let blocknumber = blocknumber as i64;
        let mut conn = params.conn.lock().unwrap();

        add_tvl(
            &mut conn,
            AddTvl {
                eth: total_eth.to_string(),
                rpl: total_rpl.to_string(),
                blocknumber,
            },
        );
    }
}
