//! Mazetool 

//#![feature(external_doc)]
//#[doc(include = "../README.md")]

#[macro_use]
extern crate log;

mod mazetool;

use std::io;
use std::io::Write;

use crossbeam::channel::unbounded;
use crossbeam::channel::Sender;
use simple_logger::SimpleLogger;
use log::LevelFilter;
use clap::{Arg, App, AppSettings, SubCommand};

use mazetool::maze::{MAZE_DIMENSION_MIN, MAZE_DIMENSION_MAX, MAZE_DIMENSION_DEFAULT};
use mazetool::maze::Dimensions;
use mazetool::mazecontrol::MazeControl;
use mazetool::userinterface::UserInterface;
use mazetool::cli::CommandLineInterface;
use mazetool::gui::GraphicalInterface;
use mazetool::common::Job;


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
	let mut use_gui = false;

	info!("Parsing command line parameters");
	if !parse_args(&from_ui_tx, &mut use_gui)
	{
		return;
	}

	info!("Creating control");

	let control_handle = MazeControl::run(from_ui_rx, to_ui_tx);

	info!("Creating user interface");

	if use_gui
	{
		let mut ui = Box::new(GraphicalInterface::new(from_ui_tx, to_ui_rx));
		ui.run();
	}
	else
	{
		let mut ui = Box::new(CommandLineInterface::new(from_ui_tx, to_ui_rx));
		ui.run();
	};

	info!("Main (UI) thread waiting for children to join");
	control_handle.join().unwrap_or_else(|_| return);

	info!("Main thread exiting");
	io::stdout().flush().unwrap();
}

/// Parse command line arguments
fn parse_args(tx: &Sender<Job>, use_gui: &mut bool) -> bool
{
	let mut success = true;
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
	                                      .arg(Arg::with_name("file").required(true))
	                      )
	                      .get_matches();
	
	if matches.is_present("gui")
	{
		*use_gui = true;
	}
	else
	{
		*use_gui = false;
	}
    
	if let Some(matches) = matches.subcommand_matches("generate")
	{
		info!("Generate requested");
		let mut dimensions = Dimensions {
			width: MAZE_DIMENSION_DEFAULT,
			height: MAZE_DIMENSION_DEFAULT 
		};
		if let Some(x) = matches.value_of("x")
		{
			if let Ok(w) = x.parse()
			{
				if w >= MAZE_DIMENSION_MIN && w <= MAZE_DIMENSION_MAX
				{
					dimensions.width = w;
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
						dimensions.height = h;
					}
				},
				Err(_e) => ()
			}
		}
		tx.send(Job::GenerateMaze(dimensions)).unwrap();
		success = true;
	}

	if let Some(_matches) = matches.subcommand_matches("solve")
	{
		println!("Solving is not implemented (yet?)");
		tx.send(Job::SolveMaze).unwrap();
		success = true;
	}

    return success;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn create_control_from_main()
	{
		let _ : MazeControl<CommandLineInterface> = MazeControl::new();
	}
}

