CREATE TABLE constants (
    id INTEGER NOT NULL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    user_id INT NOT NULL,
    value TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user_users (id)
);
