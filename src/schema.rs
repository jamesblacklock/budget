table! {
    accounts (id) {
        id -> Integer,
        name -> Text,
        is_tracking_account -> Bool,
        balance -> Integer,
    }
}

table! {
    budgets (id) {
        id -> Integer,
        month -> Integer,
        year -> Integer,
        category_id -> Integer,
        assigned -> Integer,
        activity -> Integer,
        available -> Integer,
    }
}

table! {
    categories (id) {
        id -> Integer,
        group_id -> Integer,
        name -> Text,
        order -> Integer,
    }
}

table! {
    payees (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    txs (id) {
        id -> Integer,
        timestamp -> Timestamp,
        month -> Integer,
        year -> Integer,
        account_id -> Integer,
        payee_id -> Nullable<Integer>,
        transfer_account_id -> Nullable<Integer>,
        category_id -> Nullable<Integer>,
        memo -> Text,
        amount -> Integer,
        cleared -> Bool,
    }
}

joinable!(budgets -> categories (category_id));
joinable!(txs -> accounts (account_id));
joinable!(txs -> categories (category_id));
joinable!(txs -> payees (payee_id));

allow_tables_to_appear_in_same_query!(
    accounts,
    budgets,
    categories,
    payees,
    txs,
);
