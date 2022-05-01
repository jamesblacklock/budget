CREATE TABLE accounts (
    id                  INTEGER NOT NULL
                                PRIMARY KEY AUTOINCREMENT,
    name                TEXT    NOT NULL
                                UNIQUE,
    is_tracking_account BOOLEAN NOT NULL,
    balance             INTEGER NOT NULL
);

CREATE TABLE budgets (
    id          INTEGER NOT NULL
                        PRIMARY KEY AUTOINCREMENT,
    month       INTEGER NOT NULL,
    year        INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    assigned    INTEGER NOT NULL,
    activity    INTEGER NOT NULL,
    available   INTEGER NOT NULL,
    FOREIGN KEY (
        category_id
    )
    REFERENCES categories (id) 
);

CREATE TABLE categories (
    id       INTEGER NOT NULL
                     PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    name     TEXT    NOT NULL
                     UNIQUE,
    "order"  INTEGER NOT NULL
);

CREATE TABLE payees (
    id   INTEGER NOT NULL
                 PRIMARY KEY AUTOINCREMENT,
    name TEXT    NOT NULL
                 UNIQUE
);

CREATE TABLE txs (
    id                  INTEGER NOT NULL
                                PRIMARY KEY AUTOINCREMENT,
    timestamp           TIMESTAMP NOT NULL,
    month               INTEGER NOT NULL,
    year                INTEGER NOT NULL,
    account_id          INTEGER NOT NULL,
    payee_id            INTEGER,
    transfer_account_id INTEGER,
    category_id         INTEGER,
    memo                TEXT    NOT NULL,
    amount              INTEGER NOT NULL,
    cleared             BOOLEAN NOT NULL,
    FOREIGN KEY (
        account_id
    )
    REFERENCES accounts (id),
    FOREIGN KEY (
        category_id
    )
    REFERENCES categories (id),
    FOREIGN KEY (
        payee_id
    )
    REFERENCES payees (id) 
);

INSERT INTO payees (name) VALUES ('Starting Balance');
INSERT INTO categories (name, group_id, "order") VALUES ('Unassigned', 0, 0);

