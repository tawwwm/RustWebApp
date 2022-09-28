// @generated automatically by Diesel CLI.

diesel::table! {
    comments (id) {
        id -> Int4,
        content -> Varchar,
        thread_id -> Int4,
        author_id -> Int4,
        parent_comment_id -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    threads (id) {
        id -> Int4,
        title -> Varchar,
        link -> Nullable<Varchar>,
        author_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
    }
}

diesel::joinable!(comments -> threads (thread_id));
diesel::joinable!(comments -> users (author_id));
diesel::joinable!(threads -> users (author_id));

diesel::allow_tables_to_appear_in_same_query!(
    comments,
    threads,
    users,
);
