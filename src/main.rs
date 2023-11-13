mod nt_backend;
mod state;
mod trie;
mod view;
mod widgets;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nt_backend::Backend;
use ratatui::prelude::{CrosstermBackend, Terminal};
use state::App;

fn startup() -> Result<()> {
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

    let network_table = Backend::new().await?;
    let join = network_table.spawn_update_thread().await?;
    // application state
    let mut app = App::new(network_table);
    t.draw(|f| app.render(f))?;
    loop {
        if app.update().await? {
            break;
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
