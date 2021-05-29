use std::fmt::{ Display, Formatter };
use std::result::Result;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;

use rand::prelude::*;

use super::common::AppError;

pub const NUM_OF_DIRECTIONS: usize = 4;
pub const MAZE_DIMENSION_MIN : usize = 10;
pub const MAZE_DIMENSION_MAX : usize = 10000;
pub const MAZE_DIMENSION_DEFAULT : usize = 19;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction
{
	North,
	East,
	South,
	West
}

impl Direction
{
	pub fn get_directions() -> [Direction; NUM_OF_DIRECTIONS]
	{
		let directions: [Direction; NUM_OF_DIRECTIONS] = [ Direction::North,
		                                                   Direction::East,
		                                                   Direction::South,
		                                                   Direction::West ];
		return directions;
	}

	pub fn get_opposite_direction(&self) -> Direction
	{
		match self
		{
			Direction::North => Direction::South,
			Direction::East => Direction::West,
			Direction::South => Direction::North,
			Direction::West => Direction::East,
		}
	}

//	pub fn from_u8(value: u8) -> Direction
//	{
//		match value
//		{
//			0 => Direction::North,
//			1 => Direction::East,
//			2 => Direction::South,
//			3 | _ => Direction::West,
//		}
//	}
}

impl Display for Direction
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
		let c = match &*self {
			Direction::North => "North",
			Direction::East => "East",
			Direction::South => "South",
			Direction::West => "West",
		};
        write!(f, "{}", c)
    }
}

/// Dimensions (width and height) of a maze
#[derive(Debug, Clone, Copy)]
pub struct Dimensions
{
	pub width: usize,
	pub height: usize,
}

/// Posibble states of one cell in a maze
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MazeCellType
{
	Wall,
	Passage,
	Start,
	End
}

impl Display for MazeCellType
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
		let c = match &*self {
			MazeCellType::Wall => '█',
			MazeCellType::Passage => ' ',
			MazeCellType::Start => 'S',
			MazeCellType::End => 'E',
		};
        write!(f, "{}", c)
    }
}

impl FromStr for MazeCellType
{
	type Err = AppError;
	fn from_str(hex_code: &str) -> Result<Self, Self::Err> {
		//TODO: proper implementation for MazeCellType
        let celltype: u8 = u8::from_str_radix(&hex_code[0..1], 16)?;
        Ok(MazeCellType::Wall)

        // u8::from_str_radix(src: &str, radix: u32) converts a string
        // slice in a given base to u8
        //let r: u8 = u8::from_str_radix(&hex_code[1..3], 16)?;
        //let g: u8 = u8::from_str_radix(&hex_code[3..5], 16)?;
        //let b: u8 = u8::from_str_radix(&hex_code[5..7], 16)?;

        //Ok(RGB { r, g, b })
    }
}

/// One cell of a maze
#[derive(Debug, Clone)]
pub struct MazeCell
{
	pub celltype: MazeCellType,
	pub visited: bool,
}

impl Display for MazeCell
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
        write!(f, "{}-{}", self.celltype, self.visited)
    }
}

/// The maze data structure
#[derive(Clone)]
pub struct Maze
{
	pub dimensions: Dimensions,
	pub cells: Vec<MazeCell>,
}

impl std::fmt::Debug for Maze
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
		f.debug_struct("Maze {}").finish()
    }
}

impl Maze
{
	/// Create a new maze structure
	pub fn new() -> Maze
	{
		let default_cell = MazeCell { celltype: MazeCellType::Wall, visited: false };
		let maze = Maze {
			cells: vec![default_cell; MAZE_DIMENSION_DEFAULT * MAZE_DIMENSION_DEFAULT],
			dimensions: Dimensions {
				width: MAZE_DIMENSION_DEFAULT,
				height: MAZE_DIMENSION_DEFAULT
			},
		};

		return maze;
	}

	fn parse_header_line(&self, header: &String) -> Result<Dimensions, AppError>
	{
		let mut dimensions = Dimensions { width: 0, height: 0 };
		let mut offset: usize = 0;
		let radix = 10;

		// parse "Maze" text
		if header[offset..5] == *"Maze "
		{
			offset += 5;
		}
		else
		{
			return Err(AppError::new("Error reading maze file header"));
		}

		// parse width
		match header[offset..].chars().position(|c| c == ' ')
		{
			Some(n) => {
				dimensions.width = usize::from_str_radix(&header[offset..offset+n], radix)?;
				offset += n + 1;
				debug!("Parsed width {}", dimensions.width);
			},
			None => return Err(AppError::new("Error parsing maze width from file header")),
		}

		// parse height
		dimensions.height = usize::from_str_radix(&header[offset..], radix)?;
		debug!("Parsed height {}", dimensions.height);

		Ok(dimensions)
	}

