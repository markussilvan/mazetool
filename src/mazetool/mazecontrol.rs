//! Mazetool application control
//!
//! Implements the application logic.
//! Supports different user interface implementations.

use std::sync::{ Arc, Mutex, MutexGuard };
use std::thread;
use std::result::Result;

use crossbeam::channel::{Receiver, Sender};
use rand::seq::SliceRandom;

use super::common::{ UIRequest, Job, AppError };
use super::maze::{ Direction, Dimensions, Maze };

/// A class for main logic (controller)
///
/// Interact with user through a UserInterface implementation.
pub struct MazeControl
{
	tx: Sender<UIRequest>,
	maze: Arc<Mutex<Maze>>,
	running: bool,
}

impl MazeControl
{
	/// Creates a new MazeControl instance.
	pub fn new(tx : Sender<UIRequest>) -> Self
	{
		let mc = MazeControl
		{
			tx: tx,
			maze: Arc::new(Mutex::new(Maze::new())),
			running: false,
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
	pub fn run(rx: Receiver<Job>, tx : Sender<UIRequest>) -> thread::JoinHandle<()>
	{
		let thread_tx = tx.clone();

		let builder = thread::Builder::new().name("Control".to_string());
		let handle: thread::JoinHandle<_> = builder.spawn(move || {
			let mut mc = MazeControl::new(thread_tx);
			info!("Starting control thread");
			mc.run_message_loop(&rx);
			info!("Exiting control thread");
		}).unwrap();

		info!("Main thread continues");

		return handle
	}

	fn run_message_loop(&mut self, rx: &Receiver<Job>)
	{
		self.running = true;

		while self.running
		{
			match rx.recv().unwrap_or_else(|_| Job::Quit)
			{
				job => {
					info!("Control: Received job: {:?}", job);
					match job
					{
						Job::GenerateMaze(dimensions) => {
							self.tx.send(UIRequest::ShowInfo("Generating...".to_string()))
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
	}

	/// Send a job to the UI to show an error message
	///
	/// # Parameters
	///
	/// * `message`     - The error string
	///
	fn show_error(&self, message: String)
	{
		self.tx.send(UIRequest::ShowError(message)).unwrap();
	}

	/// Generate a new maze of the given size
	///
	/// A simple recursive backtracking algorithm.
	///
	/// 1. Close all cells
	/// 2. Choose starting cell and open it. This is the current cell
	/// 3. Pick a cell adjacent to the current cell that hasn’t been visited and open it.
	///    It becomes the current cell.
	/// 4. Repeat 2 until no adjacent wall can be selected
	/// 5. The previous cell becomes the current cell.
	///    If this cell is the starting cell, then we are done. Else go to 2.
	///
	/// # Parameters
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

				// generation could be started from any position, but we choose the start position
				let position = m.randomize_start_position();
				debug!("Start position: {}", position);

				self.dig(&mut m, position)?;
				m.insert_start_and_end_positions();
			},
			Err(e) => {
				self.show_error(e.to_string());
			},
		}

		self.tx.send(UIRequest::ShowMaze(self.maze.clone())).unwrap_or_else(|_| return);
		self.tx.send(UIRequest::SaveMaze(self.maze.clone())).unwrap_or_else(|_| return);
		self.tx.send(UIRequest::Quit).unwrap_or_else(|_| return);
		Ok(())
	}

	/// Recursively dig passages in the maze
	///
	/// # Parameters
	/// * `maze`        - The maze data structure
	/// * `position`    - Current position in the maze
	///
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
	fn solve_maze(&mut self)
	{
		self.show_error("Solving a maze is not yet implemented".to_string());
		self.quit();
	}

	fn quit(&mut self)
	{
		self.tx.send(UIRequest::Quit).unwrap_or_else(|_| return);
		self.running = false;
	}
}
