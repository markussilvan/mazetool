//! Mazetool 

//#![feature(external_doc)]
//#[doc(include = "../README.md")]

#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate getopts;
extern crate rand;

mod mazetool;

use std::io;
use std::io::Write;
use std::sync::mpsc;

use simple_logger::SimpleLogger;
use log::LevelFilter;

use mazetool::mazecontrol::MazeControl;
use mazetool::userinterface::UserInterface;
use mazetool::cli::CommandLineInterface;

/// Main, the entry poin for the application.
fn main()
{
	//SimpleLogger::new().with_level(LevelFilter::Off).init().unwrap_or_else(|_| ::std::process::exit(1));
	SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap_or_else(|_| ::std::process::exit(1));


	// from_ui_tx - send from ui to control
	// from_ui_rx - receive from ui to control
	// to_ui_tx   - send to ui from control
	// to_ui_rx   - receive from ui to control
	let (from_ui_tx, from_ui_rx) = mpsc::channel();
	let (to_ui_tx, to_ui_rx) = mpsc::channel();

	info!("Creating control");

	let control_handle = MazeControl::run(from_ui_rx, to_ui_tx);

	info!("Creating user interface");

	let ui : CommandLineInterface = CommandLineInterface::new(from_ui_tx, to_ui_rx);

	ui.run();

	info!("Main (UI) thread waiting for children to join");
	control_handle.join().unwrap_or_else(|_| return);

	info!("Main thread exiting");
	io::stdout().flush().unwrap();
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

