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

use std::fs;

use anyhow::Result;
use backend::mock::{TMap, T};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use network_tables::Value;
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

fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    //let network_table = Nt::new();
    let auto_type: T = Box::new(Value::String("String Chooser".into()));
    let auto_options: T = Box::new(Value::Array(vec![
        Value::String("Left".into()),
        Value::String("Right".into()),
    ]));
    let auto_selected: T = Box::new(Value::String("Left".into()));
    let auto_default: T = Box::new(Value::String("Left".into()));
    let auto: T = Box::new(map! {
        ".type".into() => auto_type,
        "options".into() => auto_options,
        "selected".into() => auto_selected,
        "default".into() => auto_default
    });
    let tabs_type: T = Box::new(Value::String("Tabs".into()));
    let drivetrain_option: T = Box::new(Value::Array(vec![
        Value::String("/Smartdashboard/left encoder".into()),
        Value::String("/Smartdashboard/right encoder".into()),
        Value::String("/Smartdashboard/gyro yaw".into()),
        Value::String("/Smartdashboard/kA".into()),
    ]));
    let auto_option: T = Box::new(Value::Array(vec![Value::String(
        "/Smartdashboard/auto".into(),
    )]));
    let tabs: T = Box::new(map! {
        ".type".into() => tabs_type,
        "drivetrain".into() => drivetrain_option,
        "auto".into() => auto_option
    });
    let counter: T = Box::new(Value::F32(0.0));
    let mut smartdashboard_map: TMap = map! {
        "counter".into() => counter,
        "auto".into() => auto,
        "tabs".into() => tabs
    };
    for name in [
        "left encoder",
        "right encoder",
        "gyro yaw",
        "through bore",
        "kA",
    ] {
        let value: T = Box::new(Value::F32(0.0));
        smartdashboard_map.insert(name.into(), value);
    }
    let smartdashboard: T = Box::new(smartdashboard_map);
    let network_table: TMap = map! {
        "Smartdashboard".into() => smartdashboard
    };

    // application state
    let mut app = App::new(network_table);
    t.draw(|f| app.render(f))?;
    loop {
        match app.update() {
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
    }

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
