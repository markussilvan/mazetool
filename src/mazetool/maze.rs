use std::fmt::{ Display, Formatter };
use std::result::Result;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;
use std::cmp::Ordering;

use rand::prelude::*;
use heapless::binary_heap::{ BinaryHeap, Min };

use super::common::AppError;

pub const NUM_OF_DIRECTIONS: usize = 4;
pub const MAZE_DIMENSION_MIN: usize = 10;
pub const MAZE_DIMENSION_MAX: usize = 10000;
pub const MAZE_DIMENSION_DEFAULT: usize = 19;
pub const MAX_HEAP_SIZE: usize = 128;

#[derive(Clone, Copy)]
enum GraphNodeType
{
	NA,
	Straight,
	Intersection, // or a corner
	DeadEnd,
	End,
}

struct GraphNodeInfo
{
	position: usize,
	nodetype: GraphNodeType,
	directions: Vec<Direction>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction
{
	North,
	East,
	West,
	South
}

impl Direction
{
	pub fn get_directions() -> [Direction; NUM_OF_DIRECTIONS]
	{
		let directions: [Direction; NUM_OF_DIRECTIONS] = [ Direction::North,
		                                                   Direction::East,
		                                                   Direction::West,
		                                                   Direction::South ];
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

	pub fn remove_direction(directions: &mut Vec<Direction>, direction: Direction) -> bool
	{
		if let Some(i) = directions.iter().position(|x| *x == direction)
		{
			directions.remove(i);
			true
		}
		else
		{
			false
		}
	}

	pub fn from_usize(value: usize) -> Direction
	{
		match value
		{
			0 => Direction::North,
			1 => Direction::East,
			2 => Direction::West,
			3 | _ => Direction::South,
		}
	}
}

impl Display for Direction
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
		let c = match &*self {
			Direction::North => "North",
			Direction::East => "East",
			Direction::West => "West",
			Direction::South => "South",
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
        let _celltype: u8 = u8::from_str_radix(&hex_code[0..1], 16)?;
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
	pub on_route: bool,
	pub nodes: [Option<usize>; NUM_OF_DIRECTIONS],
}

impl Display for MazeCell
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
        write!(f, "{}-{}-{}", self.celltype, self.visited, self.on_route)
    }
}

/// The maze data structure
#[derive(Clone)]
pub struct Maze
{
	pub dimensions: Dimensions,
	pub cells: Vec<MazeCell>,
	pub start: usize,
	pub end: usize,
	pub graph_created: bool,
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
		let default_cell = MazeCell {
			celltype: MazeCellType::Wall,
			visited: false,
			on_route: false,
			nodes: [None; NUM_OF_DIRECTIONS]};
		let maze = Maze {
			cells: vec![default_cell; MAZE_DIMENSION_DEFAULT * MAZE_DIMENSION_DEFAULT],
			dimensions: Dimensions {
				width: MAZE_DIMENSION_DEFAULT,
				height: MAZE_DIMENSION_DEFAULT
			},
			start: 0,
			end: 0,
			graph_created: false,
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
			let default_cell = MazeCell {
				celltype: MazeCellType::Wall,
				visited: false,
				on_route: false,
				nodes: [None; NUM_OF_DIRECTIONS]};
			self.cells.resize(new_size, default_cell);
		}

		for i in 0..new_size
		{
			self.cells[i].celltype = MazeCellType::Wall;
			self.cells[i].visited = false;
			self.cells[i].on_route = false;
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
	pub fn is_diggable(&self,
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

		if !Direction::remove_direction(&mut directions, opposite_direction)
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

		self.start = start_pos;
		self.end = end_pos;
	}

	fn is_wall_or_end_position(&self, position: usize) -> bool
	{
		if ![MazeCellType::Wall, MazeCellType::End].contains(&self.cells[position].celltype)
		{
			return false;
		}
		return true;
	}

	fn get_neighboring_position(&self,
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
			Direction::West => {
				if (position > 0) && (position % self.dimensions.width != 0)
				{
					return Ok(position - 1);
				}
			},
			Direction::South => {
				if (position + self.dimensions.width) < len
				{
					return Ok(position + self.dimensions.width);
				}
			},
		};

		return Err(AppError::new("Invalid maze position encountered"));
	}

	fn are_sides_diggable(&self, position: usize, direction: Direction) -> bool
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

	fn get_neighbours(&self, position: usize) -> Vec<usize>
	{
		let directions: Vec<Direction> = Direction::get_directions().iter().cloned().collect();
		let mut neighbours: Vec<usize> = Vec::new();

		for test_direction in directions
		{
			if let Ok(pos) = self.get_neighboring_position(position, test_direction)
			{
				if self.cells[pos].celltype != MazeCellType::Wall
				{
					neighbours.push(pos);
				}
			}
		}

		neighbours
	}

	fn manhattan_distance(&self, x: usize, y: usize) -> usize
	{
		let mut v = 0;
		let mut h = 0;

		if x < y
		{
			v = (y - x) / self.dimensions.width;
			h = (y - x) % self.dimensions.width;
		}
		else if x > y
		{
			v = (x - y) / self.dimensions.width;
			h = (x - y) % self.dimensions.width;
		}

		return v + h;
	}

