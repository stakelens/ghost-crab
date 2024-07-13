<div align="center">
    <h1>GhostCrab 👻🦀</h1>
    <big>Ethereum smart contracts indexer SDK written in Rust</big>
    <div>
    <br/>
        <a href="https://github.com/stakelens/ghost-crab/pulse"><img src="https://img.shields.io/github/last-commit/stakelens/ghost-crab.svg"/></a>
        <a href="https://github.com/stakelens/ghost-crab/pulls"><img src="https://img.shields.io/github/issues-pr/stakelens/ghost-crab.svg"/></a>
        <a href="https://github.com/stakelens/ghost-crab/issues"><img src="https://img.shields.io/github/issues-closed/stakelens/ghost-crab.svg"/></a>
    </div>
</div>
<br/>
</div>

## Introduction

GhostCrab is a rust library that allows you to index Ethereum smart contracts. It provides a simple and easy-to-use API for building performant and scalable indexers.

## Getting Started

To get started with GhostCrab, you need to install the Rust toolchain and the `ghost-crab` crate. You can find the installation instructions in the [Rust documentation](https://www.rust-lang.org/tools/install).

Once you have installed the Rust toolchain, you can add the `ghost-crab` crate to your project's `Cargo.toml` file:

```toml
[dependencies]
ghost-crab = "0.2.0"
```

## Usage

To use GhostCrab, you need to create a new instance of the `Indexer` struct. This struct provides methods for loading event handlers and block handlers, as well as starting the indexer.

```rust
use ghost_crab::prelude::*;

use handlers::etherfi;
use handlers::stader;

#[tokio::main]
async fn main() {
    let mut indexer = ghost_crab::Indexer::new();

    indexer
        .load_event_handler(etherfi::EtherFiTVLUpdated::new())
        .await;

    indexer
        .load_block_handler(stader::StaderBlockHandler::new())
        .await;

    indexer.start().await;
}
```

## Event Handlers

Event handlers are used to process events emitted by smart contracts. They are defined as closures that implement the `Handler` trait. The `Handler` trait provides methods for accessing the event data, the contract address, and other useful information.

Here's an example of an event handler that processes `TVLUpdated` events emitted by the EtherFi Oracle contract:

```rust
use alloy::eips::BlockNumberOrTag;
use ghost_crab::prelude::*;

#[handler(EtherFi.TVLUpdated)]
async fn EtherFiTVLUpdated(ctx: Context) {
    let block_number = ctx.log.block_number.unwrap() as i64;
    let current_tvl = event._currentTvl.to_string();
    let log_index = ctx.log.log_index.unwrap() as i64;

    let block = ctx
        .provider
        .get_block_by_number(
            BlockNumberOrTag::Number(ctx.log.block_number.unwrap()),
            false,
        )
        .await
        .unwrap()
        .unwrap();

    let block_timestamp = block.header.timestamp as i64;

    // Save the data to your database
}
```

In the current version, we do not offer any kind of abstractions for the DB interactions, so you will have to use an external library like `sqlx` to interact with the DB, and save the data in your desired format.

In the above example, `EtherFi` is defined in the configuration as follows:

```json
{
  "database": "$DATABASE_URL",
  "dataSources": {
    "EtherFi": {
      "startBlock": 105927637,
      "address": "0x6329004E903B7F420245E7aF3f355186f2432466",
      "abi": "abis/etherfi/TVLOracle.json",
      "network": "optimism"
    }
  },
  "networks": {
    "optimism": "$OPT_RPC_URL"
  }
}
```

## Block Handlers

Block handlers are used to process blocks. They are defined as closures that implement the `BlockHandler` trait. The `BlockHandler` trait provides methods for accessing the block data and other useful information.

Here's an example of a block handler that processes blocks:

```rust
use alloy::{rpc::types::eth::BlockNumberOrTag, sol};
use ghost_crab::prelude::*;

sol!(
    #[sol(rpc)]
    StaderStakePoolsManager,
    "abis/stader/StaderStakePoolsManager.json"
);

#[block_handler(Stader)]
async fn StaderBlockHandler(ctx: BlockContext) {
    let block_number = ctx.block.header.number.unwrap();

    let stader_staking_manager = StaderStakePoolsManager::new(
        "0xcf5EA1b38380f6aF39068375516Daf40Ed70D299"
            .parse()
            .unwrap(),
        &ctx.provider,
    );

    let total_assets = stader_staking_manager
        .totalAssets()
        .block(alloy::rpc::types::eth::BlockId::Number(
            BlockNumberOrTag::Number(block_number),
        ))
        .call()
        .await
        .unwrap();

    let db = db::get().await;

    let eth = total_assets._0.to_string();
    let block_number = block_number as i64;
    let block_timestamp = ctx.block.header.timestamp as i64;

    // Save the data to your database
}

```

In the above example, `Stader` is defined in the configuration as follows:

```json
{
  "database": "$DATABASE_URL",
  "blockHandlers": {
    "Stader": {
      "startBlock": 17416153,
      "network": "ethereum",
      "step": 720
    }
  },
  "networks": {
    "ethereum": "$ETH_RPC_URL"
  }
}
```

## Templates

Templates are ideal to dynamically trigger new indexing processes. They are defined as closures that implement the `Handler` trait. The `Handler` trait provides methods for accessing the event data, the contract address, and other useful information.

Here's an example on how to use templates:

```rust
use alloy::eips::BlockNumberOrTag;
use ghost_crab::prelude::*;

#[handler(ETHVault.Deposited)]
async fn ETHVaultDeposited(ctx: Context) {
    // Handler Logic
}

#[handler(VaultsRegistry.VaultAdded)]
async fn VaultsRegistry(ctx: Context) {
    let vault = event.vault.to_string();

    ctx.templates
        .start(Template {
            address: vault.clone(),
            start_block: ctx.log.block_number.unwrap(),
            handler: ETHVaultDeposited::new(),
        })
        .await;
}
```

In the above example, we are tracking a `VaultsRegistry` contract which emits a `VaultAdded` event every time a new vault is added. Under the hood, GhostCrab is using the `TemplateManager` to start a new indexing process for the `ETHVault` contract on the specified address, and start block.

In this particular case, there is no way we could have known the address of the `ETHVault` contract before the `VaultAdded` event was emitted, so this is when templates come handy to dynamically start the indexing processes for new contracts.

## Configuration

GhostCrab uses a configuration file to specify the data sources, templates, and block handlers. Here's an example of a configuration file:

```json
{
  "data_sources": {
    "MyDataSourceName": {
      "abi": "my_contract_abi.json",
      "address": "0x1234567890123456789012345678901234567890",
      "start_block": 1000000,
      "network": "mainnet"
    }
  },
  "templates": {
    "MySecondDataSourceName": {
      "abi": "my_second_contract_abi.json",
      "network": "mainnet"
    }
  },
  "networks": {
    "mainnet": "$MAINNET_RPC_URL"
  },
  "block_handlers": {
    "MyThirdDataSourceName": {
      "start_block": 1000000,
      "network": "mainnet"
    }
  }
}
```

In summary:

- If you want to use an environment variable, you can use the `$ENV_VAR` syntax within the configuration file.
- If you want to create an event handler, you need to define a data source. This data source will be loaded by the proc macro `handler` (event handler).
- If you want to create a template, you need to define a template. This template will be loaded by the proc macro `handler` (event handler).
- If you want to create a block handler, you need to define a block handler. This block handler will be loaded by the procedural macro `block_handler`.

Note: the `handler` proc macro, tries to look for a data source first, and if it doesn't find one, it will look for a template.

# Examples

If you want to see some examples of how to use GhostCrab, you can check out our [indexers](https://github.com/stakelens/indexers) repo, where we maintain a collection of smart contracts indexers for our dashboard [Stakelens](https://stakelens.com).
