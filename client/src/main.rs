#![feature(try_blocks, never_type)]

mod input_util;
mod json_parse;
mod value_input;

use std::array::IntoIter;

use browser::fetch;
use json_parse::JsonParse;
use seed::{prelude::*, virtual_dom::update_el::UpdateElForIterator, *};
use task_manager_types::unix_process::{Process, SpawnArgs};

struct Model {
    proc_list: Vec<Process>,
    select_pid: value_input::Model<i32>,
    spawn_args: value_input::Model<JsonParse<SpawnArgs>>,
    prioriry: value_input::Model<i64>,
}

impl Model {
    fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
        orders.perform_cmd(get_list());

        Model {
            proc_list: Vec::new(),
            select_pid: value_input::Model::new("Select pid".to_string(), 0),
            spawn_args: value_input::Model::new(
                "Spawn arguments".to_string(),
                JsonParse(SpawnArgs {
                    program: "deno".to_string(),
                    args: Vec::new(),
                }),
            ),
            prioriry: value_input::Model::new("Priority".to_string(), 0),
        }
    }

    fn view(&self) -> Node<Msg> {
        div![
            C!["app"],
            self.spawn_args.view().map_msg(Msg::SpawnArgsMsg),
            button![ev(Ev::Click, |_| Msg::Spawn), "Spawn"],
            self.select_pid.view().map_msg(Msg::SelectPidMsg),
            self.prioriry.view().map_msg(Msg::PrioriryMsg),
            button![ev(Ev::Click, |_| Msg::SetPriority), "Set"],
            button![ev(Ev::Click, |_| Msg::Kill), "Kill"],
            button![ev(Ev::Click, |_| Msg::Suspend), "Suspend"],
            button![ev(Ev::Click, |_| Msg::Continue), "Continue"],
            button![ev(Ev::Click, |_| Msg::Refresh), "Refresh"],
            table![
                C!["task-table"],
                Process::FIELD_NAMES.iter().map(|name| th![name]),
                self.proc_list.iter().map(|p| tr![
                    IntoIter::new(p.list_field_displays()).map(|f| td![f.to_string()])
                ])
            ]
        ]
    }
}

enum Msg {
    SelectPidMsg(value_input::Msg),
    PrioriryMsg(value_input::Msg),
    SpawnArgsMsg(value_input::Msg),
    RecievedList(Vec<Process>),
    HandleError(fetch::FetchError),
    Refresh,
    Kill,
    Spawn,
    SetPriority,
    Suspend,
    Continue,
}

impl Msg {
    fn update(self, model: &mut Model, orders: &mut impl Orders<Msg>) {
        match self {
            Msg::RecievedList(proc_list) => model.proc_list = proc_list,
            Msg::Refresh => drop(orders.perform_cmd(get_list())),
            Msg::HandleError(e) => Err(e).expect("Got FetchError!"),
            Msg::SelectPidMsg(msg) => msg.update(&mut model.select_pid),
            Msg::SpawnArgsMsg(msg) => msg.update(&mut model.spawn_args),
            Msg::Spawn => drop(orders.perform_cmd(post_spawn(model.spawn_args.value.0.clone()))),
            Msg::SetPriority => drop(orders.perform_cmd(post_setpriority(
                model.select_pid.value,
                model.prioriry.value,
            ))),
            Msg::PrioriryMsg(msg) => msg.update(&mut model.prioriry),
            Msg::Kill => drop(orders.perform_cmd(post_kill(model.select_pid.value))),
            Msg::Suspend => drop(orders.perform_cmd(post_suspend(model.select_pid.value))),
            Msg::Continue => drop(orders.perform_cmd(post_continue(model.select_pid.value))),
        }
    }
}

async fn get_list() -> Msg {
    fetch::Result::<Vec<Process>>::map_or_else(
        try { fetch("/proc/list").await?.json().await? },
        Msg::HandleError,
        Msg::RecievedList,
    )
}

async fn post_kill(pid: i32) -> Msg {
    fetch::Request::new(&format!("/proc/kill/{}", pid))
        .method(fetch::Method::Post)
        .fetch()
        .await
        .map_or_else(Msg::HandleError, |_| Msg::Refresh)
}

async fn post_suspend(pid: i32) -> Msg {
    fetch::Request::new(&format!("/proc/suspend/{}", pid))
        .method(fetch::Method::Post)
        .fetch()
        .await
        .map_or_else(Msg::HandleError, |_| Msg::Refresh)
}

async fn post_continue(pid: i32) -> Msg {
    fetch::Request::new(&format!("/proc/continue/{}", pid))
        .method(fetch::Method::Post)
        .fetch()
        .await
        .map_or_else(Msg::HandleError, |_| Msg::Refresh)
}

async fn post_spawn(spawn_args: SpawnArgs) -> Msg {
    fetch::Result::map_or_else(
        try {
            fetch::Request::new("/proc/spawn")
                .method(fetch::Method::Post)
                .json(&spawn_args)?
                .fetch()
                .await?;
        },
        Msg::HandleError,
        |()| Msg::Refresh,
    )
}

async fn post_setpriority(pid: i32, prio: i64) -> Msg {
    fetch::Result::map_or_else(
        try {
            fetch::Request::new(format!("/proc/setprioriry/{}", pid))
                .method(fetch::Method::Post)
                .json(&prio)?
                .fetch()
                .await?;
        },
        Msg::HandleError,
        |()| Msg::Refresh,
    )
}

fn main() {
    App::start(body(), Model::init, Msg::update, Model::view);
}
