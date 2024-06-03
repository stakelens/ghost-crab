// @generated automatically by Diesel CLI.

diesel::table! {
    tvl (id) {
        id -> Int4,
        blocknumber -> Int8,
        eth -> Text,
        rpl -> Text,
    }
}
