use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::rocketpool_tvl)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RocketPoolTVL {
    pub id: i32,
    pub eth: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::etherfi_tvl)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EtherfiTVL {
    pub id: i32,
    pub eth: String,
}
