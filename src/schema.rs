// @generated automatically by Diesel CLI.

diesel::table! {
    cache (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    tvl (id) {
        id -> Int4,
        blocknumber -> Int8,
        eth -> Text,
        rpl -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cache,
    tvl,
);
