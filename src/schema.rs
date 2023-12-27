// @generated automatically by Diesel CLI.

diesel::table! {
    user_actions (id) {
        id -> Integer,
        user_id -> Integer,
        type_name -> Text,
        details -> Text,
        created_on -> Timestamp,
    }
}

diesel::table! {
    user_requests (id) {
        id -> Integer,
        action_id -> Integer,
        endpoint -> Text,
        parameters -> Text,
        created_on -> Timestamp,
    }
}

diesel::table! {
    user_rfid (id) {
        id -> Integer,
        rfid_uid -> Text,
        action_id -> Integer,
        created_on -> Timestamp,
    }
}

diesel::table! {
    user_users (id) {
        id -> Integer,
        username -> Text,
        theme -> Integer,
        language -> Text,
        created_on -> Timestamp,
    }
}

diesel::joinable!(user_actions -> user_users (user_id));
diesel::joinable!(user_requests -> user_actions (action_id));
diesel::joinable!(user_rfid -> user_actions (action_id));

diesel::allow_tables_to_appear_in_same_query!(
    user_actions,
    user_requests,
    user_rfid,
    user_users,
);
