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
use dotenv::dotenv;

// FILE IMPORTS #
use models::{User, NewUser, LoginUser, Thread, NewThread, Comment, NewComment};


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

//FUNCTIONS###
fn establish_connection() -> PgConnection{
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}



async fn index(tera: web::Data<Tera>) -> impl Responder{
    use schema::threads::dsl::{threads};
    use schema::users::dsl::{users};

    let mut connection = establish_connection();
    let thread_list :Vec<(Thread, User)> = threads.inner_join(users)
        .load(&mut connection)
        .expect("Error retrieving posts");
    
    let mut data = Context::new();
    data.insert("title", "Web App");
    data.insert("threadsInfo", &thread_list);

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

    let mut connection = establish_connection();

    diesel::insert_into(users::table)
        .values(&*data)
        .get_result::<User>(&mut connection)
        .expect("Error registering user.");

    println!("{:?}", data);
    HttpResponse::Ok().body(format!("Succesfully registered user: {}", data.username))
}

//Login page
async fn login(tera: web::Data<Tera>, id: Identity) -> impl Responder{
    let mut data = Context::new();
    data.insert("title", "Login");

    if let Some(id) = id.identity(){
        return HttpResponse::Ok().body("You are already logged in.")
    }

    let rendered = tera.render("login.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

//Login form
async fn login_user(data: web::Form<LoginUser>, id: Identity) -> impl Responder{
    use schema::users::dsl::{username,users};

    let mut connection = establish_connection();
    let user = users.filter(username.eq(&data.username)).first::<User>(&mut connection);

    match user{
        Ok(u) => {
            if u.password == data.password {
                let session_token = String::from(u.username);
                id.remember(session_token);
                HttpResponse::Ok().body(format!("Succesfully logged in as: {}", data.username))
            }else{
                HttpResponse::Ok().body("Password is incorrect.")
            }
        },
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::Ok().body("User does not exist.")
        }
    }
    
    
}

async fn logout_user(id: Identity) -> impl Responder{
    id.forget();
    HttpResponse::Ok().body("Logged user out.")
}

//Threads page
async fn thread(tera: web::Data<Tera>, id: Identity) -> impl Responder{
    let mut data = Context::new();

    data.insert("title", "Post Thread");

    if let Some(id) = id.identity(){
        let rendered = tera.render("post.html", &data).unwrap();
        return HttpResponse::Ok().body(rendered)
    }

    HttpResponse::Ok().body("User not logged in.")
}

//Thread submission form
async fn post_thread(data: web::Form<PostForm>, id: Identity) -> impl Responder {

    if let Some(id) = id.identity(){
        use schema::users::dsl::{username, users};

        let mut connection = establish_connection();
        let user: Result<User, diesel::result::Error> = users.filter(username.eq(id)).first(&mut connection);

        match user{
            Ok(u) => {
                let new_thread = NewThread::from_post_form(data.title.clone(), data.link.clone(), u.id);
                use schema::threads;

                diesel::insert_into(threads::table)
                    .values(&new_thread)
                    .get_result::<Thread>(&mut connection)
                    .expect("Error posting thread.");

                    return HttpResponse::Ok().body("Posted thread!");
            }
            Err(e) => {
                println!("{:?}", e);
                return HttpResponse::Ok().body("Failed to find user.")
            }
        }

    }
    HttpResponse::Unauthorized().body("User not logged in.")
}

// Thread specific page
async fn thread_page(tera: web::Data<Tera>, id: Identity, web::Path(thread_id): web::Path<i32>) -> impl Responder{

    use schema::threads::dsl::{threads};
    use schema::users::dsl::{users};

    let mut connection = establish_connection();

    let thread :Thread = threads.find(thread_id)
        .get_result(&mut connection)
        .expect("Failed to load thread.");
    
    let user :User = users.find(thread.authorid)
        .get_result(&mut connection)
        .expect("Failed to load thread author.");
    
    let mut data = Context::new();
    data.insert("Title",&format!("{}", thread.title));
    data.insert("thread", &thread);
    data.insert("user", &user);

    if let Some(_id) = id.identity() {
        data.insert("logged_in", "true");
    }else{
        data.insert("logged_in", "false");
    }

    let rendered = tera.render("post.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)    
}

async fn comment(data: web::Form<CommentForm>, id:Identity, web::Path(thread_id): web::Path<i32>) -> impl Responder{

    if let Some(id) = id.identity(){
        use schema::threads::dsl::{threads};
        use schema::users::dsl::{users};

        let mut connection = establish_connection();

        let thread :Thread = thread.find(thread_id)
            .get_result(&mut connection)
            .expect("Failed to find post.");
        
        let user :Result<User, diesel::result::Error> = users
            .filter(username.eq(id))
            .first(&mut connection);

        match user{
            Ok(u) => {
                let parent_id = None;
                let new_comment = NewComment::new(data.comment.clone(), post.id, u.id, parent_id);

                use schema::comments;
                diesel::insert_into(comments::table)
                    .values(&new_comment)
                    .get_result::<Comment>(&mut connection)
                    .expect("Error processing comment.");

                return HttpResponse::Ok().body("Comment posted.")
            }
            Err(e) => {
                println!("{:?}", e);
                return HttpResponse::Ok().body("User not found.");
            }

        }
    }

    HttpResponse::Unauthorized().body("Not logged in.")

}



#[actix_web::main]
async fn main() -> std::io::Result<()>{
    HttpServer::new(|| {
        let tera = Tera::new("templates/**/*").unwrap();
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                .name("auth-cookie")
                .secure(false)
            ))
            .data(tera)
            .route("/", web::get().to(index))
            .route("/register", web::get().to(register))
            .route("/register", web::post().to(register_user))
            .route("/login", web::get().to(login))
            .route("/login", web::post().to(login_user))
            .route("/logout", web::to(logout_user))
            .route("/post", web::get().to(thread))
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