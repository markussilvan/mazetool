use std::fmt::{ Display, Formatter };
use std::result::Result;

use rand::prelude::*;

use mazetool::common::AppError;

pub const NUM_OF_DIRECTIONS: usize = 4;
pub const MAZE_DIMENSION_MIN : usize = 10;
pub const MAZE_DIMENSION_MAX : usize = 10000;
pub const MAZE_DIMENSION_DEFAULT : usize = 20;

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

		self.randomize_start_and_end_positions();

		debug!("Maze reset to new size: {} x {}, cells len: {}",
			   self.dimensions.width,
			   self.dimensions.height,
			   self.cells.len());
	}

	pub fn is_diggable(&mut self,
	                   position: usize,
	                   direction: Direction
	) -> Result<bool, AppError>
	{
		let new_position: usize = self.get_neighboring_position(position, direction)?;

		// check the actual position is diggable
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

		let mut doable = false;

		for test_direction in directions.iter()
		{
			let test_position = self.get_neighboring_position(new_position, *test_direction)?;

			if !self.is_wall_or_end_position(test_position)
			{
				debug!("Neighboring position {} is not a Wall or the End", test_position);
				return Ok(false);
			}
			else
			{
				// check "sides" or "corners" of the test_position are also "diggable"
				let mut sides: [usize; 2] = [0, 0];
				if *test_direction == Direction::North || *test_direction == Direction::South
				{
					if let Ok(pos) = self.get_neighboring_position(test_position, Direction::East)
					{
						sides[0] = pos;
					}
					if let Ok(pos) = self.get_neighboring_position(test_position, Direction::West)
					{
						sides[1] = pos;
					}
				}
				else
				{
					if let Ok(pos) = self.get_neighboring_position(test_position, Direction::North)
					{
						sides[0] = pos;
					}
					if let Ok(pos) = self.get_neighboring_position(test_position, Direction::South)
					{
						sides[1] = pos;
					}
				}

				if self.is_wall_or_end_position(sides[0]) &&
				   self.is_wall_or_end_position(sides[1])
				{
					doable = true;
				}
			}
		}

		return Ok(doable);
	}

	pub fn dig_passage(&mut self,
	                   position: usize,
	                   direction: Direction
	) -> Result<usize, AppError>
	{
		let new_position: usize = self.get_neighboring_position(position, direction)?;

		if self.cells[new_position].celltype != MazeCellType::Wall
		{
			return Err(AppError::new("Trying to dig something else than a Wall"));
		}

		self.cells[new_position].celltype = MazeCellType::Passage;

		return Ok(new_position);
	}

	pub fn get_start_position(&self) -> Result<usize, AppError>
	{
		for i in 0..self.dimensions.height
		{
			for j in 0..self.dimensions.width
			{
				let pos: usize = j + (i * self.dimensions.width);
				if self.cells[pos].celltype == MazeCellType::Start
				{
					return Ok(pos);
				}
			}
		}
		return Err(AppError::new("Maze start position not found"));
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

	fn randomize_start_and_end_positions(&mut self)
	{
		let mut rng = rand::thread_rng();
		let start_pos: usize = rng.gen_range(0..self.dimensions.width);
		let end_pos: usize = rng.gen_range(0..self.dimensions.width) +
		                     (self.dimensions.width * (self.dimensions.height - 1));

		self.cells[start_pos].celltype = MazeCellType::Start;
		self.cells[end_pos].celltype = MazeCellType::End;
	}
}
