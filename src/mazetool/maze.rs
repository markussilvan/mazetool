use std::fmt::{ Display, Formatter };

pub const MAZE_DIMENSION_MIN : usize = 10;
pub const MAZE_DIMENSION_MAX : usize = 10000;
pub const MAZE_DIMENSION_DEFAULT : usize = 20;

/// Dimensions (width and height) of a maze
#[derive(Debug, Clone, Copy)]
pub struct Dimensions
{
	pub width: usize,
	pub height: usize,
}

/// Posibble states of one cell in a maze
#[derive(Debug, Clone)]
pub enum MazeCellType
{
	WALL,
	PASSAGE,
	START,
	END
}

impl Display for MazeCellType
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
		let c = match self {
			WALL => '█',
			PASSAGE => ' ',
			START => 'S',
			END => 'E',
			_ => '?',
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
        f.debug_struct("Maze {}");
		Ok(())
    }
}

impl Maze
{
	pub fn default() -> Maze
	{
		let default_cell = MazeCell { celltype: MazeCellType::WALL, visited: false };
		let maze = Maze {
			cells: vec![default_cell; MAZE_DIMENSION_DEFAULT * MAZE_DIMENSION_DEFAULT],
			dimensions: Dimensions {
				width: MAZE_DIMENSION_DEFAULT,
				height: MAZE_DIMENSION_DEFAULT
			},
		};

		return maze;
	}

	pub fn new(dim: Dimensions) -> Maze
	{
		let maze = Maze {
			cells: Vec::with_capacity(dim.width * dim.height),
			dimensions: dim,
		};
		return maze;
	}

	pub fn reset(&mut self, dimensions: Dimensions)
	{
		let new_size = dimensions.width * dimensions.height;

		self.dimensions = dimensions;

		if self.cells.len() != new_size
		{
			let default_cell = MazeCell { celltype: MazeCellType::WALL, visited: false };
			self.cells.resize(new_size, default_cell);
		}

		for i in 0..new_size
		{
			self.cells[i].celltype = MazeCellType::WALL;
			self.cells[i].visited = false;
		}

		debug!("Maze reset to new size: {} x {}, cells len: {}",
			   self.dimensions.width,
			   self.dimensions.height,
			   self.cells.len());
	}
}
