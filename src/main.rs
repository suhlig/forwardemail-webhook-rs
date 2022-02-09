use actix_files::Files;
use actix_web::{middleware, get, post, web, App, Error, HttpResponse, HttpServer};
use clap::{crate_authors, crate_description, crate_name, crate_version, Parser};
use simple_logger::SimpleLogger;
use std::{fs::OpenOptions, io::Write};
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let options: Options = Options::parse();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    log::info!("This is {}", app_description());

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                spool_dir: options.spool_dir.clone(),
            })
            .data(web::PayloadConfig::new(1024 * 1024 * 50))
            .wrap(middleware::Logger::default())
            .service(index_get)
            .service(index_post)
            .service(Files::new("/mails", options.spool_dir.as_str()).show_files_listing())
    })
    .bind(format!("127.0.0.1:{}", options.port))?
    .run()
    .await
}

#[get("/")]
async fn index_get() -> Result<HttpResponse, Error> {
    Ok(
        HttpResponse::Ok()
        .header("X-Server-Version", app_description())
        .body( format!("{}\n", app_description()) )
    )
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
    Ok(HttpResponse::Ok().body(format!("stored as {}\n", &file_name)))
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
