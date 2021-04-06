use seed::prelude::*;

pub fn get_event_value(event: &web_sys::Event) -> String {
    event
        .target()
        .expect("Unable to reach the target")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("Unknown target's type")
        .value()
}
