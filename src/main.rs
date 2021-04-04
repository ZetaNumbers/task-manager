#![feature(proc_macro_hygiene, decl_macro, try_blocks)]

#[macro_use]
extern crate rocket;

mod unix_process;

use libc::pid_t;
use rocket_contrib::{json::Json, serve::StaticFiles};
use task_manager_types::unix_process::Process;

#[get("/list")]
fn list_proc() -> Json<Vec<Process>> {
    Json(unix_process::list_processes())
}

#[get("/get/<pid>")]
fn get_proc(pid: pid_t) -> Json<Option<Process>> {
    Json(unix_process::get_process(pid))
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/client/dist")),
        )
        .mount("/proc", routes![list_proc])
        .launch();
}
