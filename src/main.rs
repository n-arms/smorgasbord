#![allow(clippy::module_inception)]
#![warn(clippy::pedantic)]

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

mod backend;
mod state;
mod view;
mod widget_tree;
mod widgets;

use std::{
    fs,
    time::{Duration, Instant},
};

use anyhow::Result;
use backend::mock::{self};
use crossterm::{
    event::{self as term_event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use state::App;
use tracing::{event, Level};
use tracing_subscriber::fmt::Subscriber;
use widgets::Size;

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

fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    //let network_table = Nt::new();

    let network_table = mock::stressing_example(80);

    // application state
    let mut app = App::new(
        Size {
            width: 8,
            height: 10,
        },
        network_table,
    );
    t.draw(|f| app.render(f))?;
    let mut total_time = Duration::ZERO;
    let mut last;
    loop {
        let event = if term_event::poll(Duration::from_millis(20))? {
            Some(term_event::read()?)
        } else {
            None
        };
        last = Instant::now();
        match app.update(event) {
            Ok(result) => {
                if result {
                    break;
                }
            }
            Err(error) => {
                event!(Level::ERROR, "top level error {}", error);
            }
        }
        t.draw(|f| app.render(f))?;

        total_time += last.elapsed();

        if app.start_time.elapsed() > Duration::from_secs(15) {
            break;
        }
    }

    event!(Level::INFO, "took a cpu total of {total_time:?} to run");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // setup terminal
    startup()?;

    let result = run();

    // teardown terminal before unwrapping Result of app run
    shutdown()?;

    result?;

    Ok(())
}
