CREATE DATABASE IF NOT EXISTS tron_db;

---------------------------------------------------------
-- WALLET INFO
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.wallet_info (
                                                   address String,              -- Base58 (T...)
                                                   balance String,              -- SUN (raw)
                                                   nonce UInt64,                -- tx count
                                                   wallet_type String,          -- wallet | smart_contract | exchange
                                                   person_id String,
                                                   inserted_at DateTime DEFAULT now()
    ) ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY address;

---------------------------------------------------------
-- TRANSACTIONS
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.transactions (
                                                    hash String,
                                                    block_number UInt64,
                                                    from_addr String,
                                                    to_addr String,
                                                    value String,                -- SUN
                                                    contract_type String,        -- TransferContract / TriggerSmartContract / ...
                                                    sensivity UInt8,
                                                    inserted_at DateTime DEFAULT now()
    ) ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY (block_number, hash);

---------------------------------------------------------
-- OWNER INFO
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.owner_info (
                                                  address String,
                                                  person_name String,
                                                  person_id String,
                                                  personal_id UInt16,
                                                  inserted_at DateTime DEFAULT now()
    ) ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY address;

---------------------------------------------------------
-- ADDRESS TAGS
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.address_tags (
                                                    address String,
                                                    tag String,
                                                    created_at DateTime DEFAULT now()
    ) ENGINE = MergeTree()
    ORDER BY (address, tag);

---------------------------------------------------------
-- TOKEN TRANSFERS (TRC20)
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.token_transfers (
                                                       tx_hash String,
                                                       block_number UInt64,
                                                       log_index UInt32,
                                                       token_address String,
                                                       from_addr String,
                                                       to_addr String,
                                                       amount String,
                                                       inserted_at DateTime DEFAULT now()
    ) ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY (tx_hash, log_index);

---------------------------------------------------------
-- TOKEN DELTA (CANONICAL)
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.address_token_delta (
                                                           tx_hash String,
                                                           log_index UInt32,
                                                           direction UInt8,     -- 0 = out, 1 = in
                                                           address String,
                                                           token_address String,
                                                           delta Int256,
                                                           block_number UInt64,
                                                           inserted_at DateTime DEFAULT now()
    ) ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY (tx_hash, log_index, direction);

---------------------------------------------------------
-- MV: DELTA FROM
---------------------------------------------------------
CREATE MATERIALIZED VIEW IF NOT EXISTS tron_db.mv_token_delta_from
TO tron_db.address_token_delta
AS
SELECT
    tx_hash,
    log_index,
    0 AS direction,
    from_addr AS address,
    token_address,
    -toInt256(amount) AS delta,
    block_number
FROM tron_db.token_transfers
WHERE from_addr != 'T9yD14Nj9j7xAB4dbGeiX9h8unkKHxuWwb'; -- zero address

---------------------------------------------------------
-- MV: DELTA TO
---------------------------------------------------------
CREATE MATERIALIZED VIEW IF NOT EXISTS tron_db.mv_token_delta_to
TO tron_db.address_token_delta
AS
SELECT
    tx_hash,
    log_index,
    1 AS direction,
    to_addr AS address,
    token_address,
    toInt256(amount) AS delta,
    block_number
FROM tron_db.token_transfers
WHERE to_addr != 'T9yD14Nj9j7xAB4dbGeiX9h8unkKHxuWwb';

---------------------------------------------------------
-- FINAL TOKEN BALANCE
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.address_token_balance (
                                                             address String,
                                                             token_address String,
                                                             balance Int256
) ENGINE = SummingMergeTree()
    ORDER BY (address, token_address);

---------------------------------------------------------
-- MV: FINAL BALANCE
---------------------------------------------------------
CREATE MATERIALIZED VIEW IF NOT EXISTS tron_db.mv_token_balance
TO tron_db.address_token_balance
AS
SELECT
    address,
    token_address,
    delta AS balance
FROM tron_db.address_token_delta;

---------------------------------------------------------
-- TOKEN METADATA (TRC20)
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.token_metadata (
                                                      token_address String,
                                                      name String,
                                                      symbol String,
                                                      decimals UInt8,
                                                      total_supply String,
                                                      is_verified UInt8,
                                                      created_at DateTime DEFAULT now(),
    updated_at DateTime DEFAULT now()
    ) ENGINE = ReplacingMergeTree(updated_at)
    ORDER BY token_address;

---------------------------------------------------------
-- CONTRACT METADATA (TRON SPECIFIC 🔥)
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.contract_metadata (
                                                         contract_address String,
                                                         contract_type String,       -- TRC20 / DEX / BRIDGE / UNKNOWN
                                                         creator_address String,
                                                         created_at_block UInt64,
                                                         created_at DateTime DEFAULT now()
    ) ENGINE = ReplacingMergeTree(created_at)
    ORDER BY contract_address;

---------------------------------------------------------
-- SYNC STATE
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.sync_state (
                                                  chain String,
                                                  last_synced_block UInt64,
                                                  updated_at DateTime DEFAULT now()
    ) ENGINE = ReplacingMergeTree(updated_at)
    ORDER BY chain;

---------------------------------------------------------
-- AML FEATURES TABLE (VERY IMPORTANT 🔥)
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.transaction_features (
                                                            tx_hash String,
                                                            block_number UInt64,
                                                            is_swap UInt8,
                                                            is_bridge UInt8,
                                                            is_contract_call UInt8,
                                                            unique_tokens UInt16,
                                                            participants UInt16,
                                                            inserted_at DateTime DEFAULT now()
    ) ENGINE = ReplacingMergeTree(inserted_at)
    ORDER BY (block_number, tx_hash);

---------------------------------------------------------
-- TRANSACTION RISK
---------------------------------------------------------
CREATE TABLE IF NOT EXISTS tron_db.transaction_risk (
                                                        tx_hash String,
                                                        block_number UInt64,

                                                        risk_score UInt8,         -- 0 → 100
                                                        risk_level String,        -- LOW / MEDIUM / HIGH

                                                        is_swap UInt8,
                                                        is_bridge UInt8,
                                                        is_contract_call UInt8,

                                                        unique_tokens UInt16,
                                                        participants UInt16,

                                                        created_at DateTime DEFAULT now()
    ) ENGINE = ReplacingMergeTree(created_at)
    ORDER BY (block_number, tx_hash);