	/// Read a maze from a file
	///
	/// Maze is read from a file to this instance of Maze, and
	/// will overwrite any data already in this Maze.
	///
	/// # Parameters
	///
	/// * `filename`        - Source filename for loading the maze
	///
	/// Returns AppError on failure.
	///
	pub fn read_from_file(&self, filename: &str) -> Result<(), AppError>
	{
		let path = Path::new(filename);
		let display = path.display();
		let file = match File::open(&path)
		{
			Err(e) => {
				let error = format!("Couldn't open maze file {}: {}", display, e);
				return Err(AppError::new(&error));
			},
			Ok(file) => file,
		};
		let mut lines = io::BufReader::new(file).lines();   // io::Lines<io::BufReader<File>>

		println!("Maze read from file");
		if let Some(Ok(header)) = lines.next()
		{
			self.parse_header_line(&header)?;
		}

		//TODO: parse the data instead of just printing it
		for line in lines
		{
			if let Ok(l) = line
			{
				for c in l.chars()
				{
					//MazeCellType::from_str(&l[..1]);
					//let foo = MazeCellType::from_str(c);
					//TODO: from_str()
					print!("{}", c);
				}

				println!("");
			}
		}
		Ok(())
	}

	/// Save an already generated maze to a file
	///
	/// # Parameters
	/// 
	/// * `filename`        - Target filename for saving the maze
	///
	/// Returns AppError on failure.
	///
	pub fn write_to_file(&self, filename: &str) -> Result<(), AppError>
	{
		let path = Path::new(filename);
		let display = path.display();

		let mut file = match File::create(&path)
		{
			Err(e) => {
				let error = format!("Couldn't create maze file {}: {}", display, e);
				return Err(AppError::new(&error));
			},
			Ok(file) => file,
		};

		match writeln!(file, "Maze {} {}", self.dimensions.width, self.dimensions.height)
		{
			Err(e) => return Err(AppError::new(format!("Error writing maze: {}", e).as_str())),
			Ok(_) => {}
		}

		for i in 0..self.dimensions.height
		{
			for j in 0..self.dimensions.width
			{
				match write!(file, "{}", self.cells[j + (i * self.dimensions.width)].celltype)
				{
					Err(e) => return Err(AppError::new(format!("Error writing maze: {}", e).as_str())),
					Ok(_) => {}
				}
			}
			match writeln!(file, "")
			{
				Err(e) => return Err(AppError::new(format!("Error writing maze: {}", e).as_str())),
				Ok(_) => {}
			}
		}

		return Ok(())
	}

	/// Reset a maze by clearing it content and resize it
	/// to new dimensions if needed.
	///
	/// # Parameters
	///
	/// * `dimensions`      - New dimensions to set for the maze
	///
	pub fn reset(&mut self, dimensions: Dimensions)
	{
		let new_size = dimensions.width * dimensions.height;

		self.dimensions = dimensions;

		if self.cells.len() != new_size
		{
			let default_cell = MazeCell { celltype: MazeCellType::Wall, visited: false };
			self.cells.resize(new_size, default_cell);
		}

		for i in 0..new_size
		{
			self.cells[i].celltype = MazeCellType::Wall;
			self.cells[i].visited = false;
		}

		debug!("Maze reset to new size: {} x {}, cells len: {}",
			   self.dimensions.width,
			   self.dimensions.height,
			   self.cells.len());
	}

	/// Test if the given position in the Maze is diggable or not
	/// to the given direction.
	///
	/// # Parameters
	///
	/// * `position`        - Position from the maze to test
	/// * `direction`       - Direction of digging to test
	///
	/// Returns a boolean value.
	///
	pub fn is_diggable(&mut self,
	                   position: usize,
	                   direction: Direction
	) -> Result<bool, AppError>
	{
		let intermediate_position: usize = self.get_neighboring_position(position, direction)?;
		let new_position: usize = self.get_neighboring_position(intermediate_position, direction)?;

		// check the actual position is diggable (if it is, then also the intermediate is
		if !self.is_wall_or_end_position(new_position)
		{
			return Ok(false);
		}

		debug!("Position: {}, new position: {}, direction: {}", position, new_position, direction);

		// check all (other) positions around it (they must walls, or the end, all around)
		let mut directions: Vec<Direction> = Direction::get_directions().iter().cloned().collect();
		let opposite_direction = direction.get_opposite_direction();
		if let Some(pos) = directions.iter().position(|x| *x == opposite_direction)
		{
			directions.remove(pos);
		}
		else
		{
			return Err(AppError::new("Error while handling directions"));
		}

		// check "sides" or "corners" of the new position and the test_position is also "diggable"
		if self.are_sides_diggable(new_position, direction)
		{
			for test_direction in directions.iter()
			{
				let test_position = self.get_neighboring_position(new_position, *test_direction)?;

				if !self.is_wall_or_end_position(test_position)
				{
					debug!("Neighboring position {} is not a Wall or the End", test_position);
					return Ok(false);
				}
			}
			return Ok(true);
		}

		return Ok(false);
	}

