// @generated automatically by Diesel CLI.

diesel::table! {
    tvl (id) {
        id -> Int4,
        eth -> Int8,
        rpl -> Int8,
        blocknumber -> Int8,
    }
}
