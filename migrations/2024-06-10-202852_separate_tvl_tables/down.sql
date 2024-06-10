-- This file should undo anything in `up.sql`
CREATE TABLE "tvl"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"blocknumber" INT8 NOT NULL,
	"eth" TEXT NOT NULL,
	"rpl" TEXT NOT NULL
);

DROP TABLE IF EXISTS "rocketpool_tvl";
DROP TABLE IF EXISTS "etherfi_tvl";
