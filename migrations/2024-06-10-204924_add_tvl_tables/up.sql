-- Your SQL goes here
CREATE TABLE "etherfi_tvl"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"blocknumber" INT8 NOT NULL,
	"eth" TEXT NOT NULL
);

CREATE TABLE "rocketpool_tvl"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"blocknumber" INT8 NOT NULL,
	"eth" TEXT NOT NULL,
	"rpl" TEXT NOT NULL
);

CREATE TABLE "cache"(
	"id" TEXT NOT NULL PRIMARY KEY,
	"data" TEXT NOT NULL
);

