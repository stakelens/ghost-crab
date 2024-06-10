-- Your SQL goes here
DROP TABLE IF EXISTS "tvl";
CREATE TABLE "rocketpool_tvl"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"blocknumber" INT8 NOT NULL,
	"eth" TEXT NOT NULL,
	"rpl" TEXT NOT NULL
);

CREATE TABLE "etherfi_tvl"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"blocknumber" INT8 NOT NULL,
	"eth" TEXT NOT NULL
);

