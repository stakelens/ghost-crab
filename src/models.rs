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

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::swell_tvl)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SwellTVL {
    pub id: i32,
    pub eth: String,
    pub blocknumber: i64,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::stakewise_tvl)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StakewiseTVL {
    pub id: i32,
    pub eth: String,
    pub blocknumber: i64,
    pub rewards: String,
}
