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
	authorID INT NOT NULL,
	createdAt TIMESTAMP NOT NULL,

	CONSTRAINT fk_author
		FOREIGN KEY(authorID)
			REFERENCES users(id)
);

CREATE TABLE comments (
	id SERIAL PRIMARY KEY,
	comment VARCHAR NOT NULL,
	threadID INT NOT NULL,
	userID INT NOT NULL,
	parentCommentID INT,
	createdAt TIMESTAMP NOT NULL,

	CONSTRAINT fk_thread
		FOREIGN KEY(threadID)
			REFERENCES threads(id),

	CONSTRAINT fk_user
		FOREIGN KEY(userID)
			REFERENCES users(id),

	CONSTRAINT fk_parentComment
		FOREIGN KEY(parentCommentID)
			REFERENCES comments(id)

);