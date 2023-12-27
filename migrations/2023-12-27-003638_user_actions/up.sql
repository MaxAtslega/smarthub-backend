CREATE TABLE user_actions (
    id INTEGER NOT NULL PRIMARY KEY,
    user_id INT NOT NULL,
    type_name VARCHAR NOT NULL,
    details TEXT NOT NULL,
    created_on TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user_users (id)
);