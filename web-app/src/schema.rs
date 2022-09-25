// @generated automatically by Diesel CLI.

diesel::table! {
    comments (id) {
        id -> Int4,
        comment -> Varchar,
        threadid -> Int4,
        userid -> Int4,
        parentcommentid -> Nullable<Int4>,
        createdat -> Timestamp,
    }
}

diesel::table! {
    threads (id) {
        id -> Int4,
        title -> Varchar,
        link -> Nullable<Varchar>,
        authorid -> Int4,
        createdat -> Timestamp,
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

diesel::joinable!(comments -> threads (threadid));
diesel::joinable!(comments -> users (userid));
diesel::joinable!(threads -> users (authorid));

diesel::allow_tables_to_appear_in_same_query!(
    comments,
    threads,
    users,
);
