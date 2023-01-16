CREATE DATABASE seabattle;
USE seabattle;

CREATE TABLE server (
    name VARCHAR(16) NOT NULL DEFAULT 'seabattle',
    motd VARCHAR(80),
    rules BLOB,
    PRIMARY KEY (name)
);
INSERT INTO server (name, motd) VALUES ('Server', 'Welcome, fancy a game of Sea Battle?');

CREATE TABLE user (
        name VARCHAR(8) NOT NULL,
        password_hash VARCHAR(74) NOT NULL DEFAULT '',
        display_name VARCHAR(32) NOT NULL,
        email_address VARCHAR(64) NOT NULL,
        admin BOOLEAN NOT NULL DEFAULT false,
        active BOOLEAN NOT NULL DEFAULT false,
        notify BOOLEAN NOT NULL DEFAULT true,
        verification INT UNSIGNED NOT NULL DEFAULT 0,
        new_password_hash VARCHAR(74) NOT NULL DEFAULT '',
    PRIMARY KEY (name)
);
INSERT INTO user (name, display_name, email_address, notify, password_hash) VALUES ('self','Server', 'server@server.org', false, '$2b$10$lVkR4Xbo0BiOVts1QOw0w.dqTUSBOepeZbSxMNggY9KlOg1T5MTU6');
INSERT INTO user (name, display_name, email_address, admin, active, notify, password_hash) VALUES ('admin','Administrator', 'admin@server.org', true, true, false, '$2b$10$lVkR4Xbo0BiOVts1QOw0w.dqTUSBOepeZbSxMNggY9KlOg1T5MTU6');

CREATE TABLE game (
        id INT UNSIGNED NOT NULL auto_increment,
        status TINYINT UNSIGNED NOT NULL DEFAULT 0, 
        board_size TINYINT UNSIGNED NOT NULL DEFAULT 8 CHECK (board_size >= 8 AND board_size <= 16),
        amount_of_players TINYINT UNSIGNED NOT NULL DEFAULT 1,
        placing TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        started TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        finished TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE board (
        game_id INT UNSIGNED NOT NULL,
        user_name VARCHAR(8) NOT NULL,
        player_id TINYINT UNSIGNED NOT NULL,
        status TINYINT UNSIGNED NOT NULL DEFAULT 0,
        shots_fired INT UNSIGNED DEFAULT 0,
        shots_map VARBINARY(32),
        score INT UNSIGNED DEFAULT 0,
    PRIMARY KEY (game_id, user_name),
    FOREIGN KEY (game_id) REFERENCES game(id),
    FOREIGN KEY (user_name) REFERENCES user(name)
);

CREATE TABLE ship_class (
        name VARCHAR(16) NOT NULL,
        size TINYINT UNSIGNED NOT NULL CHECK (size >= 2 AND size <= 8),
    PRIMARY KEY (name)
);

INSERT INTO ship_class (name, size) VALUES ('Carrier', 5);
INSERT INTO ship_class (name, size) VALUES ('Battleship', 4);
INSERT INTO ship_class (name, size) VALUES ('Destroyer', 3);
INSERT INTO ship_class (name, size) VALUES ('Submarine', 3);
INSERT INTO ship_class (name, size) VALUES ('Patrol Boat', 2);

CREATE TABLE ship (
        game_id INT UNSIGNED NOT NULL,
        user_name VARCHAR(8) NOT NULL,
        name VARCHAR(8) NOT NULL,
        class VARCHAR(16) NOT NULL,
        position_x TINYINT UNSIGNED NOT NULL,
        position_y TINYINT UNSIGNED NOT NULL,
        direction ENUM ('north', 'south', 'east', 'west'),
        damage TINYINT UNSIGNED NOT NULL DEFAULT 0,
    PRIMARY KEY (game_id, user_name, name),
    FOREIGN KEY (class) REFERENCES ship_class(name),
    FOREIGN KEY (game_id) REFERENCES game(id),
    FOREIGN KEY (user_name) REFERENCES user(name),
    FOREIGN KEY (game_id, user_name) REFERENCES board(game_id, user_name)
);
