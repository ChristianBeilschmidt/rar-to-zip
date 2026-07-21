use leptos::prelude::*;

mod components;
mod converter;
mod utils;

use components::App;

fn main() {
    mount_to_body(|| view! { <App /> });
}
