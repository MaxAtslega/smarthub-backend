// @generated automatically by Diesel CLI.

diesel::table! {
    constants (id) {
        id -> Integer,
        name -> Text,
        user_id -> Integer,
        value -> Text,
    }
}

diesel::table! {
    user_actions (id) {
        id -> Integer,
        user_id -> Integer,
        rfid_uid -> Text,
        type_name -> Text,
        details -> Text,
        created_on -> Timestamp,
    }
}

diesel::table! {
    user_requests (id) {
        id -> Integer,
        user_id -> Integer,
        name -> Text,
        endpoint -> Text,
        parameters -> Text,
        created_on -> Timestamp,
    }
}

diesel::table! {
    user_users (id) {
        id -> Integer,
        username -> Text,
        theme -> Integer,
        birthday -> Date,
        language -> Text,
        keyboard -> Text,
        created_on -> Timestamp,
    }
}

diesel::joinable!(constants -> user_users (user_id));
diesel::joinable!(user_actions -> user_users (user_id));
diesel::joinable!(user_requests -> user_users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    constants,
    user_actions,
    user_requests,
    user_users,
);
