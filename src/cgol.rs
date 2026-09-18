//! # Rules
//! Let `n` be the number of live neighbors.
//! 1. If alive and `n != 2 && n != 3` then die; otherwise remain alive
//! 2. If dead and `n == 3` then become alive



/// Represents a cell in a [Grid].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Cell(bool);
impl Cell {
	pub fn new(alive: bool) -> Self {
		Self(alive)
	}

	pub fn is_alive(&self) -> bool {
		self.0
	}
}

/// A grid where Conway's Game of Life is played.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Grid{
	origin: (u64, u64),
	cells: Vec<Vec<Cell>>,
}

impl Grid {
	fn compute_cell(&self, x: u64, y: u64) -> Cell {
		let height = self.cells.len() as u64;
		let width = self.cells[0].len() as u64;

		let offsets: [(i64, i64); 8] = [
			(-1, -1), ( 0, -1), ( 1, -1),
			(-1,  0),           ( 1,  0),
			(-1,  1), ( 0,  1), ( 1,  1),
		];

		let cell = &self.cells[y as usize][x as usize];

		let mut n = 0;
		for (i, j) in offsets {
			// don't count self
			if i == 0 && j == 0{
				continue;
			}

			// ignore outside of canvas
			if (x == 0 && i < 0) || (x + 1 >= width && i > 0) {
				continue;
			}
			if (y == 0 && j < 0) || (y + 1 >= height && j > 0) {
				continue;
			}

			let other = &self.cells[y.checked_add_signed(j).unwrap() as usize][x.checked_add_signed(i).unwrap() as usize];
			if other.0 {
				n += 1;
			}
		}

		Cell(n == 3 || (cell.0 && n == 2))
	}

	fn top_extend(&mut self) {
		self.origin.1 += 1;

		let mut tmp = Vec::new();
		tmp.resize(self.cells[0].len(), Cell::default());
		self.cells.insert(0, tmp);
	}
	fn bottom_extend(&mut self) {
		let mut tmp = Vec::new();
		tmp.resize(self.cells[0].len(), Cell::default());
		self.cells.push(tmp);
	}
	fn left_extend(&mut self) {
		self.origin.0 += 1;

		for row in self.cells.iter_mut() {
			row.insert(0, Cell::default())
		}
	}
	fn right_extend(&mut self) {
		for row in self.cells.iter_mut() {
			row.push(Cell::default())
		}
	}

	fn prefit(&mut self) {
		if self.cells.len() <= 0 {
			return;
		}

		let edges = (
			self.cells[0].iter().any(|x| x.0),                       // top
			self.cells[self.cells.len() - 1].iter().any(|x| x.0),    // bottom
			self.cells.iter().map(|x| &x[0]).any(|x| x.0),           // left
			self.cells.iter().map(|x| &x[x.len() - 1]).any(|x| x.0), // right
		);

		// add edges if the current edge has any cells that're alive
		if edges.0 {
			self.top_extend();
		}
		if edges.1 {
			self.bottom_extend();
		}
		if edges.2 {
			self.left_extend();
		}
		if edges.3 {
			self.right_extend();
		}
	}

	/// Advence by a generation.
	pub fn advance(&mut self) {
		if self.cells.len() <= 0 {
			return;
		}
		self.prefit();

		let height = self.cells.len();
		let width = self.cells[0].len();

		// setup scratch buffer
		let mut scratch = {
			let mut row = Vec::new();
			row.resize(width, Cell::default());

			let mut out = Vec::new();
			out.resize(height, row);

			out
		};

		// actually compute then swap self data with scratch buffer
		for y in 0..height {
			for x in 0..width {
				scratch[y][x] = self.compute_cell(x as u64, y as u64);
			}
		}
		self.cells = scratch;
	}

	/// Set a cell's value at a given xy coordinate.
	pub fn set_cell (&mut self, x: u64, y: u64, value: Cell) {
		let x = x as usize;
		let y = y as usize;

		if self.cells.len() <= 0 {
			self.cells = vec![vec![Cell::default()]]
		}

		while self.cells.len() <= y {
			self.bottom_extend();
		}
		while self.cells[0].len() <= x {
			self.right_extend();
		}

		self.cells[y][x] = value;
	}

