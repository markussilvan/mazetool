//! Mazetool application control
//!
//! Implements the application logic.
//! Supports different user interface implementations.

use std::sync::mpsc;
use std::marker::PhantomData;
use std::sync::{ Arc, Mutex, MutexGuard };
use std::result::Result;

//use rand::prelude::*;
//use rand::{ thread_rng, Rng };
use rand::seq::SliceRandom;

use mazetool::userinterface::UserInterface;
use mazetool::common::{ UIRequest, Job, AppError };
use mazetool::maze::{ Direction, Dimensions, Maze };

/// A class for main logic (controller) of the food consumption database application.
///
/// Accesses database through FoodieDatabase.
/// Interact with user through a UserInterface implementation.
pub struct MazeControl<T: UserInterface>
{
	ui_type: PhantomData<T>,
	to_ui_tx: Option<mpsc::Sender<UIRequest>>,
	maze: Arc<Mutex<Maze>>,
}

impl<T> MazeControl<T>
where T: UserInterface
{
	/// Creates a new MazeControl instance.
	pub fn new() -> Self
	{
		let mc = MazeControl
		{
			ui_type: PhantomData,
			to_ui_tx: None,
			maze: Arc::new(Mutex::new(Maze::new())),
		};
		return mc;
	}

	/// Run the control
	///
	/// Initializes and runs the UI (which must create its own thread).
	/// Continues to run the control message loop in the main thread.
	///
	/// Communicates with the UI using channels.
	///
	pub fn run(&mut self)
	{
		let (from_ui_tx, from_ui_rx) = mpsc::channel();
		let (to_ui_tx, to_ui_rx) = mpsc::channel();
		self.to_ui_tx = Some(to_ui_tx.clone());

		debug!("Starting user interface");

		let handle = <T>::run(from_ui_tx, to_ui_rx);

		debug!("Main thread continues");

		to_ui_tx.send(UIRequest::ParseArgs).unwrap_or_else(|_| return);
		loop {
			match from_ui_rx.recv().unwrap_or_else(|_| Job::Quit)
			{
				job => {
					debug!("Main: Received job: {:?}", job);
					match job
					{
						Job::GenerateMaze(dimensions) => {
							to_ui_tx.send(UIRequest::ShowInfo("Generating...".to_string()))
								.unwrap_or_else(|_| return);
							match self.generate_maze(dimensions)
							{
								Ok(_) => info!("Maze generated successfully"),
								Err(e) => self.show_error(format!("Error generating maze: {}", e))
							};
						},
						Job::SolveMaze => {
							self.solve_maze();
						},
						Job::Quit => {
							break;
						},
					};
				},
			};
		}
		debug!("Main thread waiting for children to join");
		handle.join().unwrap_or_else(|_| return);
	}

	/// Send a job to the UI to show an error message
	///
	/// # Arguments
	///
	/// * `message`     - The error string
	///
	fn show_error(&self, message: String)
	{
		match self.to_ui_tx
		{
			Some(ref channel) => {
				channel.send(UIRequest::ShowError(message)).unwrap();
			},
			None => {},
		}
	}

	/// Generate a new maze of the given size
	///
	/// 1. Close all cells
	/// 2. Choose starting cell and open it. This is the current cell
	/// 3. Pick a cell adjacent to the current cell that hasn’t been visited and open it. It becomes the current cell
	/// 4. Repeat 2 until no adjacent wall can be selected
	/// 5. The previous cell becomes the current cell. If this cell is the starting cell, then we are done. Else go to 2
	///
	/// # Arguments
	///
	/// * `dimensions`  - The dimensions of a new maze to generate
	///
	fn generate_maze(&mut self, dimensions: Dimensions) -> Result<(), AppError>
	{
		info!("Request to generate a maze received");

		match self.maze.lock()
		{
			Ok(mut m) => {
				m.reset(dimensions);

				let position = m.get_start_position()?;

				self.dig(&mut m, position)?;
			},
			Err(e) => {
				self.show_error(e.to_string());
			},
		}

		if let Some(ref channel) = self.to_ui_tx
		{
			channel.send(UIRequest::ShowMaze(self.maze.clone())).unwrap_or_else(|_| return);
			channel.send(UIRequest::Quit).unwrap_or_else(|_| return);
		}
		Ok(())
	}

	fn dig(&self, maze: &mut MutexGuard<Maze>, position: usize) -> Result<(), AppError>
	{
		debug!("Checking if digging possible at position {}", position);

		// generate ranndom order of directions to try for this cell
		let mut rng = rand::thread_rng();
		let mut directions = Direction::get_directions();
		directions.shuffle(&mut rng);

		for direction in directions.iter()
		{
			match maze.is_diggable(position, *direction)
			{
				Ok(result) => {
					if result == true
					{
						debug!("Digging new passage towards {}", direction);
						let new_position = maze.dig_passage(position, *direction)?;
						debug!("Moving to new position {}", new_position);
						self.dig(maze, new_position)?; // recurse into digging the next position
					}
					else
					{
						debug!("Can't dig to {}", direction);
					}
				},
				Err(e) => {
					debug!("Can't dig to {}, error: {}", direction, e.to_string());
				}
			}
		}
		debug!("Stepping back from {}", position);
		Ok(())
	}

	/// Solve an already generated maze
	///
	/// Find a path through the maze
	fn solve_maze(&self)
	{
		self.show_error("Solving a maze is not yet implemented".to_string());
		if let Some(ref channel) = self.to_ui_tx
		{
			channel.send(UIRequest::Quit).unwrap_or_else(|_| return);
		}
	}
}
