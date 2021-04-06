#![feature(try_blocks)]

mod unix_process;

use std::{convert::TryFrom, io};

use actix_files as afs;
use actix_web::{get, post, web, App, HttpServer, Responder};
use libc::{c_int, pid_t};
use task_manager_types::unix_process::{CSpawnArgs, Process, SpawnArgs};

#[get("/list")]
async fn list() -> web::Json<Vec<Process>> {
    web::Json(unix_process::list_processes())
}

#[get("/get/{pid}")]
async fn get(pid: web::Path<pid_t>) -> web::Json<Option<Process>> {
    web::Json(unix_process::process_from_pid(*pid))
}

#[post("/kill/{pid}")]
async fn kill(pid: web::Path<pid_t>) -> io::Result<impl Responder> {
    unix_process::kill(*pid)?;
    Ok(web::HttpResponse::Ok())
}

#[post("/suspend/{pid}")]
async fn suspend(pid: web::Path<pid_t>) -> io::Result<impl Responder> {
    unix_process::suspend(*pid)?;
    Ok(web::HttpResponse::Ok())
}

#[post("/continue/{pid}")]
async fn continue_(pid: web::Path<pid_t>) -> io::Result<impl Responder> {
    unix_process::continue_(*pid)?;
    Ok(web::HttpResponse::Ok())
}

#[post("/setprioriry/{pid}")]
async fn setprioriry(pid: web::Path<pid_t>, prio: web::Json<c_int>) -> io::Result<impl Responder> {
    unix_process::set_priority(*pid, *prio)?;
    Ok(web::HttpResponse::Ok())
}

#[post("/spawn")]
async fn spawn(web::Json(body): web::Json<SpawnArgs>) -> io::Result<web::Json<pid_t>> {
    unix_process::posix_spawn(CSpawnArgs::try_from(body)?).map(web::Json)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/proc")
                    .service(list)
                    .service(get)
                    .service(spawn)
                    .service(setprioriry)
                    .service(suspend)
                    .service(continue_)
                    .service(kill),
            )
            .service(
                afs::Files::new(
                    "/static",
                    concat!(env!("CARGO_MANIFEST_DIR"), "/client/dist"),
                )
                .index_file("index.html"),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
