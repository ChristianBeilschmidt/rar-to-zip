use leptos::prelude::*;

mod components;
mod converter;

use components::App;

fn main() {
    mount_to_body(|| view! { <App /> });
}
