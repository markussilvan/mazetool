// Mazetool - graphical user interface with ggez

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
use super::maze::{ Maze, MazeCellType };

struct ShowMazeState
{
	maze: Arc<Mutex<Maze>>,
	screen: Rect,
	block_size: f32,
	error_text: Option<String>,
	show_distances: bool,
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
			error_text: None,
			show_distances: false,
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
	}

	fn set_maze(&mut self, maze: Arc<Mutex<Maze>>)
	{
		self.maze = maze.clone();
	}

	fn set_show_distances(&mut self, show_distances: bool)
	{
		self.show_distances = show_distances;
	}

	fn draw_text(&self, ctx: &mut Context, text_str: &String, pos_x: f32, pos_y: f32)
	{
		let mut text =  graphics::Text::new(format!("{}", text_str));
		text.set_font(graphics::Font::default(), graphics::PxScale::from(24.0));
		let params = graphics::DrawParam::default()
			.dest([pos_x, pos_y])
			.color(graphics::Color::YELLOW);

		graphics::draw(ctx, &text, params).expect("Error drawing text");
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
		                                         rect,
		                                         Color::WHITE)?;
		let route = graphics::Mesh::new_rectangle(ctx,
		                                          graphics::DrawMode::fill(),
		                                          rect,
		                                          Color::GREEN)?;
		let visited = graphics::Mesh::new_rectangle(ctx,
		                                            graphics::DrawMode::fill(),
		                                            rect,
		                                            Color {r: 0.0, g: 0.5, b: 0.5, a: 1.0 })?;
		let node = graphics::Mesh::new_circle(ctx,
		                                      graphics::DrawMode::fill(),
		                                      Vec2::new(0.0, 0.0),
		                                      self.block_size / 3.0,
		                                      2.0,
		                                      Color::GREEN)?;

		graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

		if let Ok(m) = self.maze.lock()
		{
			for x in 0..m.dimensions.width
			{
				for y in 0..m.dimensions.height
				{
					let cell = &m.cells[x + (y * m.dimensions.width)];
					let pos_x = x as f32 * self.block_size;
					let pos_y = y as f32 * self.block_size;

					// draw maze walls
					if cell.celltype == MazeCellType::Wall
					{
						graphics::draw(ctx, &wall, (Vec2::new(pos_x, pos_y),))?;
					}
					if cell.on_route
					{
						graphics::draw(ctx, &route, (Vec2::new(pos_x, pos_y),))?;
					}
					else if cell.visited
					{
						graphics::draw(ctx, &visited, (Vec2::new(pos_x, pos_y),))?;
					}
					if self.show_distances && (cell.celltype == MazeCellType::Passage)
					{
						self.draw_text(ctx, &cell.text, pos_x, pos_y);
					}

					// draw maze topology graph nodes
					for i in 0..cell.nodes.len()
					{
						if let Some(_) = cell.nodes[i]
						{
							graphics::draw(ctx, &node, (Vec2::new(pos_x + self.block_size / 2.0,
							                                      pos_y + self.block_size / 2.0),))?;
							break;
						}
					}
				}
			}

			// draw maze topology graph connections, if any
			if m.graph_created == true
			{
				for (px, py, x, y, _cell) in m.into_iter()
				{
					debug!("Maze graph iterator returned x = {}, y = {}", x, y);
					let pos_x = x as f32 * self.block_size + (self.block_size / 2.0);
					let pos_y = y as f32 * self.block_size + (self.block_size / 2.0);
					let prev_x = px as f32 * self.block_size + (self.block_size / 2.0);
					let prev_y = py as f32 * self.block_size + (self.block_size / 2.0);

					if (prev_x != pos_x) || (prev_y != pos_y)
					{
						let points = &[Vec2::new(prev_x, prev_y), Vec2::new(pos_x, pos_y)];
						let mut line_width = self.block_size / 10.0;
						if line_width < 0.6
						{
							line_width = 0.6;
						}
						let connection = graphics::Mesh::new_line(ctx,
						                                          points,
						                                          line_width,
						                                          Color::GREEN)?;
						graphics::draw(ctx, &connection, (Vec2::new(0.0, 0.0),))?;
					}
					else
					{
						info!("Error drawing connections, previous is 0.0");
					}
				}
			}
		}

		// draw error text, if any
		if let Some(error_str) = &self.error_text
		{
			let mut text =  graphics::Text::new(format!("Error: {}", error_str));
			text.set_font(graphics::Font::default(), graphics::PxScale::from(72.0));
			let pos_x = self.screen.w / 2.0 - text.width(ctx) as f32 / 2.0;
			let pos_y = 200.0;
			let params = graphics::DrawParam::default()
				.dest([pos_x, pos_y])
				.color(graphics::Color::RED);


			// draw a white background behind the text
			let rect = graphics::Rect::new(0.0, 0.0, text.width(ctx), text.height(ctx));
			let wall = graphics::Mesh::new_rectangle(ctx,
			                                         graphics::DrawMode::fill(),
			                                         rect, Color::WHITE)?;
			graphics::draw(ctx, &wall, (Vec2::new(pos_x, pos_y),))?;
			graphics::draw(ctx, &text, params).expect("Error drawing text");
		}

		graphics::present(ctx)?;
		Ok(())
	}
}

/// Graphical user interface for Mazetool
pub struct GraphicalInterface
{
	#[allow(dead_code)]
	tx: Sender<Job>,
	rx: Receiver<UIRequest>,
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

	fn run(&mut self, show_distances: bool)
	{
		let window_mode = ggez::conf::WindowMode::default()
			.dimensions(1920.0, 1080.0)
			.fullscreen_type(ggez::conf::FullscreenType::True);

		let window_setup = ggez::conf::WindowSetup {
                               title: "Mazetool".to_owned(),
                               samples: ggez::conf::NumSamples::One,
                               vsync: true,
                               icon: "".to_owned(),
                               srgb: true,
		};

		let cb = ggez::ContextBuilder::new("Mazetool", "Mape")
			.window_mode(window_mode)
			.window_setup(window_setup);
	    
		let (mut ctx, event_loop) = cb.build().unwrap();
		let mut state = ShowMazeState::new().unwrap();
		let rx_clone = self.rx.clone();
		let screen = ggez::graphics::screen_coordinates(&ctx);

		// Handle events. Refer to `winit` docs for more information.
		event_loop.run(move |mut event, _window_target, control_flow|
		{
			state.set_screen_size(screen);
			state.set_show_distances(show_distances);
			if !ctx.continuing
			{
				*control_flow = ControlFlow::Exit;
			}

			if let Ok(request) = rx_clone.try_recv()
			{
				info!("UI received request: {:?}", request);
				match request
				{
					UIRequest::ShowError(message) => {
						state.error_text = Some(message);
					},
					UIRequest::ShowInfo(_message) => {
						//state.error_text = Some(message);
					},
					UIRequest::ShowMaze(maze) => {
						state.set_maze(maze);
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
	}

}
