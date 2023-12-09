mod nt;
mod nt_worker;
mod state;
mod trie;
mod view;
mod widget_tree;
mod widgets;

use std::fs;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nt::Backend;
use ratatui::prelude::{CrosstermBackend, Terminal};
use state::App;
use tracing::{event, Level};
use tracing_subscriber::fmt::Subscriber;

fn init_logging() -> Result<()> {
    let file = fs::OpenOptions::new().write(true).open("smorgasbord.log")?;
    Subscriber::builder().with_writer(file).init();
    Ok(())
}

fn startup() -> Result<()> {
    init_logging()?;
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

async fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let network_table = Backend::new().await;
    // application state
    let mut app = App::new(network_table);
    t.draw(|f| app.render(f))?;
    loop {
        match app.update().await {
            Ok(result) => {
                if result {
                    break;
                }
            }
            Err(error) => {
                event!(Level::ERROR, "top level error {}", error)
            }
        }
        t.draw(|f| app.render(f))?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // setup terminal
    startup()?;

    let result = run().await;

    // teardown terminal before unwrapping Result of app run
    shutdown()?;

    result?;

    Ok(())
}
