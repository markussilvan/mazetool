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
	let use_gui = true;

	info!("Parsing command line parameters");
	if !parse_args(&from_ui_tx)
	{
		return;
	}

	info!("Creating control");

	let control_handle = MazeControl::run(from_ui_rx, to_ui_tx);

	info!("Creating user interface");

	if use_gui
	{
		let mut ui = GraphicalInterface::new(from_ui_tx, to_ui_rx);
		ui.run();
	}
	else
	{
		let mut ui = CommandLineInterface::new(from_ui_tx, to_ui_rx);
		ui.run();
	};

	info!("Main (UI) thread waiting for children to join");
	control_handle.join().unwrap_or_else(|_| return);

	info!("Main thread exiting");
	io::stdout().flush().unwrap();
}

/// Parse command line arguments
fn parse_args(tx: &Sender<Job>) -> bool
{
	info!("Parsing command line arguments");

	let args: Vec<String> = std::env::args().collect();
	let program = &args[0];

	if args.len() < 2
	{
		print_usage(program);
		return false;
	}

	let command = &args[1];
	match command.as_ref()
	{
		"generate" => {
			info!("Generate requested");
			let mut dimensions = Dimensions {
				width: MAZE_DIMENSION_DEFAULT,
				height: MAZE_DIMENSION_DEFAULT 
			};
			if args.len() == 2
			{
				info!("No dimensions given. Using default size.");
			}
			else if args.len() == 4
			{
				info!("Parsing dimensions from command line parameteres");
				if !parse_dimension(&args[2], &mut dimensions.width)
				{
					info!("Parsing maze width failed");
					return false;
				}
				if !parse_dimension(&args[3], &mut dimensions.height)
				{
					info!("Parsing maze height failed");
					return false;
				}
			}
			else
			{
				info!("Invalid parameters");
				print_usage(program);
				return false;
			}
			tx.send(Job::GenerateMaze(dimensions)).unwrap_or_else(|_| return);
		},
		"solve" => {
			info!("Solve requested");
			if args.len() != 2
			{
				info!("Invalid parameters");
				print_usage(program);
				return false;
			}
			tx.send(Job::SolveMaze).unwrap_or_else(|_| return);
		},
		"help" | _ => {
			print_usage(program);
			return false;
		},
	}

	return true;
}

fn print_usage(program: &str)
{
	println!("Usage: {} <command> [options]", program);
	println!("Run the program in the directory containing the database.");
	println!("");
	println!("Commands:");
	println!("  generate        Generate a new random maze of given size");
	println!("  solve           Solve a given maze");
	println!("  help            Print this help");
}

fn parse_dimension(arg: &str, out: &mut usize) -> bool
{
	let mut ret = true;

	match arg.parse::<usize>()
	{
		Ok(n) => *out = n,
		Err(_) => {
			println!("Invalid parameters");
			ret = false;
		}
	}

	if *out > MAZE_DIMENSION_MAX && *out < MAZE_DIMENSION_MIN
	{
		ret = false;
	}

	return ret;
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

