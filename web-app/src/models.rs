use super::schema::{users, threads, comments};
use diesel::{Queryable, Insertable};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Debug, Queryable)]
pub struct User{
	pub id: i32,
	pub username: String,
	pub email: String,
	pub password: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name= users)]
pub struct NewUser{
	pub username: String,
	pub email: String,
	pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser{
	pub username: String,
	pub password: String,
}

#[derive(Serialize, Debug, Queryable, Identifiable)]
pub struct Thread{
	pub id: i32,
	pub title: String,
	pub link: Option<String>,
	pub author_id: i32,
	pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Insertable)]
#[diesel(table_name= threads)]
pub struct NewThread{
	pub title: String,
	pub link: String,
	pub author_id: i32,
	pub created_at: chrono::NaiveDateTime,
}

impl NewThread{
	pub fn from_post_form(title: String, link: String, uid: i32) -> Self{
		NewThread{
			title: title,
			link: link,
			author_id: uid,
			created_at: chrono::Local::now().naive_utc(),
		}
	}
}


#[derive(Serialize, Debug, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Thread))]
pub struct Comment{
	pub id: i32,
	pub content: String,
	pub thread_id: i32,
	pub author_id: i32,
	pub parent_comment_id: Option<i32>,
	pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Insertable)]
#[diesel(table_name= comments)]
pub struct NewComment{
	pub content: String,
	pub thread_id: i32,
	pub author_id: i32,
	pub parent_comment_id: Option<i32>,
	pub created_at: chrono::NaiveDateTime,
}

impl NewComment{
	pub fn new(content: String, thread_id: i32, author_id: i32, parent_comment_id: Option<i32>) -> Self{
		NewComment{
			content: content,
			thread_id: thread_id,
			author_id: author_id,
			parent_comment_id: parent_comment_id,
			created_at: chrono::Local::now().naive_utc(),
		}
	}


}