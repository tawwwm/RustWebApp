-- Your SQL goes here
CREATE TABLE users (
	id SERIAL PRIMARY KEY,
	username VARCHAR NOT NULL,
	email VARCHAR NOT NULL,
	password VARCHAR NOT NULL,

	UNIQUE(username),
	UNIQUE(email)
);

CREATE TABLE threads (
	id SERIAL PRIMARY KEY,
	title VARCHAR NOT NULL,
	link VARCHAR,
	author_id INT NOT NULL,
	created_at TIMESTAMP NOT NULL,

	CONSTRAINT fk_author
		FOREIGN KEY(author_id)
			REFERENCES users(id)
);

CREATE TABLE comments (
	id SERIAL PRIMARY KEY,
	content VARCHAR NOT NULL,
	thread_id INT NOT NULL,
	author_id INT NOT NULL,
	parent_comment_id INT,
	created_at TIMESTAMP NOT NULL,

	CONSTRAINT fk_thread
		FOREIGN KEY(thread_id)
			REFERENCES threads(id),

	CONSTRAINT fk_user
		FOREIGN KEY(author_id)
			REFERENCES users(id),

	CONSTRAINT fk_parent_comment
		FOREIGN KEY(parent_comment_id)
			REFERENCES comments(id)

);