	/// Dig a new passage to the maze.
	///
	/// # Parameters
	///
	/// * `position`        - Starting position for the digging
	/// * `direction`       - Direction of digging
	///
	/// Returns the new position where the digging ended.
	/// That is two cells towards the given direction from the stating position.
	///
	pub fn dig_passage(&mut self,
	                   position: usize,
	                   direction: Direction
	) -> Result<usize, AppError>
	{
		let intermediate_position: usize = self.get_neighboring_position(position, direction)?;
		let new_position: usize = self.get_neighboring_position(intermediate_position, direction)?;

		if self.cells[intermediate_position].celltype != MazeCellType::Wall ||
		   !self.is_wall_or_end_position(new_position)
		{
			let error = format!("Trying to dig something foul (positions: {}, {}) (types: {}, {})",
			                    intermediate_position,
			                    new_position,
			                    self.cells[intermediate_position].celltype,
			                    self.cells[new_position].celltype);
			return Err(AppError::new(error.as_str()));
		}

		self.cells[intermediate_position].celltype = MazeCellType::Passage;
		if self.cells[new_position].celltype != MazeCellType::End
		{
			self.cells[new_position].celltype = MazeCellType::Passage;
		}

		return Ok(new_position);
	}

	/// Randomize the starting point for the maze generation.
	///
	/// Returns the randomized starting position.
	pub fn randomize_start_position(&mut self) -> usize
	{
		let position = self.randomize_position_from_row(1);
		self.cells[position].celltype = MazeCellType::Passage;
		return position;
	}

	/// Insert start and end cells to a maze
	pub fn insert_start_and_end_positions(&mut self)
	{
		let start_pos = self.randomize_position_from_row(0);
		let end_pos = self.randomize_position_from_row(self.dimensions.height - 1);

		self.cells[start_pos].celltype = MazeCellType::Start;
		self.cells[end_pos].celltype = MazeCellType::End;
	}

	fn is_wall_or_end_position(&mut self, position: usize) -> bool
	{
		if ![MazeCellType::Wall, MazeCellType::End].contains(&self.cells[position].celltype)
		{
			return false;
		}
		return true;
	}

	fn get_neighboring_position(&mut self,
	                            position: usize,
	                            direction: Direction
	) -> Result<usize, AppError>
	{
		let len = self.dimensions.width * self.dimensions.height;

		match direction
		{
			Direction::North => {
				if position > self.dimensions.width
				{
					return Ok(position - self.dimensions.width);
				}
			},
			Direction::East => {
				if ((position + 1) < len) && ((position + 1) % self.dimensions.width != 0)
				{
					return Ok(position + 1);
				}
			},
			Direction::South => {
				if (position + self.dimensions.width) < len
				{
					return Ok(position + self.dimensions.width);
				}
			},
			Direction::West => {
				if (position > 0) && (position % self.dimensions.width != 0)
				{
					return Ok(position - 1);
				}
			},
		};

		return Err(AppError::new("Invalid maze position encountered"));
	}

	fn are_sides_diggable(&mut self, position: usize, direction: Direction) -> bool
	{
		// check "sides" or "corners" of the test_position are also "diggable"
		let mut sides: [usize; 2] = [0, 0];
		let mut doable = false;

		if direction == Direction::North || direction == Direction::South
		{
			if let Ok(pos) = self.get_neighboring_position(position, Direction::East)
			{
				sides[0] = pos;
			}
			if let Ok(pos) = self.get_neighboring_position(position, Direction::West)
			{
				sides[1] = pos;
			}
		}
		else
		{
			if let Ok(pos) = self.get_neighboring_position(position, Direction::North)
			{
				sides[0] = pos;
			}
			if let Ok(pos) = self.get_neighboring_position(position, Direction::South)
			{
				sides[1] = pos;
			}
		}

		if self.is_wall_or_end_position(sides[0]) &&
		   self.is_wall_or_end_position(sides[1])
		{
			doable = true;
		}

		return doable;
	}

	fn randomize_position_from_row(&self, row: usize) -> usize
	{
		let mut rng = rand::thread_rng();
		let mut position: usize = rng.gen_range(1..self.dimensions.width - 1);

		if position % 2 == 0
		{
			position = position - 1;
		}

		position = position + (row * self.dimensions.width);

		return position;
	}
}
