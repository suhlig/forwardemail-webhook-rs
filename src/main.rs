use actix_files::Files;
use actix_web::{middleware, post, web, App, Error, HttpResponse, HttpServer};
use clap::{crate_authors, crate_description, crate_name, crate_version, Parser};
use log;
use simple_logger::SimpleLogger;
use std::{fs::OpenOptions, io::Write};
use uuid::Uuid;

#[derive(Parser)]
#[clap(
    name(crate_name!()),
    version( crate_version!() ),
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

    log::info!("This is {} v.{}", crate_name!(), crate_version!());

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                spool_dir: options.spool_dir.clone(),
            })
            .data(web::PayloadConfig::new(1024 * 1024 * 50))
            .wrap(middleware::Logger::default())
            .service(index_post)
            .service(Files::new("/mails", options.spool_dir.as_str()).show_files_listing())
    })
    .bind(format!("127.0.0.1:{}", options.port))?
    .run()
    .await
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
