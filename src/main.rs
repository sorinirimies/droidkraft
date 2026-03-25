use crate::app::App;

pub mod adb;
pub mod app;
pub mod effects;
pub mod event;
pub mod fastboot;
pub mod logcat;
pub mod menu;
pub mod message;
pub mod model;

pub mod update;
pub mod view;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}
