// Common types and utilities for Mazetool

use std::io::Error as IOError;
use std::fmt;
use std::error::Error;
use std::sync::{ Arc, Mutex };

use mazetool::maze::{ Dimensions, Maze };

/// Commands given by the user (interface) to the control logic
#[derive(Debug)]
pub enum Job
{
	GenerateMaze(Dimensions),
	SolveMaze,
	Quit
}

/// Type of requests sent from control logic to the user interface
#[derive(Debug)]
pub enum UIRequest
{
	ParseArgs,
	ShowError(String),
	ShowInfo(String),
	ShowMaze(Arc<Mutex<Maze>>),
	Quit,
}

/// Type of errors returned by different components in the application
#[derive(Debug)]
pub struct ApplicationError
{
	details: String
}

impl ApplicationError
{
	pub fn new(msg: &str) -> ApplicationError
	{
		ApplicationError
		{
			details: msg.to_string()
		}
	}
}

impl fmt::Display for ApplicationError
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "Error: {}", self.details)
	}
}

impl Error for ApplicationError
{
	fn description(&self) -> &str
	{
		&self.details
	}
}

impl From<IOError> for ApplicationError
{
	fn from(err: IOError) -> ApplicationError
	{
		ApplicationError::new(&err.to_string())
	}
}

