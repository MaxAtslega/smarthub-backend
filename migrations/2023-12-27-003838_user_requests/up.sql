CREATE TABLE user_requests (
    id INTEGER NOT NULL PRIMARY KEY,
    user_id INT NOT NULL,
    name VARCHAR NOT NULL,
    endpoint VARCHAR NOT NULL,
    parameters TEXT NOT NULL,
    created_on TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user_users (id)
);