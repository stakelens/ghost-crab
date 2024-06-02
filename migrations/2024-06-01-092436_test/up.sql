-- Your SQL goes here
CREATE TABLE tvl (
    id SERIAL PRIMARY KEY,
    eth BIGINT NOT NULL,
    rpl BIGINT NOT NULL,
    blocknumber BIGINT NOT NULL
)
