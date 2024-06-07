use crate::models::TVL;
use crate::schema::{cache, tvl};
use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn establish_connection(database_url: String) -> PgConnection {
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(Insertable)]
#[diesel(table_name = tvl)]
pub struct AddTvl {
    pub eth: String,
    pub rpl: String,
    pub blocknumber: i64,
}

pub fn add_tvl(conn: &mut PgConnection, value: AddTvl) -> TVL {
    diesel::insert_into(tvl::table)
        .values(&value)
        .returning(TVL::as_returning())
        .get_result(conn)
        .expect("Error saving TVL")
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
