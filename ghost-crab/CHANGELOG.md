# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0](https://github.com/stakelens/ghost-crab/compare/ghost-crab-v0.7.1...ghost-crab-v0.8.0) - 2024-07-26

### Added
- Load "event_handler" config dynamically at runtime
- load block handler config dynamically at runtime

### Other
- Refactor "get_provider"
- Refactor indexer and add errors

## [0.7.1](https://github.com/stakelens/ghost-crab/compare/ghost-crab-v0.7.0...ghost-crab-v0.7.1) - 2024-07-26

### Added
- Add ctx.block() to EventContext

## [0.7.0](https://github.com/stakelens/ghost-crab/compare/ghost-crab-v0.6.0...ghost-crab-v0.7.0) - 2024-07-25

### Added
- Add requests per second to the "config.json"

## [0.6.0](https://github.com/stakelens/ghost-crab/compare/ghost-crab-v0.5.1...ghost-crab-v0.6.0) - 2024-07-24

### Added
- Add rate limit layer

## [0.5.1](https://github.com/stakelens/ghost-crab/compare/ghost-crab-v0.5.0...ghost-crab-v0.5.1) - 2024-07-23

### Fixed
- Fix macro invalid address error

## [0.5.0](https://github.com/stakelens/ghost-crab/compare/ghost-crab-v0.4.0...ghost-crab-v0.5.0) - 2024-07-23

### Added
- Now, we do not prefetch the block in the block handlers. To get the current block you have to call `ctx.block()` instead of `ctx.block`

### Other
- Rename BlockContext.block -> BlockContext.block_number

## [0.4.0](https://github.com/stakelens/ghost-crab/compare/ghost-crab-v0.3.1...ghost-crab-v0.4.0) - 2024-07-23

### Other
- Remove network from cache layer
- Add cache layer to the provider and remove the rpc proxy server

## [0.3.1](https://github.com/stakelens/ghost-crab/compare/ghost-crab-v0.3.0...ghost-crab-v0.3.1) - 2024-07-21

### Other
- Add typed errors for cache loading

## [0.3.0](https://github.com/stakelens/ghost-crab/compare/ghost-crab-v0.2.1...ghost-crab-v0.3.0) - 2024-07-19

### Other
- Adds step, network, start_block and execution_mode to "block_handler"
- Adds "network" to handler
- Adds execution_mode to handler
- Adds "ghost_crab_common" to share the config struct
- Adds "template" proc-macro and renames "handle" to "event_handler"

## [0.2.1](https://github.com/stakelens/ghost-crab/compare/ghost-crab-v0.2.0...ghost-crab-v0.2.1) - 2024-07-13

### Other
- add clippy/rustfmt

## [0.2.0](https://github.com/vistastaking/ghost-crab/compare/ghost-crab-v0.1.0...ghost-crab-v0.2.0) - 2024-06-28

### Other
- Adds contract_address to context
- release ([#24](https://github.com/vistastaking/ghost-crab/pull/24))

## [0.1.0](https://github.com/vistastaking/ghost-crab/releases/tag/ghost-crab-v0.1.0) - 2024-06-27

### Fixed
- macros imports

### Other
- lib name underscore
- add missing fields
- workspaces
