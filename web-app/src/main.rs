#[macro_use]
extern crate diesel;
pub mod schema;
pub mod models;

// IMPORTS ###
use actix_web::{HttpServer, App, web, HttpResponse, Responder};
use actix_identity::{Identity, CookieIdentityPolicy, IdentityService};
use tera::{Tera, Context};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::{r2d2::ConnectionManager};
use dotenv::dotenv;
use argonautica::Verifier;
use actix_web::middleware::Logger;


// FILE IMPORTS #
use models::{User, NewUser, LoginUser, Thread, NewThread, Comment, NewComment};

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;



// STRUCTS ###
#[derive(Deserialize, Debug)]
struct PostAttempt {
    title: String,
    link: String
}

#[derive(Deserialize, Debug)]
struct LoginAttempt{
    username: String,
    password: String
}

#[derive(Deserialize, Debug)]
struct PostForm{
    title: String,
    link: String,
}

#[derive(Deserialize, Debug)]
struct CommentForm{
    content: String,
}

// ERROR HANDLING

#[derive(Debug)]
enum ServerError{
    ArgonauticError,
    DieselError,
    EnvironmentError,
    R2D2Error,
    TeraError,
    UserError(String)
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        write!(f, "Test")
    }
}

impl actix_web::error::ResponseError for ServerError{
    fn error_response(&self) -> HttpResponse{
        match self{
            ServerError::ArgonauticError => HttpResponse::InternalServerError().json("Argonautica Error."),
            ServerError::DieselError => HttpResponse::InternalServerError().json("Diesel Error."),
            ServerError::R2D2Error => HttpResponse::InternalServerError().json("R2D2 Error."),
            ServerError::EnvironmentError => HttpResponse::InternalServerError().json("Environment Error."),
            ServerError::TeraError => HttpResponse::InternalServerError().json("Tera Error"),
            ServerError::UserError(data) => HttpResponse::InternalServerError().json(data),

        }
    }
}

impl From<std::env::VarError> for ServerError {
    fn from(_: std::env::VarError) -> ServerError{
        log::error!("{:?}", ServerError::EnvironmentError);
        ServerError::EnvironmentError
    }
}

impl From<r2d2::Error> for ServerError{
    fn from(_: r2d2::Error) -> ServerError{
        log::error!("{:?}", ServerError::R2D2Error);
        ServerError::R2D2Error
    }
}

impl From<diesel::result::Error> for ServerError{
    fn from(err: diesel::result::Error) -> ServerError{
        match err {
            diesel::result::Error::NotFound => {
                log::error!("{:?}", err);
                ServerError::UserError("Username not found".to_string())
            }, 
            _ => ServerError::DieselError
        }
    }
}

impl From<argonautica::Error> for ServerError{
    fn from(_: argonautica::Error) -> ServerError{
        log::error!("{:?}", ServerError::ArgonauticError);
        ServerError::ArgonauticError
    }
}

impl From<tera::Error> for ServerError{
    fn from(_: tera::Error) -> ServerError{
        log::error!("{:?}", ServerError::TeraError);
        ServerError::TeraError
    }
}




//FUNCTIONS###

async fn index(tera: web::Data<Tera>, pool: web::Data<Pool>) -> Result<HttpResponse, ServerError>{
    use schema::threads::dsl::{threads};
    use schema::users::dsl::{users};

    let mut connection = pool.get()?;
    let thread_list :Vec<(Thread, User)> = threads.inner_join(users)
        .load(&mut connection)?;
    
    let mut data = Context::new();
    data.insert("title", "Web App");
    data.insert("threadsInfo", &thread_list);

    let rendered = tera.render("index.html", &data)?;
    Ok(HttpResponse::Ok().body(rendered))
}

//Registration page
async fn register(tera: web::Data<Tera>) -> Result<HttpResponse, ServerError>{
    let mut data = Context::new();

    data.insert("title", "Sign Up");
    let rendered = tera.render("register.html", &data)?;
    Ok(HttpResponse::Ok().body(rendered))
}

//Registration form
async fn register_user(data: web::Form<NewUser>, pool: web::Data<Pool>) -> Result<HttpResponse, ServerError>{
    use schema::users;

    let mut connection = pool.get()?;
    
    let new_user = NewUser::new(data.username.clone(), data.email.clone(), data.password.clone());

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut connection)?;

    println!("{:?}", data);
    Ok(HttpResponse::Ok().body(format!("Succesfully registered user: {}", data.username)))
}

//Login page
async fn login(tera: web::Data<Tera>, id: Identity) -> Result<HttpResponse, ServerError>{
    let mut data = Context::new();
    data.insert("title", "Login");

    if let Some(_id) = id.identity(){
        return Ok(HttpResponse::Ok().body("You are already logged in."))
    }

    let rendered = tera.render("login.html", &data)?;
    Ok(HttpResponse::Ok().body(rendered))
}

