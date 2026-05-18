-- =========================================================
-- TRON AML DATABASE
-- Chainalysis-style Architecture
-- Optimized for:
--   - AML tracing
--   - graph export
--   - exchange detection
--   - Neo4j ingestion
--   - high-speed ClickHouse analytics
-- =========================================================

CREATE DATABASE IF NOT EXISTS tron_db;

-- =========================================================
-- BLOCKS
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.blocks
(
    block_number UInt64,
    block_hash String,
    parent_hash String,

    tx_count UInt32,

    witness_address String,

    block_size UInt32,

    timestamp DateTime,

    inserted_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY block_number;

-- =========================================================
-- TRANSACTIONS
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.transactions
(
    tx_hash String,

    block_number UInt64,

    timestamp DateTime,

    from_address String,
    to_address String,

    contract_address String,

    contract_type String,

    amount Decimal(38,0),

    fee Decimal(38,0),
    energy_fee Decimal(38,0),
    net_fee Decimal(38,0),

    energy_usage UInt64,
    energy_usage_total UInt64,

    net_usage UInt64,

    status UInt8,

    memo String,

    raw_data String,

    inserted_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY (block_number, tx_hash);

-- =========================================================
-- RAW LOGS
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.raw_logs
(
    tx_hash String,

    block_number UInt64,

    log_index UInt32,

    contract_address String,

    topics Array(String),

    data String,

    removed UInt8,

    timestamp DateTime,

    inserted_at DateTime DEFAULT now()
    )
    ENGINE = MergeTree()
    ORDER BY (
                 block_number,
                 tx_hash,
                 log_index
             );

-- =========================================================
-- TOKEN METADATA
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.token_metadata
(
    token_address String,

    token_name String,
    token_symbol String,

    decimals UInt8,

    total_supply Decimal(38,0),

    owner_address String,

    is_verified UInt8,

    first_seen_block UInt64,

    created_at DateTime DEFAULT now(),

    updated_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(updated_at)
    ORDER BY token_address;

-- =========================================================
-- TOKEN TRANSFERS
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.token_transfers
(
    tx_hash String,

    block_number UInt64,

    timestamp DateTime,

    log_index UInt32,

    token_address String,

    token_symbol String,

    decimals UInt8,

    from_address String,
    to_address String,

    amount Decimal(38,0),

    is_mint UInt8,
    is_burn UInt8,

    event_signature String,

    inserted_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY (
                 block_number,
                 tx_hash,
                 log_index
             );

-- =========================================================
-- INTERNAL TRANSFERS
-- REQUIRED FOR:
--   swaps
--   bridges
--   contract tracing
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.internal_transfers
(
    tx_hash String,

    block_number UInt64,

    timestamp DateTime,

    trace_id String,

    caller String,
    callee String,

    amount Decimal(38,0),

    call_type String,

    depth UInt16,

    inserted_at DateTime DEFAULT now()
    )
    ENGINE = MergeTree()
    ORDER BY (
                 tx_hash,
                 trace_id
             );

-- =========================================================
-- ADDRESS RELATIONSHIPS
-- PRIMARY GRAPH EXPORT TABLE
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.address_relationships
(
    from_address String,

    to_address String,

    token_address String,

    tx_hash String,

    block_number UInt64,

    timestamp DateTime,

    amount Decimal(38,0),

    transfer_type String,
    -- native
    -- trc20
    -- swap
    -- bridge
    -- liquidity
    -- stake
    -- unstake
    -- internal

    protocol String,

    risk_score UInt8,

    inserted_at DateTime DEFAULT now()
    )
    ENGINE = MergeTree()
    ORDER BY (
                 from_address,
                 to_address,
                 block_number
             );

-- =========================================================
-- CONTRACT INTERACTIONS
-- HEART OF AML INTELLIGENCE
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.contract_interactions
(
    tx_hash String,

    block_number UInt64,

    timestamp DateTime,

    caller String,

    contract_address String,

    protocol String,

    interaction_type String,
    -- swap
    -- bridge
    -- stake
    -- unstake
    -- liquidity_add
    -- liquidity_remove
    -- mint
    -- burn
    -- borrow
    -- repay

    method_id String,

    token_in String,
    amount_in Decimal(38,0),

    token_out String,
    amount_out Decimal(38,0),

    confidence Float32,

    inserted_at DateTime DEFAULT now()
    )
    ENGINE = MergeTree()
    ORDER BY (
                 contract_address,
                 interaction_type,
                 block_number
             );

-- =========================================================
-- WALLET STATE
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.wallet_state
(
    address String,
    native_balance Decimal(38,0),
    account_type String,
    is_contract UInt8,
    tx_count UInt64,
    first_seen DateTime,
    last_seen DateTime,
    last_active_block UInt64,
    risk_score UInt8,
    updated_at DateTime DEFAULT now()
)
    ENGINE = ReplacingMergeTree(updated_at)
    ORDER BY address;

-- =========================================================
-- LEGACY WALLET/OWNER COMPATIBILITY
-- Used by the current Rust wallet identity code while wallet_state
-- migration is completed.
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.wallet_info
(
    address String,
    balance String,
    nonce UInt64,
    type String,
    person_id String,
    inserted_at DateTime DEFAULT now()
)
    ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY address;

CREATE TABLE IF NOT EXISTS tron_db.owner_info
(
    address String,
    person_name String,
    person_id String,
    personal_id UInt16,
    inserted_at DateTime DEFAULT now()
)
    ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY address;

-- =========================================================
-- ADDRESS TAGS
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.address_tags
(
    address String,
    tag String,
    tag_type String,
    -- exchange
    -- bridge
    -- mixer
    -- sanctioned
    -- otc
    -- scam
    -- protocol
    confidence Float32,
    source String,
    created_at DateTime DEFAULT now()
)
    ENGINE = MergeTree()
    ORDER BY ( address, tag );

-- =========================================================
-- ENTITY CLUSTERING
-- MOST IMPORTANT AML TABLE
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.address_entity
(
    address String,
    entity_id String,
    entity_name String,
    entity_type String,
    -- exchange
    -- bridge
    -- mixer
    -- otc
    -- darknet
    -- protocol
    -- sanctioned
    confidence Float32,
    source String,
    created_at DateTime DEFAULT now()
)
    ENGINE = ReplacingMergeTree(created_at)
    ORDER BY address;

-- =========================================================
-- EXCHANGE DEPOSIT DETECTION
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.exchange_deposit_addresses
(
    address String,
    exchange_name String,
    hot_wallet String,
    confidence Float32,
    detection_method String,
    first_seen_block UInt64,
    last_seen_block UInt64,
    inserted_at DateTime DEFAULT now()
)
    ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY address;

-- =========================================================
-- CONTRACT METADATA
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.contract_metadata
(
    contract_address String,
    protocol_name String,
    contract_type String,
    -- dex
    -- bridge
    -- lending
    -- staking
    -- router
    -- token
    -- nft
    -- mixer
    creator_address String,
    implementation_address String,
    verified UInt8,
    created_block UInt64,
    created_at DateTime DEFAULT now(),
    updated_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(updated_at)
    ORDER BY contract_address;

-- =========================================================
-- AML EVENTS
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.aml_events
(
    event_id UUID,
    tx_hash String,
    block_number UInt64,
    timestamp DateTime,
    event_type String,
    -- swap
    -- bridge_in
    -- bridge_out
    -- liquidity_add
    -- liquidity_remove
    -- mint
    -- burn
    -- peel_chain
    -- mixer_interaction
    -- exchange_deposit
    protocol String,
    user_address String,
    counterparty String,

    token_in String,
    amount_in Decimal(38,0),

    token_out String,
    amount_out Decimal(38,0),

    confidence Float32,
    inserted_at DateTime DEFAULT now()
    )
    ENGINE = MergeTree()
    ORDER BY ( event_type,block_number );

-- =========================================================
-- TRANSACTION FEATURES
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.transaction_features
(
    tx_hash String,
    block_number UInt64,
    timestamp DateTime,

    is_swap UInt8,
    is_bridge UInt8,
    is_mint UInt8,
    is_burn UInt8,
    is_liquidity_add UInt8,
    is_liquidity_remove UInt8,

    is_contract_call UInt8,
    unique_tokens UInt16,
    participants UInt16,
    hop_count UInt16,

    fan_in UInt16,
    fan_out UInt16,

    inserted_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY ( block_number,tx_hash );

-- =========================================================
-- TRANSACTION RISK
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.transaction_risk
(
    tx_hash String,
    block_number UInt64,
    timestamp DateTime,
    risk_score UInt8,
    risk_level String,
    is_swap UInt8,
    is_bridge UInt8,
    is_contract_call UInt8,
    unique_tokens UInt16,
    participants UInt16,
    risk_reasons Array(String),
    exposure_depth UInt16,
    touches_sanctioned UInt8,
    touches_mixer UInt8,
    touches_exchange UInt8,
    inserted_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY (risk_score, block_number );

-- =========================================================
-- WALLET RISK
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.wallet_risk
(
    address String,
    risk_score UInt8,
    risk_level String,
    sanctioned_exposure UInt8,
    mixer_exposure UInt8,
    darknet_exposure UInt8,
    exchange_cashout_probability Float32,
    first_calculated DateTime,
    updated_at DateTime DEFAULT now()
)
    ENGINE = ReplacingMergeTree(updated_at)
    ORDER BY address;

-- =========================================================
-- EXPOSURE PATHS
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.exposure_paths
(
    source_address String,
    target_address String,
    path_hash String,
    depth UInt16,
    total_amount Decimal(38,0),
    first_seen DateTime,
    last_seen DateTime,
    risk_score UInt8
)
    ENGINE = MergeTree()
    ORDER BY (source_address,target_address, depth);

-- =========================================================
-- TOKEN BALANCE DELTA
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.address_token_delta
(
    tx_hash String,
    block_number UInt64,
    timestamp DateTime,
    address String,
    token_address String,
    delta Decimal(38,0),
    direction Int8,
    inserted_at DateTime DEFAULT now()
)
    ENGINE = MergeTree()
    ORDER BY (address,token_address, block_number);

-- =========================================================
-- FINAL TOKEN BALANCES
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.address_token_balance
(
    address String,
    token_address String,
    balance Decimal(38,0)
)
    ENGINE = SummingMergeTree()
    ORDER BY ( address, token_address );

-- =========================================================
-- MATERIALIZED VIEW
-- TOKEN BALANCES
-- =========================================================

CREATE MATERIALIZED VIEW IF NOT EXISTS tron_db.mv_token_balance
TO tron_db.address_token_balance
AS
SELECT
    address,
    token_address,
    delta AS balance
FROM tron_db.address_token_delta;

-- =========================================================
-- Method Signatures
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.method_signatures
(
    method_id String,
    method_name String,
    protocol String,
    category String
)
    ENGINE = MergeTree()
ORDER BY method_id;

---------------------------------------------------------
-- EXCHANGE ENTITIES
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.exchange_entities
(
    entity_id String,
    exchange_name String,
    exchange_type String,
    confidence Float32,
    created_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(created_at)
    ORDER BY entity_id;

---------------------------------------------------------
-- EXCHANGE ADDRESSES
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.exchange_addresses
(
    address String,
    entity_id String,
    exchange_name String,
    address_role String,
    confidence Float32,
    detection_source String,
    first_seen_block UInt64,
    last_seen_block UInt64,
    created_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(created_at)
    ORDER BY address;

---------------------------------------------------------
-- ADDRESS CLUSTERS
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.address_clusters
(
    cluster_id UUID,
    address String,
    cluster_type String,
    confidence Float32,
    created_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(created_at)
    ORDER BY (cluster_id, address);

---------------------------------------------------------
-- EXCHANGE FLOWS
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.exchange_flows
(
    tx_hash String,
    block_number UInt64,
    from_address String,
    to_address String,
    exchange_name String,
    flow_type String,
    token_address String,
    amount UInt256,
    confidence Float32,
    created_at DateTime DEFAULT now()
)
    ENGINE = MergeTree()
    ORDER BY (block_number, tx_hash);

---------------------------------------------------------
-- EXPOSURE SEEDS
---------------------------------------------------------

CREATE TABLE IF NOT EXISTS tron_db.exposure_seeds
(
    address String,
    entity_name String,
    entity_type String,
    risk_level UInt8,
    source String,
    created_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(created_at)
    ORDER BY address;

CREATE TABLE IF NOT EXISTS tron_db.address_exposure
(
    source_address String,
    exposed_address String,
    hop_distance UInt8,
    exposure_score Float64,
    path_count UInt32,
    last_tx_hash String,
    last_seen_block UInt64,
    exposure_type String,
    updated_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(updated_at)
    ORDER BY ( source_address, exposed_address );

---------------------------------------------------------
-- EXCHANGE CLUSTERS
---------------------------------------------------------

CREATE TABLE IF NOT EXISTS tron_db.exchange_clusters
(
    cluster_id String,
    exchange_name String,
    address String,
    role String,
    confidence Float32,
    discovered_from String,
    created_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(created_at)
    ORDER BY (cluster_id, address);

---------------------------------------------------------
-- ADDRESS PROFILES
---------------------------------------------------------

CREATE TABLE IF NOT EXISTS tron_db.address_profiles
(
    address String,
    total_in_tx UInt64,
    total_out_tx UInt64,
    unique_senders UInt64,
    unique_receivers UInt64,
    total_volume_in UInt256,
    total_volume_out UInt256,
    interacted_tokens UInt32,
    probable_exchange UInt8,
    probable_deposit_wallet UInt8,
    probable_sweeper UInt8,
    risk_score Float32,
    updated_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(updated_at)
    ORDER BY address;

---------------------------------------------------------
-- ADDRESS COUNTERPARTIES
---------------------------------------------------------

CREATE TABLE IF NOT EXISTS tron_db.address_counterparties
(
    address String,
    counterparty String,
    direction String,
    token_address String,
    total_txs SimpleAggregateFunction(sum, UInt64),
    total_volume SimpleAggregateFunction(sum, UInt256),
    first_seen SimpleAggregateFunction(min, UInt64),
    last_seen SimpleAggregateFunction(max, UInt64),
    updated_at DateTime DEFAULT now()
    )
    ENGINE = AggregatingMergeTree()
    ORDER BY (address,counterparty,direction,token_address);

-- =========================================================
-- SYNC STATE
-- =========================================================

CREATE TABLE IF NOT EXISTS tron_db.sync_state
(
    chain String,
    last_synced_block UInt64,
    updated_at DateTime DEFAULT now()
    )
    ENGINE = ReplacingMergeTree(updated_at)
    ORDER BY chain;

-- =========================================================
-- PERFORMANCE INDEXES
-- =========================================================

ALTER TABLE tron_db.transaction_features
    ADD INDEX IF NOT EXISTS idx_swap (is_swap)
TYPE minmax
GRANULARITY 4;

ALTER TABLE tron_db.transaction_risk
    ADD INDEX IF NOT EXISTS idx_risk (risk_score)
TYPE minmax
GRANULARITY 4;

ALTER TABLE tron_db.contract_interactions
    ADD INDEX IF NOT EXISTS idx_interaction (interaction_type)
TYPE set(100)
GRANULARITY 4;

ALTER TABLE tron_db.address_relationships
    ADD INDEX IF NOT EXISTS idx_transfer_type (transfer_type)
TYPE set(100)
GRANULARITY 4;

ALTER TABLE tron_db.address_entity
    ADD INDEX IF NOT EXISTS idx_entity_type (entity_type)
TYPE set(100)
GRANULARITY 4;
