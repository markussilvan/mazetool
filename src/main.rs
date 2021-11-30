//! Mazetool 

//#![feature(external_doc)]
//#[doc(include = "../README.md")]

#[macro_use]
extern crate log;

mod mazetool;

use std::io;
use std::io::Write;
use std::str::FromStr;

use crossbeam::channel::unbounded;
use simple_logger::SimpleLogger;
use log::LevelFilter;
use clap::{Arg, App, AppSettings, SubCommand, ArgMatches};

use mazetool::maze::{MAZE_DIMENSION_MIN, MAZE_DIMENSION_MAX, MAZE_DIMENSION_DEFAULT};
use mazetool::maze::Dimensions;
use mazetool::mazecontrol::MazeControl;
use mazetool::userinterface::UserInterface;
use mazetool::cli::CommandLineInterface;
use mazetool::gui::GraphicalInterface;
use mazetool::common::Job;
use mazetool::common::SolveMethod;

struct Config
{
	use_gui: bool,
	solve: Option<SolveMethod>,
	dimensions: Dimensions,
}

impl Config
{
	fn new() -> Self
	{
		Config {
			use_gui: false,
			solve: None,
			dimensions: Dimensions {
				width: MAZE_DIMENSION_DEFAULT,
				height: MAZE_DIMENSION_DEFAULT 
			}
		}
	}
}

/// Main, the entry poin for the application.
fn main()
{
	//SimpleLogger::new().with_level(LevelFilter::Off).init().unwrap_or_else(|_| ::std::process::exit(1));
	SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap_or_else(|_| ::std::process::exit(1));

	// from_ui_tx - send from ui to control
	// from_ui_rx - receive from ui to control
	// to_ui_tx   - send to ui from control
	// to_ui_rx   - receive from ui to control
	let (from_ui_tx, from_ui_rx) = unbounded();
	let (to_ui_tx, to_ui_rx) = unbounded();
	let mut config = Config::new();

	info!("Parsing command line parameters");
	if !parse_args(&mut config)
	{
		return;
	}

	info!("Creating control");

	let control_handle = MazeControl::run(from_ui_rx, to_ui_tx);

	info!("Creating user interface");

	from_ui_tx.send(Job::GenerateMaze(config.dimensions)).unwrap();

	//TODO: works here (but not after constructing gui) (which is what i need)
	if let Some(solve_method) = config.solve
	{
		from_ui_tx.send(Job::SolveMaze(solve_method)).unwrap();
	}
	else
	{
		//from_ui_tx.send(Job::SolveMaze(SolveMethod::GraphElimination)).unwrap();
	}

	//std::thread::sleep(std::time::Duration::from_millis(1000));

	if config.use_gui
	{
		let mut ui = Box::new(GraphicalInterface::new(from_ui_tx.clone(), to_ui_rx));
		ui.run();
	}
	else
	{
		let mut ui = Box::new(CommandLineInterface::new(from_ui_tx.clone(), to_ui_rx));
		ui.run();
	};

	//if let Some(solve_method) = config.solve
	//{
	//	from_ui_tx.send(Job::SolveMaze(solve_method)).unwrap();
	//}

	info!("Main (UI) thread waiting for children to join");
	control_handle.join().unwrap_or_else(|_| return);

	info!("Main thread exiting");
	io::stdout().flush().unwrap();
}

/// Parse command line arguments
fn parse_args(config: &mut Config) -> bool
{
	let mut success = false;
	let matches = App::new("mazetool")
	                      .version("0.1.0")
	                      .author("Markus Silván <markus.silvan@iki.fi>")
	                      .about("Maze generating and solving tool")
	                      .setting(AppSettings::SubcommandRequiredElseHelp)
	                      .args_from_usage("
	                           --gui                'Use graphical interface'")
	                      .subcommand(SubCommand::with_name("generate")
	                                      .about("generates a new maze")
	                                      .arg(Arg::with_name("x")
		                                      .required(true)
		                                      .help("Width of the maze"))
	                                      .arg(Arg::with_name("y")
		                                      .required(true)
		                                      .help("Height of the maze"))
	                      )
	                      .subcommand(SubCommand::with_name("solve")
	                                      .about("solves a given maze")
	                                      .arg(Arg::with_name("method")
		                                      .required(true))
	                                          .help("GraphOnly, GraphElimination or AStar")
	                                      .arg(Arg::with_name("x")
		                                      .required(false)
		                                      .help("Width of the maze"))
	                                      .arg(Arg::with_name("y")
		                                      .required(false)
		                                      .help("Height of the maze"))
	                      )
	                      .get_matches();
	
	if matches.is_present("gui")
	{
		config.use_gui = true;
	}
	else
	{
		config.use_gui = false;
	}
    
	if let Some(generate_matches) = matches.subcommand_matches("generate")
	{
		info!("Generate requested");
		success = parse_dimensions(config, generate_matches);
	}

	if let Some(solve_matches) = matches.subcommand_matches("solve")
	{
		if let Some(m) = solve_matches.value_of("method")
		{
			if let Ok(method) = SolveMethod::from_str(m)
			{
				config.solve = Some(method);
				success = true;
			}
			else
			{
				println!("Invalid solve method specified");
				success = false;
			}
		}
		if success == true
		{
			success = parse_dimensions(config, solve_matches);
		}
	}

    return success;
}

fn parse_dimensions(config: &mut Config, matches: &ArgMatches<'_>) -> bool
{
	if let Some(x) = matches.value_of("x")
	{
		if let Ok(w) = x.parse()
		{
			if w >= MAZE_DIMENSION_MIN && w <= MAZE_DIMENSION_MAX
			{
				config.dimensions.width = w;
			}
			else
			{
				return false;
			}
		}
	}
	if let Some(y) = matches.value_of("y")
	{
		// same as above, written in a different way
		match y.parse()
		{
			Ok(h) => {
				if h >= MAZE_DIMENSION_MIN && h <= MAZE_DIMENSION_MAX
				{
					config.dimensions.height = h;
				}
			},
			Err(_e) => {
				return false;
			}
		}
	}

	true
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn create_cli()
	{
		let (from_ui_tx, _from_ui_rx) = unbounded();
		let (_to_ui_tx, to_ui_rx) = unbounded();
		let _ = CommandLineInterface::new(from_ui_tx, to_ui_rx);
	}

	#[test]
	fn create_gui()
	{
		let (from_ui_tx, _from_ui_rx) = unbounded();
		let (_to_ui_tx, to_ui_rx) = unbounded();
		let _ = GraphicalInterface::new(from_ui_tx, to_ui_rx);
	}

	#[test]
	fn run_and_quit_control()
	{
		let (from_ui_tx, from_ui_rx) = unbounded();
		let (to_ui_tx, _to_ui_rx) = unbounded();
		let _ = MazeControl::run(from_ui_rx, to_ui_tx);
		from_ui_tx.send(Job::Quit).unwrap();
	}
}