//Login form
async fn login_user(data: web::Form<LoginUser>, id: Identity, pool: web::Data<Pool>) -> Result<HttpResponse, ServerError>{
    use schema::users::dsl::{username,users};

    let mut connection = pool.get()?;
    let user = users.filter(username.eq(&data.username)).first::<User>(&mut connection)?;
    dotenv().ok();
            
    let secret = std::env::var("SECRET_KEY")?;
            
    let valid = Verifier::default()
        .with_hash(user.password)
        .with_password(data.password.clone())
        .with_secret_key(secret)
        .verify()?;
            
    if valid {
        let session_token = String::from(user.username);
        id.remember(session_token);
        Ok(HttpResponse::Ok().body(format!("Succesfully logged in as: {}", data.username)))
    }else{
        Ok(HttpResponse::Ok().body("Password is incorrect."))
    }    
    
}

async fn logout_user(id: Identity) -> Result<HttpResponse, ServerError>{
    id.forget();
    Ok(HttpResponse::Ok().body("Logged user out."))
}

//Threads page
async fn post(tera: web::Data<Tera>, id: Identity) -> Result<HttpResponse, ServerError>{
    let mut data = Context::new();

    data.insert("title", "Post Thread");

    if let Some(_id) = id.identity(){
        let rendered = tera.render("post.html", &data)?;
        return Ok(HttpResponse::Ok().body(rendered))
    }

    Ok(HttpResponse::Ok().body("User not logged in."))
}

//Thread submission form
async fn post_thread(data: web::Form<PostForm>, id: Identity, pool: web::Data<Pool>) -> Result<HttpResponse, ServerError> {

    if let Some(id) = id.identity(){
        use schema::users::dsl::{username, users};

        let mut connection = pool.get()?;
        let user: Result<User, diesel::result::Error> = users.filter(username.eq(id)).first(&mut connection);

        match user{
            Ok(u) => {
                let new_thread = NewThread::from_post_form(data.title.clone(), data.link.clone(), u.id);
                use schema::threads;

                diesel::insert_into(threads::table)
                    .values(&new_thread)
                    .get_result::<Thread>(&mut connection)?;

                    return Ok(HttpResponse::Ok().body("Posted thread!"));
            }
            Err(e) => {
                println!("{:?}", e);
                return Ok(HttpResponse::Ok().body("Failed to find user."))
            }
        }

    }
    Ok(HttpResponse::Unauthorized().body("User not logged in."))
}

// Thread specific page
async fn thread_page(tera: web::Data<Tera>, id: Identity, web::Path(thread_id): web::Path<i32>, pool: web::Data<Pool>) -> Result<HttpResponse, ServerError>{

    use schema::threads::dsl::{threads};
    use schema::users::dsl::{users};
    //use schema::comments::dsl::{comments};

    let mut connection = pool.get()?;

    let thread :Thread = threads.find(thread_id)
        .get_result(&mut connection)?;
    
    let user :User = users.find(thread.author_id)
        .get_result(&mut connection)?;
    
    let comments :Vec<(Comment, User)> = Comment::belonging_to(&thread)
        .inner_join(users)
        .load(&mut connection)?;

    let mut data = Context::new();
    data.insert("title",&format!("{}", thread.title));
    data.insert("thread", &thread);
    data.insert("user", &user);
    data.insert("comments", &comments);

    if let Some(_id) = id.identity() {
        data.insert("logged_in", "true");
    }else{
        data.insert("logged_in", "false");
    }

    let rendered = tera.render("thread.html", &data)?;
    Ok(HttpResponse::Ok().body(rendered))    
}

async fn comment(data: web::Form<CommentForm>, id:Identity, web::Path(thread_id): web::Path<i32>, pool: web::Data<Pool>) -> Result<HttpResponse, ServerError>{

    if let Some(id) = id.identity(){
        use schema::threads::dsl::{threads};
        use schema::users::dsl::{users, username};

        let mut connection = pool.get()?;

        let thread :Thread = threads.find(thread_id)
            .get_result(&mut connection)?;
        
        let user :Result<User, diesel::result::Error> = users
            .filter(username.eq(id))
            .first(&mut connection);

        match user{
            Ok(u) => {
                let parent_id = None;
                let new_comment = NewComment::new(data.content.clone(), thread.id, u.id, parent_id);

                use schema::comments;
                diesel::insert_into(comments::table)
                    .values(&new_comment)
                    .get_result::<Comment>(&mut connection)?;

                return Ok(HttpResponse::Ok().body("Comment posted."))
            }
            Err(e) => {
                println!("{:?}", e);
                return Ok(HttpResponse::Ok().body("User not found."));
            }

        }
    }

    Ok(HttpResponse::Unauthorized().body("Not logged in."))

}



#[actix_web::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();
    env_logger::init();
    let database_url = std::env::var("DATABASE_URL")
        .expect("Database URL not set.");
    
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder().build(manager)
        .expect("Failed to create Postgres pool.");
    
    
    HttpServer::new(move|| {
        let tera = Tera::new("templates/**/*").unwrap();
        App::new()
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                .name("auth-cookie")
                .secure(false)
            ))
            .data(tera)
            .data(pool.clone())
            .route("/", web::get().to(index))
            .route("/register", web::get().to(register))
            .route("/register", web::post().to(register_user))
            .route("/login", web::get().to(login))
            .route("/login", web::post().to(login_user))
            .route("/logout", web::to(logout_user))
            .route("/post", web::get().to(post))
            .route("/post", web::post().to(post_thread))
            .service(
                web::resource("/thread/{thread_id}")
                    .route(web::get().to(thread_page))
                    .route(web::post().to(comment))
            )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}