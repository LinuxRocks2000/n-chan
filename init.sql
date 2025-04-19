CREATE TABLE boards (name TEXT, desc TEXT, id INT);

CREATE TABLE users (name TEXT, passwd TEXT, rights INT);

CREATE TABLE posts (username TEXT, content TEXT, board INT, image TEXT, id INTEGER PRIMARY KEY, time INT);

CREATE TABLE replies (username TEXT, content TEXT, post INT, image TEXT, time INT);