#[macro_use]
extern crate diesel;
pub mod schema;
pub mod models;

// IMPORTS ###
use actix_web::{HttpServer, App, web, HttpResponse, Responder};
use tera::{Tera, Context};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;

// FILE IMPORTS #
use models::{User, NewUser};


// STRUCTS ###
#[derive(Serialize)]
struct Thread {
    title: String,
    link: String,
    author: String
}

#[derive(Deserialize, Debug)]
struct PostAttempt {
    title: String,
    link: String
}

/*#[derive(Deserialize, Debug)]
struct User{
    username: String,
    email: String,
    password: String
}*/

#[derive(Deserialize, Debug)]
struct LoginAttempt{
    username: String,
    password: String
}

//FUNCTIONS###
fn establish_connection() -> PgConnection{
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}



async fn index(tera: web::Data<Tera>) -> impl Responder{
    let mut data = Context::new();
    
    let posts = [
        Thread{
            title: String::from("Test Post"),
            link: String::from("https://tera.netlify.app/docs/"),
            author: String::from("Tawm")
        }
        
    ];

    data.insert("title", "Web App");
    data.insert("posts", &posts);

    let rendered = tera.render("index.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

//Registration page
async fn register(tera: web::Data<Tera>) -> impl Responder{
    let mut data = Context::new();

    data.insert("title", "Sign Up");
    let rendered = tera.render("register.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

//Registration form
async fn register_user(data: web::Form<NewUser>) -> impl Responder{
    use schema::users;

    let connection = establish_connection();

    diesel::insert_into(users::table)
        .values(&*data)
        .get_result::<User>(&connection)
        .expect("Error registering user.");

    println!("{:?}", data);
    HttpResponse::Ok().body(format!("Succesfully registered user: {}", data.username))
}

//Login page
async fn login(tera: web::Data<Tera>) -> impl Responder{
    let mut data = Context::new();

    data.insert("title", "Login");
    let rendered = tera.render("login.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

//Login form
async fn login_user(data: web::Form<LoginAttempt>) -> impl Responder{
    println!("{:?}", data);
    HttpResponse::Ok().body(format!("Succesfully logged in as: {}", data.username))
}

//Threads page
async fn thread(tera: web::Data<Tera>) -> impl Responder{
    let mut data = Context::new();

    data.insert("title", "Post Thread");
    let rendered = tera.render("post.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

//Thread submission form
async fn post_thread(data: web::Form<PostAttempt>) -> impl Responder {
    println!("{:?}", data);
    HttpResponse::Ok().body(format!("Succesfully Posted Thread: {}", data.title))
}


#[actix_web::main]
async fn main() -> std::io::Result<()>{
    HttpServer::new(|| {
        let tera = Tera::new("templates/**/*").unwrap();
        App::new()
            .data(tera)
            .route("/", web::get().to(index))
            .route("/register", web::get().to(register))
            .route("/register", web::post().to(register_user))
            .route("/login", web::get().to(login))
            .route("/login", web::post().to(login_user))
            .route("/post", web::get().to(thread))
            .route("/post", web::post().to(post_thread))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}