	pub fn run_a_star(&mut self, step: bool) -> bool
	{
		#[derive(Clone, Copy, Eq, PartialEq, Debug)]
		struct ListItem
		{
			position: usize,
			parent: usize,
			f: usize,
			g: usize,
			h: usize,
		}

		impl Ord for ListItem {
			fn cmp(&self, other: &Self) -> Ordering {
				if self.f == other.f
				{
					Ordering::Equal
				}
				else if self.f > other.f
				{
					Ordering::Greater
				}
				else
				{
					Ordering::Less
				}
			}
		}

		impl PartialOrd for ListItem {
			fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
				Some(self.cmp(other))
			}
		}

		static mut OPEN_LIST: BinaryHeap<ListItem, Min, MAX_HEAP_SIZE> = BinaryHeap::new();
		static mut CLOSED_LIST: Vec<ListItem> = Vec::new();

		let mut finished = false;

		unsafe
		{
			if OPEN_LIST.len() == 0
			{
				let start: ListItem = ListItem { position: self.start, parent: 0, f: 0, g: 0, h: 0 };
				match OPEN_LIST.push(start)
				{
					Ok(_) => {},
					Err(_) => {},
				}
				CLOSED_LIST.push(start);
			}

			while OPEN_LIST.len() > 0
			{
				let item = OPEN_LIST.pop().unwrap();

				self.cells[item.position].visited = true;

				let mut successors : Vec<ListItem> = Vec::new();
				for p in self.get_neighbours(item.position)
				{
					if p != item.parent
					{
						successors.push(ListItem { position: p, parent: item.position, f: 0, g: 0, h: 0 });
					}
				}

				while let Some(mut s) = successors.pop()
				{
					s.g = item.g + 1;
					s.h = self.manhattan_distance(s.position, self.end);
					s.f = s.g + s.h;
					s.parent = item.position;

					if self.cells[s.position].celltype == MazeCellType::End
					{
						self.cells[s.position].visited = true;
						CLOSED_LIST.push(s);

						// only one route through the maze, no need to continue
						OPEN_LIST.clear();
						break;
					}

					CLOSED_LIST.push(s);

					if let Some(_old) = OPEN_LIST.iter().find(|x| (x.position == s.position) && (x.f < s.f))
					{
						// skip, there is already a shorter way to get there
						continue;
					}

					if let Some(_old) = CLOSED_LIST.iter().find(|x| (x.position == s.position) && (x.f < s.f))
					{
						// skip, there is already a shorter way to get there
						continue;
					}

					OPEN_LIST.push(s).unwrap();
				}

				if step == true
				{
					break
				}
			}

			// if finished, mark the route (quick'n'dirty)
			if OPEN_LIST.len() == 0
			{
				let mut parent = CLOSED_LIST.last().unwrap().position;
				let mut found = true;
				while found && (parent != 0)
				{
					found = false;
					for item in CLOSED_LIST.iter().rev()
					{
						if item.position == parent
						{
							self.cells[item.position].on_route = true;
							parent = item.parent;
							found = true;
							break;
						}
					}
				}

				finished = true;
			}
		}

