use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::tvl)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TVL {
    pub id: i32,
    pub eth: i64,
}
