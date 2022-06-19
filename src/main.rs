use actix_web::dev::ServiceRequest;
use actix_web::http::header::ContentType;
use actix_web::{
    get, middleware, post,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use clap::{crate_authors, crate_description, crate_name, crate_version, Parser};
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::{
    fs::{self, OpenOptions},
    io::Write,
};
use uuid::Uuid;

#[derive(Parser)]
#[clap(
    name(crate_name!()),
    version( app_version() ),
    author( crate_authors!() ),
    about( crate_description!() ),
)]
pub struct Options {
    /// Spool directory where received mails are stored. If not set, /tmp is used.
    #[clap(short('s'), long, default_value = "/tmp")]
    spool_dir: String,

    /// The port to listen to.
    #[clap(short('p'), long, default_value = "8080")]
    port: u16,
}

struct AppState {
    spool_dir: String,
}

async fn validator(req: ServiceRequest, _credentials: BasicAuth) -> Result<ServiceRequest, Error> {
    let password = match _credentials.password() {
        Some(pwd) => format!("with password '{}'", pwd),
        None => String::from("without password"),
    };

    log::info!(
        "User '{}' authenticating {}",
        _credentials.user_id(),
        password
    );

    Ok(req)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let options: Options = Options::parse();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    log::info!("This is {}", app_description());

    let app_state = Data::new(AppState {
        spool_dir: options.spool_dir.clone(),
    });

    HttpServer::new(move || {
        let auth = HttpAuthentication::basic(validator);

        App::new()
            .app_data(Data::clone(&app_state))
            .app_data(web::PayloadConfig::new(1024 * 1024 * 50))
            .wrap(middleware::Logger::default())
            .service(index_get)
            .service(index_post)
            .service(
                web::scope("/mails")
                    .service(logout)
                    .service(mails)
                    .service(mails_index)
                    .wrap(auth),
            )
    })
    .bind(format!("127.0.0.1:{}", options.port))?
    .run()
    .await
}

#[get("/")]
async fn index_get() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .append_header(("X-Server-Version", app_description()))
        .body(format!("{}\n", app_description())))
}

#[post("/")]
async fn index_post(data: web::Data<AppState>, body: web::Bytes) -> Result<HttpResponse, Error> {
    log::info!("New mail received ({} bytes)", body.len());

    let file_name = format!("{}/{}.json", &data.spool_dir, Uuid::new_v4());
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_name)?;
    file.write_all(&body)?;

    log::info!("stored as {}", &file_name);

    Ok(HttpResponse::Ok().body("Thanks!\n"))
}

#[get("")]
async fn mails_index(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let paths = match fs::read_dir(&data.spool_dir) {
        Ok(p) => p,
        Err(e) => {
            log::error!("failed to read spool dir: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Sorry"));
        }
    };

    let mut links = vec![];

    for path in paths {
        match path {
            Ok(p) => links.push(format!(
                "<li><a href={}/{}>{}</a></li>",
                req.path(),
                p.path().strip_prefix(&data.spool_dir).unwrap().display(),
                p.path().strip_prefix(&data.spool_dir).unwrap().display()
            )),
            Err(_) => (),
        }
    }

    Ok(HttpResponse::Ok().body(format!("<h1>Mails</h1><ul>{}</ul>", links.join("\n"))))
}

#[get("logout")]
async fn logout() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Unauthorized().body("kthxbye"))
}

#[get("{filename:.*}")]
async fn mails(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut path = PathBuf::from(&data.spool_dir);
    let file_name: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    path.push(file_name);

    match fs::read_to_string(path) {
        Ok(content) => Ok(HttpResponse::Ok()
            .insert_header(ContentType::json())
            .body(content)),
        Err(e) => {
            log::error!("failed to read spool file: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Sorry"));
        }
    }
}

/// Provides the app version at build time - either the current git version, or, if not available, the static version string of the crate.
fn app_version() -> &'static str {
    match built_info::GIT_VERSION {
        Some(g) => g,
        None => crate_version!(),
    }
}

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs")); // The file has been placed there by the build script.
}

fn app_description() -> String {
    format!("{} {}", crate_name!(), app_version())
}
