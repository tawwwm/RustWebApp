use super::schema::{users, threads, comments};
use diesel::{Queryable, Insertable, Identifiable, Associations};
use serde::{Serialize, Deserialize};
use dotenv::dotenv;
use argonautica::Hasher;

#[derive(Serialize, Debug, Identifiable, Queryable)]
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

impl NewUser{
	pub fn new(username: String, email: String, password:String) -> Self{
		dotenv().ok();
		
		let secret = std::env::var("SECRET_KEY")
			.expect("SECRET_KEY must be set");
		
		let hash = Hasher::default()
			.with_password(password)
			.with_secret_key(secret)
			.hash()
			.unwrap();

		NewUser{
			username: username,
			email: email,
			password: hash,
		}

	}
}

#[derive(Debug, Deserialize)]
pub struct LoginUser{
	pub username: String,
	pub password: String,
}

#[derive(Serialize, Debug, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(User))]
pub struct Thread{
	pub id: i32,
	pub title: String,
	pub link: Option<String>,
	pub user_id: i32,
	pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Insertable)]
#[diesel(table_name= threads)]
pub struct NewThread{
	pub title: String,
	pub link: String,
	pub user_id: i32,
	pub created_at: chrono::NaiveDateTime,
}

impl NewThread{
	pub fn from_post_form(title: String, link: String, uid: i32) -> Self{
		NewThread{
			title: title,
			link: link,
			user_id: uid,
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
	pub user_id: i32,
	pub parent_comment_id: Option<i32>,
	pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Insertable)]
#[diesel(table_name= comments)]
pub struct NewComment{
	pub content: String,
	pub thread_id: i32,
	pub user_id: i32,
	pub parent_comment_id: Option<i32>,
	pub created_at: chrono::NaiveDateTime,
}

impl NewComment{
	pub fn new(content: String, thread_id: i32, user_id: i32, parent_comment_id: Option<i32>) -> Self{
		NewComment{
			content: content,
			thread_id: thread_id,
			user_id: user_id,
			parent_comment_id: parent_comment_id,
			created_at: chrono::Local::now().naive_utc(),
		}
	}


}