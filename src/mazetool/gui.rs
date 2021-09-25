// Mazetool - graphical user interface with Piston

use std::env;
use std::sync::{Arc, Mutex};

use crossbeam::channel::{Receiver, Sender};
use ggez::event;
use ggez::event::winit_event::{Event, KeyboardInput, WindowEvent};
use ggez::graphics::{self, Color, Rect};
use ggez::{Context, GameResult};
use winit::event_loop::ControlFlow;
use glam::*;

use super::userinterface::UserInterface;
use super::common::{ UIRequest, Job };
use super::maze::{ Dimensions, Maze, MazeCellType, MAZE_DIMENSION_MIN, MAZE_DIMENSION_MAX, MAZE_DIMENSION_DEFAULT };

struct ShowMazeState
{
	maze: Arc<Mutex<Maze>>,
	screen: Rect,
	block_size: f32,
}

impl ShowMazeState
{
	//fn new(maze: Arc<Mutex<Maze>>) -> GameResult<ShowMazeState>
	fn new() -> GameResult<ShowMazeState>
	{
		let s = ShowMazeState {
			maze: Arc::new(Mutex::new(Maze::new())), // this is replaced later by real data from Control
			screen: Rect { x: 0.0, y: 0.0, w: 0.0 , h: 0.0},
			block_size: 0.0,
		};
		Ok(s)
	}

	fn set_screen_size(&mut self, screen: Rect)
	{
		self.screen = screen;

		if let Ok(m) = self.maze.lock()
		{
			self.block_size = (std::cmp::min(self.screen.h as usize / m.dimensions.height,
			                                self.screen.w as usize / m.dimensions.width)) as f32;
		}
		//self.block_size = 10.0; //TODO: remove when real block size works...
	}

	fn handle_show_maze(&mut self, maze: Arc<Mutex<Maze>>)
	{
		self.maze = maze.clone();
	}

}

impl event::EventHandler<ggez::GameError> for ShowMazeState
{
	fn update(&mut self, _ctx: &mut Context) -> GameResult
	{
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult
	{

		let rect = graphics::Rect::new(0.0, 0.0, self.block_size, self.block_size);
		let wall = graphics::Mesh::new_rectangle(ctx,
		                                         graphics::DrawMode::fill(),
		                                         rect, Color::WHITE)?;

		graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

		if let Ok(m) = self.maze.lock()
		{
			for x in 0..m.dimensions.width
			{
				for y in 0..m.dimensions.height
				{
					if m.cells[x + (y * m.dimensions.width)].celltype == MazeCellType::Wall
					{
						let pos_x = x as f32 * self.block_size;
						let pos_y = y as f32 * self.block_size;
						graphics::draw(ctx, &wall, (Vec2::new(pos_x, pos_y),))?;
					}
				}
			}
		}

		graphics::present(ctx)?;
		Ok(())
	}
}

/// Graphical user interface for Mazetool
pub struct GraphicalInterface
{
	tx: Sender<Job>,
	rx: Receiver<UIRequest>,
}

impl GraphicalInterface
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

	/// Show an info message in the user interface
	///
	/// # Parameters
	///
	/// * `message`       - Information string to show
	///
	fn _show_info(&self, message: &str)
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


impl UserInterface for GraphicalInterface
{
	/// Create new command line user interface instance
	fn new(tx: Sender<Job>, rx: Receiver<UIRequest>) -> Self
	{
		GraphicalInterface
		{
			tx: tx,
			rx: rx,
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
				if args.len() != 2
				{
					info!("Invalid parameters");
					return false;
				}
				tx.send(Job::SolveMaze).unwrap_or_else(|_| return);
			},
			"help" | _ => {
				self.print_usage(program);
				return false;
			},
		}

		return true;
	}

	fn run(&mut self)
	{
		let mut window_mode = ggez::conf::WindowMode::default().dimensions(800.0, 600.0);
		window_mode.fullscreen_type = ggez::conf::FullscreenType::Desktop;
		//let mut running = true;
		let cb = ggez::ContextBuilder::new("Mazetool", "Mape")
			//.window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0));
			.window_mode(window_mode);
		let (mut ctx, event_loop) = cb.build().unwrap();
		let mut state = ShowMazeState::new().unwrap();
		let rx_clone = self.rx.clone();
		let screen = ggez::graphics::screen_coordinates(&ctx);
		ggez::graphics::set_window_title(&ctx, "Mazetool");

		//TODO: just do this here for now (for texting), refactor parse_args later
		let dimensions = Dimensions {
			//width: MAZE_DIMENSION_DEFAULT,
			//height: MAZE_DIMENSION_DEFAULT
			width: 39,
			height: 39
		};
		self.tx.send(Job::GenerateMaze(dimensions)).unwrap_or_else(|_| return);

		// Handle events. Refer to `winit` docs for more information.
		event_loop.run(move |mut event, _window_target, control_flow|
		{
			//TODO: create a local copy of maze here (remove it from the class)
			//      or is some arc mutex thing enough?
			if !ctx.continuing
			{
				*control_flow = ControlFlow::Exit;
			}

			if let Ok(request) = rx_clone.try_recv()
			{
				info!("UI received request: {:?}", request);
				match request
				{
					UIRequest::ParseArgs => {
						//keep_running = self.parse_args(&self.tx);
					},
					UIRequest::ShowError(_message) => {
						//self.show_error(&message);
					},
					UIRequest::ShowInfo(_message) => {
						//state.handle_show_maze(&message);
					},
					UIRequest::ShowMaze(maze) => {
						state.handle_show_maze(maze);
						state.set_screen_size(screen); //TODO: move this somewhere else?
					},
					UIRequest::Quit => {
						*control_flow = ControlFlow::Exit;
					},
				};
			}

			*control_flow = ControlFlow::Poll;

			let ctx = &mut ctx;

			// This tells `ggez` to update it's internal states, should the event require that.
			// These include cursor position, view updating on resize, etc.
			event::process_event(ctx, &mut event);
			match event
			{
				Event::WindowEvent { event, .. } => match event
				{
					WindowEvent::CloseRequested => event::quit(ctx),
					WindowEvent::KeyboardInput
					{
						input:
							KeyboardInput {
								virtual_keycode: Some(keycode),
								..
							},
							..
					} => {
						if let event::KeyCode::Escape = keycode {
							*control_flow = ControlFlow::Exit
						}
					}
					// `CloseRequested` and `KeyboardInput` events won't appear here.
					x => println!("Other window event fired: {:?}", x),
				},
				Event::MainEventsCleared => {
					// Tell the timer stuff a frame has happened.
					// Without this the FPS timer functions and such won't work.
					ctx.timer_context.tick();

					let eh : &mut dyn event::EventHandler<ggez::GameError> = &mut state;
					eh.update(ctx).unwrap();
					eh.draw(ctx).unwrap();

					ctx.mouse_context.reset_delta();

					ggez::timer::yield_now();
				}

				x => println!("Device event fired: {:?}", x),
			}
		});
		//TODO: move parse_args out of CommandLineInterface (to main)
	}

}
