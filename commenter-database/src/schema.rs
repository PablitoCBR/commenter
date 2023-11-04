// @generated automatically by Diesel CLI.

diesel::table! {
    comments (id) {
        #[max_length = 36]
        id -> Bpchar,
        #[max_length = 255]
        group_id -> Varchar,
        #[max_length = 1024]
        text -> Nullable<Varchar>,
        state -> Int4,
    }
}