		finished
	}

	pub fn run_graph_elimination(&mut self, step: bool) -> bool
	{
		for i in self.dimensions.width..(self.cells.len() - self.dimensions.width)
		{
			// just for optimization, skip walls, start and end
			if self.cells[i].celltype != MazeCellType::Passage
			{
				continue;
			}

			let mut leaf = Some(i);
			while let Some(node) = leaf
			{
				if self.get_num_of_graph_connections(node) == 1
				{
					leaf = self.remove_dead_end(node);

					if step
					{
						info!("Graph elimination stepped");
						return true;
					}
				}
				else
				{
					leaf = None
				}
			}
		}

		info!("Graph elimination done");
		return false;
	}

	fn get_num_of_graph_connections(&mut self, position: usize) -> usize
	{
		let mut count = 0;

		for i in 0..NUM_OF_DIRECTIONS
		{
			if self.cells[position].nodes[i] != None
			{
				count += 1;
			}
		}

		count
	}

	fn remove_dead_end(&mut self, position: usize) -> Option<usize>
	{
		for i in 0..NUM_OF_DIRECTIONS
		{
			if self.cells[position].nodes[i] != None
			{
				if let Some(prev) = self.cells[position].nodes[i]
				{
					let opposite = Direction::from_usize(i).get_opposite_direction();
					self.cells[prev].nodes[opposite as usize] = None;
					self.cells[position].nodes[i] = None;
					return Some(prev);
				}
			}
		}
		None
	}

	/// Generate a topology graph of this maze.
	pub fn create_topology_graph(&mut self)
	{
		let mut stack: Vec<(usize, usize, Direction)> = Vec::new();

		// add start position to the stack (only way from the start is south)
		stack.push((self.start, self.start, Direction::South));

		while let Some((previous, position, direction)) = stack.pop()
		{
			let node_info = self.check_passage(position, direction);
			match node_info.nodetype
			{
				GraphNodeType::Straight => {
					stack.push((previous, node_info.position, direction));
				},
				GraphNodeType::Intersection => {
					for dir in node_info.directions.iter()
					{
						stack.push((node_info.position, node_info.position, *dir));
					}
					self.add_topology_node(previous, node_info.position, direction);
				},
				GraphNodeType::DeadEnd => {
					self.add_topology_node(previous, node_info.position, direction);
				},
				GraphNodeType::End => {
					self.add_topology_node(previous, node_info.position, direction);
					//break;
				},
				GraphNodeType::NA => {
					debug!("Internal error. Invalid maze position encountered {}", position);
					break;
				},
			}
		}

		self.graph_created = true;
	}

	fn check_passage(&self, position: usize, direction: Direction) -> GraphNodeInfo
	{
		let mut node_info = GraphNodeInfo {
			position: 0,
			nodetype: GraphNodeType::NA,
			directions: Vec::new()
		};

		if let Ok(pos) = self.get_neighboring_position(position, direction)
		{
			if self.cells[pos].celltype == MazeCellType::Passage
			{
				let opposite_direction = direction.get_opposite_direction();
				node_info.directions = self.get_possible_directions(pos, opposite_direction);

				match node_info.directions.len()
				{
					0 => {
						node_info.nodetype = GraphNodeType::DeadEnd;
					},
					1 => {
						if node_info.directions[0] == direction
						{
							node_info.nodetype = GraphNodeType::Straight;
						}
						else
						{
							// a corner
							node_info.nodetype = GraphNodeType::Intersection;
						}
					},
					_ => {
						node_info.nodetype = GraphNodeType::Intersection;
					},
				}
				node_info.position = pos;
			}
			else if self.cells[pos].celltype == MazeCellType::End
			{
				node_info.position = pos;
				node_info.nodetype = GraphNodeType::End;
			}
		}
		debug!("Topology: node_info position: {}, nodetype: {}, num directions: {}",
		       node_info.position,
		       node_info.nodetype as usize,
		       node_info.directions.len());
		return node_info;
	}

	// Get all possible directions to proceed
	// (not including the direction given as parameter)
	fn get_possible_directions(&self, position: usize, direction: Direction) -> Vec<Direction>
	{
		let mut directions: Vec<Direction> = Direction::get_directions().iter().cloned().collect();

		// remove incoming direction from directions
		if !Direction::remove_direction(&mut directions, direction)
		{
			debug!("Internal error. Removing incoming direction failed.");
		}

		let mut result = directions.clone();

		// check other directions
		for test_direction in directions
		{
			if let Ok(pos) = self.get_neighboring_position(position, test_direction)
			{
				if self.cells[pos].celltype == MazeCellType::Wall
				{
					Direction::remove_direction(&mut result, test_direction);
				}
			}
			else
			{
				Direction::remove_direction(&mut result, test_direction);
			}
		}

		result
	}

	fn add_topology_node(&mut self, start: usize, end: usize, direction: Direction)
	{
		debug!("Topology: adding node, start: {}, end: {}, direction: {}", start, end, direction);
		self.cells[start].nodes[direction as usize] = Some(end);
		self.cells[end].nodes[direction.get_opposite_direction() as usize] = Some(start);
	}
}

impl<'a> IntoIterator for &'a Maze {
	type Item = (usize, usize, usize, usize, &'a MazeCell);
	type IntoIter = MazeGraphIterator<'a>;

	fn into_iter(self) -> Self::IntoIter {
		let mut iter = MazeGraphIterator {
			maze: self,
			stack: Vec::new(),
		};

		// find start position
		for i in 0..self.dimensions.width
		{
			if self.cells[i].celltype == MazeCellType::Start
			{
				iter.stack.push((i, Direction::South)); // only way from the start is south
				break;
			}
		}

		iter
	}
}

pub struct MazeGraphIterator<'a>
{
	maze: &'a Maze,
	stack: Vec<(usize, Direction)>,
}

impl<'a> Iterator for MazeGraphIterator<'a>
{
	type Item = (usize, usize, usize, usize, &'a MazeCell);
	fn next(&mut self) -> Option<(usize, usize, usize, usize, &'a MazeCell)>
	{
		let mut new_position = 0;
		if let Some((position, direction)) = self.stack.pop()
		{
			debug!("Iterator: popped position {}, direction {}", position, direction);
			if let Some(pos) = self.maze.cells[position].nodes[direction as usize]
			{
				new_position = pos;
				for dir in Direction::get_directions()
				{
					if (self.maze.cells[pos].nodes[dir as usize] != None) &&
					   (dir != direction.get_opposite_direction())
					{
						self.stack.push((pos, dir));
					}
				}
			}

			let y = new_position / self.maze.dimensions.width;
			let x = new_position % self.maze.dimensions.width;
			let prev_y = position / self.maze.dimensions.width;
			let prev_x = position % self.maze.dimensions.width;

			return Some((prev_x, prev_y, x, y, &self.maze.cells[position]));
		}
		None
    }
}
