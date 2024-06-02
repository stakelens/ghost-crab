use crate::models::TVL;
use crate::schema::tvl;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(Insertable)]
#[diesel(table_name = tvl)]
pub struct AddTvl {
    pub eth: i64,
    pub rpl: i64,
    pub blocknumber: i64,
}

pub fn add_tvl(conn: &mut PgConnection, value: AddTvl) -> TVL {
    diesel::insert_into(tvl::table)
        .values(&value)
        .returning(TVL::as_returning())
        .get_result(conn)
        .expect("Error saving new post")
}
