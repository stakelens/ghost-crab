-- Your SQL goes here
ALTER TABLE "tvl" DROP COLUMN "eth";
ALTER TABLE "tvl" DROP COLUMN "rpl";
ALTER TABLE "tvl" ADD COLUMN "eth" TEXT NOT NULL;
ALTER TABLE "tvl" ADD COLUMN "rpl" TEXT NOT NULL;

