CREATE TABLE user_rfid (
    id INTEGER PRIMARY KEY,
    rfid_uid VARCHAR NOT NULL,
    action_id INT NOT NULL,
    created_on TIMESTAMP NOT NULL,
    FOREIGN KEY (action_id) REFERENCES user_actions (id)
);
