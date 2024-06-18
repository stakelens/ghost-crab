use crate::models::EtherfiTVL;
use crate::models::RocketPoolTVL;
use crate::models::StakewiseTVL;
use crate::models::SwellTVL;
use crate::schema::{cache, etherfi_tvl, rocketpool_tvl, stakewise_tvl, swell_tvl};
use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn establish_connection(database_url: String) -> PgConnection {
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(Insertable)]
#[diesel(table_name = cache)]
pub struct AddCache {
    pub id: String,
    pub data: String,
}

pub fn add_cache(conn: &mut PgConnection, value: AddCache) {
    diesel::insert_into(cache::table)
        .values(&value)
        .execute(conn)
        .expect("Error saving cache");
}

pub fn get_cache(conn: &mut PgConnection, id: &str) -> Option<String> {
    cache::table
        .find(id)
        .first::<(String, String)>(conn)
        .map(|(_, data)| data)
        .ok()
}

#[derive(Insertable)]
#[diesel(table_name = rocketpool_tvl)]
pub struct AddRocketPoolTVL {
    pub eth: String,
    pub rpl: String,
    pub blocknumber: i64,
}

pub fn add_rocketpool_tvl(conn: &mut PgConnection, value: AddRocketPoolTVL) -> RocketPoolTVL {
    diesel::insert_into(rocketpool_tvl::table)
        .values(&value)
        .returning(RocketPoolTVL::as_returning())
        .get_result(conn)
        .expect("Error saving RocketPoolTVL")
}

#[derive(Insertable)]
#[diesel(table_name = etherfi_tvl)]
pub struct AddEtherfiTVL {
    pub eth: String,
    pub blocknumber: i64,
}

pub fn add_etherfi_tvl(conn: &mut PgConnection, value: AddEtherfiTVL) -> EtherfiTVL {
    diesel::insert_into(etherfi_tvl::table)
        .values(&value)
        .returning(EtherfiTVL::as_returning())
        .get_result(conn)
        .expect("Error saving EtherfiTVL")
}

#[derive(Insertable)]
#[diesel(table_name = swell_tvl)]
pub struct AddSwellTVL {
    pub eth: String,
    pub blocknumber: i64,
}

pub fn add_swell_tvl(conn: &mut PgConnection, value: AddSwellTVL) -> SwellTVL {
    diesel::insert_into(swell_tvl::table)
        .values(&value)
        .returning(SwellTVL::as_returning())
        .get_result(conn)
        .expect("Error saving SwellTVL")
}

#[derive(Insertable)]
#[diesel(table_name = stakewise_tvl)]
pub struct AddStakewiseTVL {
    pub eth: String,
    pub blocknumber: i64,
    pub rewards: String,
}

pub fn add_stakewise_tvl(conn: &mut PgConnection, value: AddStakewiseTVL) -> StakewiseTVL {
    diesel::insert_into(stakewise_tvl::table)
        .values(&value)
        .returning(StakewiseTVL::as_returning())
        .get_result(conn)
        .expect("Error saving StakewiseTVL")
}
