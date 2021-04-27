//! Mazetool 

//#![feature(external_doc)]
//#[doc(include = "../README.md")]

#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate getopts;

mod mazetool;

use std::io;
use std::io::Write;

use simple_logger::SimpleLogger;

use mazetool::mazecontrol::MazeControl;
use mazetool::cli::CommandLineInterface;

/// Main, the entry poin for the application.
fn main()
{
	SimpleLogger::new().init().unwrap_or_else(|_| ::std::process::exit(1));

	let mut control : MazeControl<CommandLineInterface> = MazeControl::new();

	control.run();

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

