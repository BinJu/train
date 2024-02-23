// @generated automatically by Diesel CLI.

diesel::table! {
    account (id) {
        id -> Int4,
        name -> Text,
        total -> Int4,
        in_stock -> Int4,
        data -> Text,
        owner -> Nullable<Int4>,
        desp -> Nullable<Text>,
    }
}

diesel::table! {
    acnt_ctl (id) {
        id -> Int4,
        account_id -> Nullable<Int4>,
        team_id -> Nullable<Int4>,
    }
}

diesel::table! {
    artifact (id) {
        id -> Int4,
        name -> Text,
        total -> Int4,
        target -> Int4,
        team_id -> Int4,
        build -> Json,
        clean -> Nullable<Json>,
    }
}

diesel::table! {
    sec_ctl (id) {
        id -> Int4,
        secret_id -> Nullable<Int4>,
        team_id -> Nullable<Int4>,
    }
}

diesel::table! {
    secret (id) {
        id -> Int4,
        name -> Text,
        data -> Text,
        owner -> Nullable<Int4>,
        desp -> Nullable<Text>,
    }
}

diesel::table! {
    team (id) {
        id -> Int4,
        name -> Text,
        token -> Text,
        desp -> Nullable<Text>,
    }
}

diesel::joinable!(account -> team (owner));
diesel::joinable!(acnt_ctl -> account (account_id));
diesel::joinable!(acnt_ctl -> team (team_id));
diesel::joinable!(artifact -> team (team_id));
diesel::joinable!(sec_ctl -> secret (secret_id));
diesel::joinable!(sec_ctl -> team (team_id));
diesel::joinable!(secret -> team (owner));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    acnt_ctl,
    artifact,
    sec_ctl,
    secret,
    team,
);
