#![feature(try_blocks)]

use seed::{prelude::*, *};
use task_manager_types::unix_process::Process;

struct Model {
    processes: Vec<Process>,
}

impl Model {
    fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
        orders.perform_cmd(async {
            Msg::RecievedTest(try { fetch("/proc/list").await?.json().await? })
        });

        Model {
            processes: Vec::new(),
        }
    }

    fn view(&self) -> Node<Msg> {
        pre![self.processes.iter().map(|t| format!("{:#?}", t))]
    }
}

enum Msg {
    RecievedTest(browser::fetch::Result<Vec<Process>>),
}

impl Msg {
    fn update(self, model: &mut Model, _: &mut impl Orders<Msg>) {
        match self {
            Msg::RecievedTest(result) => model.processes = result.expect("Got error on request"),
        }
    }
}

fn main() {
    App::start(body(), Model::init, Msg::update, Model::view);
}
