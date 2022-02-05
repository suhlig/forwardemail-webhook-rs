use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer, Responder};
use clap::{crate_authors, crate_description, crate_name, crate_version, Parser};
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

    HttpServer::new(move || App::new().data(AppState {
                spool_dir: options.spool_dir.clone(),
            }).service(index_get).service(index_post))
        .bind(format!("127.0.0.1:{}", options.port))?
        .run()
        .await
}

#[get("/")]
async fn index_get() -> impl Responder {
    HttpResponse::Ok().body("Hello\n")
}

#[post("/")]
async fn index_post(data: web::Data<AppState>, body: web::Bytes) -> Result<HttpResponse, Error> {
    let file_name = format!("{}/{}.json", &data.spool_dir, Uuid::new_v4() );
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_name)?;
    file.write_all(&body)?;
    Ok(HttpResponse::Ok().body(format!("stored as {}\n", &file_name)))
}
