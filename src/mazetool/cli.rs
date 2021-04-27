// Mazetool - command line user interface

use std::env;
use std::io;
use std::io::Write;
use std::sync::mpsc::*;
use std::sync::{ Arc, Mutex };
use std::thread;

use mazetool::userinterface::UserInterface;
use mazetool::common::ApplicationError;
use mazetool::common::{ UIRequest, Job };
use mazetool::maze::{ Dimensions, Maze, MAZE_DIMENSION_MIN, MAZE_DIMENSION_MAX, MAZE_DIMENSION_DEFAULT };

/// Command line user interface for Mazetool
pub struct CommandLineInterface
{
}

impl CommandLineInterface
{
	/// Print help text of program's command line argument usage
	fn print_usage(&self, program: &str)
	{
		println!("Usage: {} <command> [options]", program);
		println!("Run the program in the directory containing the database.");
		println!("");
		println!("Commands:");
		println!("  generate        Generate a new random maze of given size");
		println!("  solve           Solve a given maze");
		println!("  help            Print this help");
	}

	/// Get a food item from user input
	//fn get_maze_parameters(&self) -> Result<FoodItem, ApplicationError>
	//{
	//	let mut item = FoodItem::new(0, "Coca-Cola", 100, 137);
	//	println!("Enter information for new food item");
	//	match self.read_string("Name")
	//	{
	//		Ok(value) => item.name = value,
	//		Err(x) => return Err(x),
	//	}
	//	match self.read_integer_value("Weight (g)")
	//	{
	//		Ok(value) => item.weight = value as u16,
	//		Err(x) => return Err(x),
	//	}
	//	match self.read_integer_value("Energy (kJ)")
	//	{
	//		Ok(value) => item.energy = value as u16,
	//		Err(x) => return Err(x),
	//	}
	//	Ok(item)
	//}

	/// Read a trimmed string from the user input
	fn read_string(&self, prompt: &str) -> Result<String, ApplicationError>
	{
		let mut input_text = String::new();
		print!("{}: ", prompt);
		io::stdout().flush()?;
		io::stdin().read_line(&mut input_text)?;
		let trimmed = input_text.trim();
		Ok(trimmed.to_string())
	}

	/// Read a integer value from the user input
	fn read_integer_value(&self, prompt: &str) -> Result<u32, ApplicationError>
	{
		let input = match self.read_string(prompt)
		{
			Ok(x) => x,
			Err(e) => return Err(e),
		};
		match input.parse::<u32>()
		{
			Ok(value) => return Ok(value),
			Err(_) => return Err(ApplicationError::new("Input was not an integer")),
		};
	}

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
						print!("{}", m.cells[i*j].celltype);
					}
					println!("");
				}
			},
			Err(e) => {
				self.show_error(&e.to_string());
			},
		}
	}

	/// Handle a single request from the controller
	///
	/// # Parameters
	///
	/// * `tx`      - Channel for giving Jobs to the controller
	/// * `rx`      - Channel to receive UI requests from controller
	///
	/// # Returns
	///
	/// * `bool`    - True, if UI thread should keep running
	///
	fn handle_request(&self, tx: &Sender<Job>, rx: &Receiver<UIRequest>) -> bool
	{
		let mut keep_running = true;
		let request = rx.recv().unwrap_or_else(|_| UIRequest::Quit);
		info!("UI received request: {:?}", request);
		match request
		{
			UIRequest::ParseArgs => {
				keep_running = self.parse_args(tx);
			},
			UIRequest::ShowError(message) => {
				self.show_error(&message);
			},
			UIRequest::ShowInfo(message) => {
				self.show_info(&message);
			},
			UIRequest::ShowMaze(maze) => {
				self.show_maze(maze);
				keep_running = false;
			},
			UIRequest::Quit => {
				keep_running = false;
			},
		};

		if keep_running == false
		{
			info!("UI thread exiting");
		}

		return keep_running;
	}

	fn parse_dimension(&self, arg: &str, out: &mut usize) -> bool
	{
		let mut ret = true;

		match arg.parse::<usize>()
		{
			Ok(n) => *out = n,
			Err(_) => {
				self.show_error("Invalid parameters");
				ret = false;
			}
		}

		if *out > MAZE_DIMENSION_MAX && *out < MAZE_DIMENSION_MIN
		{
			ret = false;
		}

		return ret;
	}
}


impl UserInterface for CommandLineInterface
{
	/// Create new command line user interface instance
	fn new() -> Self
	{
		CommandLineInterface
		{
		}
	}

	/// Parse command line arguments
	fn parse_args(&self, tx: &Sender<Job>) -> bool
	{
		info!("Parsing command line arguments");

		let args: Vec<String> = env::args().collect();
		let program = &args[0];

		if args.len() < 2
		{
			self.print_usage(program);
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
					info!("Using default size");
				}
				else if args.len() == 4
				{
					info!("Parsing dimensions from command line parameteres");
					if !self.parse_dimension(&args[2], &mut dimensions.width)
					{
						info!("Parsing maze width failed");
						return false;
					}
					if !self.parse_dimension(&args[3], &mut dimensions.height)
					{
						info!("Parsing maze height failed");
						return false;
					}
				}
				else
				{
					self.show_error("Invalid parameters");
					self.print_usage(program);
					return false;
				}
				tx.send(Job::GenerateMaze(dimensions)).unwrap_or_else(|_| return);
			},
			"solve" => {
				info!("Solve requested");
				println!("Solve not yet implemented");

				//TODO: get maze filename? (optional?)

				return false;
			},
			"help" | _ => {
				self.print_usage(program);
				return false;
			},
		}

		return true;
	}

	fn run(tx: Sender<Job>, rx: Receiver<UIRequest>) -> thread::JoinHandle<()>
	{
		//let handle = thread::spawn(move || {
		let builder = thread::Builder::new().name("UserInterface".to_string());
		let handle: thread::JoinHandle<_> = builder.spawn(move || {
			info!("UI thread starting...");
			let cli = CommandLineInterface::new();

			loop
			{
				if cli.handle_request(&tx, &rx) != true
				{
					break;
				}
			}
		}).unwrap();

		return handle;
	}
}
