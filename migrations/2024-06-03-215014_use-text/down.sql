-- This file should undo anything in `up.sql`
ALTER TABLE "tvl" DROP COLUMN "eth";
ALTER TABLE "tvl" DROP COLUMN "rpl";
ALTER TABLE "tvl" ADD COLUMN "eth" INT8 NOT NULL;
ALTER TABLE "tvl" ADD COLUMN "rpl" INT8 NOT NULL;

