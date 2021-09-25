// Common types and utilities for Mazetool

use std::io::Error as IOError;
use std::fmt;
use std::error::Error;
use std::sync::{ Arc, Mutex };

use super::maze::{ Dimensions, Maze };

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
pub struct AppError
{
	details: String
}

impl AppError
{
	pub fn new(msg: &str) -> AppError
	{
		AppError
		{
			details: msg.to_string()
		}
	}
}

impl fmt::Display for AppError
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "Error: {}", self.details)
	}
}

impl Error for AppError
{
	fn description(&self) -> &str
	{
		&self.details
	}
}

impl From<IOError> for AppError
{
	fn from(err: IOError) -> AppError
	{
		AppError::new(&err.to_string())
	}
}

