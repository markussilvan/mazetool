//! Mazetool application control
//!
//! Implements the application logic.
//! Supports different user interface implementations.

use std::sync::mpsc;
use std::marker::PhantomData;
use std::sync::{ Arc, Mutex };

use rand::prelude::*;

use mazetool::userinterface::UserInterface;
use mazetool::common::{ UIRequest, Job };
use mazetool::maze::{ Dimensions, Maze, MazeCellType };

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
							self.generate_maze(dimensions);
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

	fn generate_maze(&mut self, dimensions: Dimensions)
	{
		info!("Request to generate a maze received");

		match self.maze.lock()
		{
			Ok(mut m) => {
				m.reset(dimensions);

				//TODO: implementation to generate a maze
				for i in 0..m.dimensions.height
				{
					for j in 0..m.dimensions.width
					{
						if m.cells[j + (i * m.dimensions.width)].celltype == MazeCellType::Start
						{
							debug!("lol, start found - continue from here");
						}
					}
				}

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
