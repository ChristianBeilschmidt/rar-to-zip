use leptos::*;
mod components;
mod converter;

use components::App;

fn main() {
    mount_to_body(|| view! { <App /> });
}
