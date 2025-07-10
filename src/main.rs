pub mod db;

use actix_files::Files;
use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use chrono_tz::Asia::Kolkata;
use flexi_logger::{Duplicate, FileSpec, Logger, WriteMode};
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::error::Error;
use tera::{Context, Tera};
use tokio::net::TcpStream;

#[derive(Serialize, Deserialize)]
struct UserData {
    message: String,
}

async fn home(tmpl: web::Data<Tera>) -> impl Responder {
    let ctx = Context::new(); //used to pass variables to the webpage
    let index_page = match tmpl.render("index.html", &ctx) {
        Ok(page) => page,
        Err(e) => {
            error!("Template render error: {}", e);
            panic!("Template render error: {}", e);
        }
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(index_page)
}

#[get("/nettest")]
async fn net_test() -> HttpResponse {
    match TcpStream::connect("db.jpogzpxmojratlnoxozq.supabase.co:5432").await {
        Ok(_) => HttpResponse::Ok().body("TCP connection successful"),
        Err(e) => HttpResponse::Ok().body(format!("TCP connection failed: {}", e)),
    }
}

#[post("/submit")]
async fn handle_submit(
    userdata: web::Form<UserData>,
    pool: web::Data<PgPool>,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let dt = Utc::now()
        .with_timezone(&Kolkata)
        .format("%d/%m/%Y %H:%M:%S")
        .to_string();

    let ctx = Context::new(); //used to pass variables to the webpage
    let server_error_page = match tmpl.render("501.html", &ctx) {
        Ok(page) => page,
        Err(e) => {
            error!("Cannot render internal server html page");
            panic!("Template render error: {}", e);
        }
    };

    let user_message = &userdata.message;

    if let Err(e) = db::insert_data(&pool, &dt, user_message).await {
        error!("Error inserting data: {}", e);
        return HttpResponse::InternalServerError()
            .content_type("text/html")
            .body(server_error_page);
    }

    info!(
        "Inserted ====>  {}-- {} <====  into database",
        &dt, &user_message
    );

    HttpResponse::Found()
        .insert_header(("Location", "/"))
        .finish()
}

#[derive(Deserialize)]
struct VerID {
    id: i32,
}

#[get("/messages/{id}")]
async fn view_messages(
    pool: web::Data<PgPool>,
    tmpl: web::Data<Tera>,
    path: web::Path<VerID>,
) -> impl Responder {
    if path.id == 121121 {
        let ctx = Context::new(); //used to pass variables to the webpage
        let server_error_page = match tmpl.render("501.html", &ctx) {
            Ok(page) => page,
            Err(e) => {
                error!("Template render error: {}", e);
                return HttpResponse::InternalServerError().body("Error rendering page");
            }
        };

        let rows: Vec<db::FetchedMessage> = match db::fetch_all(&pool).await {
            Ok(data) => data,
            Err(e) => {
                error!("Error due to {e}");
                return HttpResponse::InternalServerError().body(server_error_page);
            }
        };

        let mut rows_contx = Context::new();
        rows_contx.insert("secret_messages", &rows);

        let message_page = match tmpl.render("messages.html", &rows_contx) {
            Ok(t) => t,
            Err(e) => {
                error!("Cannot render page due to -----> {e}");
                return HttpResponse::InternalServerError()
                    .body(format!("Template error: {:?}", e));
            }
        };

        HttpResponse::Ok()
            .content_type("text/html")
            .body(message_page)
    } else {
        HttpResponse::Found()
            .insert_header(("Location", "/"))
            .finish()
    }
}

#[post("/delete_mesg/{id}")]
async fn delete_single_message(pool: web::Data<PgPool>, path: web::Path<VerID>) -> impl Responder {
    let id = path.id;
    info!("Deleting message with id:{}", id);

    match db::delete_mesg(&pool, id).await {
        Ok(_) => HttpResponse::Found()
            .insert_header(("Location", "/message/121121"))
            .finish(),
        Err(e) => {
            error!("Failed to delete message: {}", e);
            HttpResponse::InternalServerError().body("Failed to delete message")
        }
    }
}

async fn fallback(req: HttpRequest, tmpl: web::Data<Tera>) -> impl Responder {
    info!(
        "Landed in fallback page due to path for {} to {}",
        req.method(),
        req.path()
    );

    let ctx = Context::new(); //used to pass variables to the webpage
    let error_page = match tmpl.render("404.html", &ctx) {
        Ok(page) => page,
        Err(e) => {
            error!("Template render error: {}", e);
            return HttpResponse::InternalServerError().body("Error rendering page");
        }
    };

    HttpResponse::NotFound()
        .content_type("text/html")
        .body(error_page)
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    // Initialize logger
    Logger::try_with_str("info")
        .unwrap()
        .log_to_file(
            FileSpec::default()
                .directory("log_files")
                .basename("app") // this will make it app.log
                .suffix("log"),
        )
        .write_mode(WriteMode::BufferAndFlush)
        .append() // <-- important: append instead of overwrite
        .duplicate_to_stdout(Duplicate::Info)
        .start()
        .unwrap();

    let db_url = std::env::var("DB_URL").unwrap_or_else(|e| {
        info!("Cannot find any variable DB_URL in env file due to error: {e}");
        panic!("Missing DB_URL");
    });

    /*let socket_ip = std::env::var("IP").unwrap_or_else(|e| {
        error!("Cannot find any variable PORT in env file due to error: {e}");
        panic!("Cannot find any variable IP in env file due to error: {e}");
    });*/
    let socket_ip = "0.0.0.0";

    let socket_port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());

    let template_path = std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| {
        error!("Cannot find TEMPLATE_PATH in env file");
        panic!("Cannot find TEMPLATE_PATH in env file");
    });

    let static_path = std::env::var("STATIC_PATH").unwrap_or_else(|_| {
        error!("Cannot find STATIC_PATH in env file");
        panic!("Cannot find STATIC_PATH in env file");
    });

    let dbpool = db::db_init(&db_url).await.unwrap_or_else(|e| {
        error!("Database init failed due to :{e}");
        panic!("Database init failed due to :{e}");
    });

    let tera = web::Data::new(match Tera::new(&template_path) {
        Ok(t) => t,
        Err(e) => {
            error!("Template parsing error due to: {e}");
            panic!("Parsing error(s): {}", e);
        }
    });

    let socket = format!("{}:{}", socket_ip, socket_port);
    info!("Starting server at http://{}", socket);

    HttpServer::new(move || {
        App::new()
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .app_data(web::Data::new(dbpool.clone()))
            .app_data(tera.clone())
            .service(Files::new("/static", &static_path).show_files_listing())
            .route("/", web::get().to(home))
            .route("/", web::head().to(home))
            .service(handle_submit)
            .service(net_test)
            .service(view_messages)
            .service(delete_single_message)
            .default_service(web::route().to(fallback))
    })
    .bind(&socket)
    .unwrap_or_else(|e| {
        error!("Failed to bind to {}: {}", &socket, e);
        panic!("Failed to bind to {}: {}", &socket, e);
    })
    .run()
    .await
    .unwrap_or_else(|e| {
        error!("Server runtime error due to {}: {}", socket, e);
        panic!("Server runtime error due to {}: {}", socket, e);
    });

    Ok(())
}
