use crate::models::EtherfiTVL;
use crate::models::RocketPoolTVL;
use crate::schema::{cache, etherfi_tvl, rocketpool_tvl};
use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn establish_connection(database_url: String) -> PgConnection {
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
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
