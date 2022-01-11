// Mazetool - command line user interface

use std::sync::{ Arc, Mutex };

use crossbeam::channel::{Receiver, Sender};

use super::userinterface::UserInterface;
use super::common::{ UIRequest, Job };
use super::maze::Maze;

/// Command line user interface for Mazetool
pub struct CommandLineInterface
{
	#[allow(dead_code)]
	tx: Sender<Job>,
	rx: Receiver<UIRequest>
}

impl CommandLineInterface
{
	/// Show an info message in the user interface
	///
	/// # Parameters
	///
	/// * `message`       - Information string to show
	///
	fn show_info(&self, message: &str)
	{
		println!("{}", message);
	}

	/// Show an error message in the user interface
	///
	/// # Parameters
	///
	/// * `error`       - Error string to show
	///
	fn show_error(&self, error: &str)
	{
		println!("Error: {}", error);
	}

	fn show_maze(&self, maze: Arc<Mutex<Maze>>)
	{
		match maze.lock()
		{
			Ok(m) => {
				debug!("Size: {} x {}, cells len: {}",
					   m.dimensions.width,
					   m.dimensions.height,
					   m.cells.len());

				for i in 0..m.dimensions.height
				{
					for j in 0..m.dimensions.width
					{
						let cell = &m.cells[j + (i * m.dimensions.width)];
						if cell.on_route
						{
							print!("o");
						}
						else if cell.visited
						{
							print!(".");
						}
						else
						{
							print!("{}", cell.celltype);
						}
					}
					println!("");
				}
			},
			Err(e) => {
				self.show_error(&e.to_string());
			},
		}
	}

	//fn save_maze(&self, maze: Arc<Mutex<Maze>>)
	//{
	//	match maze.lock()
	//	{
	//		Ok(m) => {
	//			match m.write_to_file("saved.maze")
	//			{
	//				Ok(_) => {},
	//				Err(e) => self.show_error(&e.to_string()),
	//			}
	//		},
	//		Err(e) => {
	//			self.show_error(&e.to_string());
	//		},
	//	}
	//}

	/// Handle a single request from the controller
	///
	/// # Returns
	///
	/// * `bool`    - True, if UI thread should keep running
	///
	fn handle_request(&self) -> bool
	{
		let mut keep_running = true;
		let request = self.rx.recv().unwrap_or_else(|_| UIRequest::Quit);
		info!("UI received request: {:?}", request);
		match request
		{
			UIRequest::ShowError(message) => {
				self.show_error(&message);
			},
			UIRequest::ShowInfo(message) => {
				self.show_info(&message);
			},
			UIRequest::ShowMaze(maze) => {
				self.show_maze(maze);
			},
			UIRequest::Quit => {
				keep_running = false;
			},
		};

		if keep_running == false
		{
			info!("UI message loop exiting");
		}

		return keep_running;
	}
}


impl UserInterface for CommandLineInterface
{
	/// Create new command line user interface instance
	fn new(tx: Sender<Job>, rx: Receiver<UIRequest>) -> Self
	{
		CommandLineInterface
		{
			tx: tx,
			rx: rx,
		}
	}

	fn run(&mut self, _show_distances: bool)
	{
		loop
		{
			if self.handle_request() != true
			{
				break;
			}
		}
	}
}
