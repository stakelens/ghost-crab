// @generated automatically by Diesel CLI.

diesel::table! {
    cache (id) {
        id -> Text,
        data -> Text,
    }
}

diesel::table! {
    etherfi_tvl (id) {
        id -> Int4,
        blocknumber -> Int8,
        eth -> Text,
    }
}

diesel::table! {
    rocketpool_tvl (id) {
        id -> Int4,
        blocknumber -> Int8,
        eth -> Text,
        rpl -> Text,
    }
}

diesel::table! {
    swell_tvl (id) {
        id -> Int4,
        blocknumber -> Int8,
        eth -> Text,
    }
}

diesel::table! {
    stakewise_tvl (id) {
        id -> Int4,
        blocknumber -> Int8,
        eth -> Text,
        rewards -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(cache, etherfi_tvl, rocketpool_tvl, swell_tvl,);
