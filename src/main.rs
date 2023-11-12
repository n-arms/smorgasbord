mod app;
mod grid;
mod nt_backend;
mod table;

use anyhow::Result;
use app::App;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nt_backend::Backend;
use ratatui::prelude::{CrosstermBackend, Terminal};

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
    loop {
        t.draw(|f| app.render(f))?;

        if app.update().await? {
            break;
        }
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
