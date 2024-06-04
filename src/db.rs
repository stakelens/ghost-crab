use crate::models::TVL;
use crate::schema::tvl;
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
        .expect("Error saving new post")
}