	/// Shrinks the table to be as small as possible (without loosing any data).
	pub fn shrink(&mut self) {
		if self.cells.len() <= 0 {
			return;
		}

		// top
		let mut count = 0;
		while count < self.cells.len() && !self.cells[count].iter().any(|x| x.0) {
			count += 1;
		}
		self.origin.1 -= count as u64;
		drop(self.cells.drain(..count));

		if self.cells.len() <= 0 {
			return;
		}

		// bottom
		let mut count = 0;
		while count < self.cells.len() && !self.cells[self.cells.len() - 1 - count].iter().any(|x| x.0) {
			count += 1;
		}
		drop(self.cells.drain(self.cells.len() - count..));

		if self.cells.len() <= 0 {
			return;
		}

		// right
		while !self.cells.iter().any(|row| row[0].0) {
			for row in self.cells.iter_mut() {
				drop(row.drain(..1));
				if row.len() <= 0 {
					self.cells.clear();
					return;
				}
			}
		}

		// left
		while !self.cells.iter().any(|row| row[row.len() - 1].0) {
			for row in self.cells.iter_mut() {
				self.origin.0 -= 1;
				row.pop();
			}
		}
	}

	/// Return the height.
	pub fn height(&self) -> u64 {
		return self.cells.len() as u64;
	}

	/// Return the width.
	pub fn width(&self) -> u64 {
		if self.cells.len() <= 0 {
			return 0;
		}

		return self.cells[0].len() as u64;
	}

	/// Create a new `Canvas` with an initial width and height.
	pub fn new(width: u64, height: u64) -> Self {
		let width = width as usize;
		let height = height as usize;

		let mut row = Vec::new();
		row.resize(width, Cell::default());

		let mut cells = Vec::new();
		cells.resize(height, row);

		Self{origin: (0, 0), cells}
	}

	/// Expand the grid, relative to the origin.
	pub fn set_bounds(&mut self, neg_x: u64, neg_y: u64, pos_x: u64, pos_y: u64) {
		while self.origin.0 < neg_x {
			self.left_extend();
		}

		while self.origin.1 < neg_y {
			self.top_extend();
		}

		while self.width() - self.origin.0 < pos_x {
			self.right_extend();
		}

		while self.height() - self.origin.1 < pos_y {
			self.bottom_extend();
		}
	}

	/// Get the origin point of the grid.
	pub fn get_origin(&self) -> (u64, u64) {
		self.origin
	}

	/// Get all of the cells that this grid manages.
	pub fn get_cells(&self) -> &'_ Vec<Vec<Cell>> {
		&self.cells
	}
}



#[test]
fn shrink() {
	let mut canvas = Grid::new(0, 0);
	canvas.set_cell(0, 0, Cell(false));
	canvas.shrink();

	assert_eq!(canvas.width(), 0);
	assert_eq!(canvas.height(), 0);

	let mut canvas = Grid::new(0, 0);
	canvas.set_cell(0, 0, Cell(true));
	canvas.shrink();

	assert_eq!(canvas.width(), 1);
	assert_eq!(canvas.height(), 1);

	let mut canvas = Grid::new(3, 3);
	canvas.set_cell(0, 1, Cell(true));
	canvas.set_cell(1, 1, Cell(true));
	canvas.set_cell(2, 1, Cell(true));
	canvas.shrink();

	assert_eq!(canvas.width(), 3);
	assert_eq!(canvas.height(), 1);
}


#[test]
fn blinker() {
	let mut canvas = Grid::new(3, 3);

	canvas.set_cell(0, 1, Cell(true));
	canvas.set_cell(1, 1, Cell(true));
	canvas.set_cell(2, 1, Cell(true));

	let mut init = canvas.clone();
	init.shrink();

	canvas.advance();
	canvas.shrink();

	let mut other = Grid::new(3, 3);
	other.set_cell(1, 0, Cell(true));
	other.set_cell(1, 1, Cell(true));
	other.set_cell(1, 2, Cell(true));
	other.shrink();

	assert_eq!(canvas, other);

	canvas.advance();
	canvas.shrink();
	assert_eq!(canvas, init);
}
