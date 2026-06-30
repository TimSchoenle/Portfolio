mod components;
mod github;
mod hooks;
mod i18n;
mod pages;
mod router;

use components::app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
