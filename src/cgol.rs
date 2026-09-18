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
pub struct Grid(Vec<Vec<Cell>>);
impl Grid {
	fn compute_cell(&self, x: u64, y: u64) -> Cell {
		let height = self.0.len() as u64;
		let width = self.0[0].len() as u64;

		let offsets: [(i64, i64); 8] = [
			(-1, -1), ( 0, -1), ( 1, -1),
			(-1,  0),           ( 1,  0),
			(-1,  1), ( 0,  1), ( 1,  1),
		];

		let cell = &self.0[y as usize][x as usize];

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

			let other = &self.0[y.checked_add_signed(j).unwrap() as usize][x.checked_add_signed(i).unwrap() as usize];
			if other.0 {
				n += 1;
			}
		}

		Cell(n == 3 || (cell.0 && n == 2))
	}

	fn top_extend(&mut self) {
		let mut tmp = Vec::new();
		tmp.resize(self.0[0].len(), Cell::default());
		self.0.insert(0, tmp);
	}
	fn bottom_extend(&mut self) {
		let mut tmp = Vec::new();
		tmp.resize(self.0[0].len(), Cell::default());
		self.0.push(tmp);
	}
	fn left_extend(&mut self) {
		for row in self.0.iter_mut() {
			row.insert(0, Cell::default())
		}
	}
	fn right_extend(&mut self) {
		for row in self.0.iter_mut() {
			row.push(Cell::default())
		}
	}

	fn prefit(&mut self) {
		if self.0.len() <= 0 {
			return;
		}

		let edges = (
			self.0[0].iter().any(|x| x.0),                       // top
			self.0[self.0.len() - 1].iter().any(|x| x.0),        // bottom
			self.0.iter().map(|x| &x[0]).any(|x| x.0),           // left
			self.0.iter().map(|x| &x[x.len() - 1]).any(|x| x.0), // right
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
		if self.0.len() <= 0 {
			return;
		}
		self.prefit();

		let height = self.0.len();
		let width = self.0[0].len();

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
		self.0 = scratch;
	}

	/// Set a cell's value at a given xy coordinate.
	pub fn set_cell (&mut self, x: u64, y: u64, value: Cell) {
		let x = x as usize;
		let y = y as usize;

		if self.0.len() <= 0 {
			self.0 = vec![vec![Cell::default()]]
		}

		while self.0.len() <= y {
			self.bottom_extend();
		}
		while self.0[0].len() <= x {
			self.right_extend();
		}

		self.0[y][x] = value;
	}

	/// Shrinks the table to be as small as possible (without loosing any data).
	pub fn shrink(&mut self) {
		if self.0.len() <= 0 {
			return;
		}

		// top
		let mut count = 0;
		while count < self.0.len() && !self.0[count].iter().any(|x| x.0) {
			count += 1;
		}
		drop(self.0.drain(..count));

		if self.0.len() <= 0 {
			return;
		}

		// bottom
		let mut count = 0;
		while count < self.0.len() && !self.0[self.0.len() - 1 - count].iter().any(|x| x.0) {
			count += 1;
		}
		drop(self.0.drain(self.0.len() - count..));

		if self.0.len() <= 0 {
			return;
		}

		// right
		while !self.0.iter().any(|row| row[0].0) {
			for row in self.0.iter_mut() {
				drop(row.drain(..1));
				if row.len() <= 0 {
					self.0.clear();
					return;
				}
			}
		}

		// left
		while !self.0.iter().any(|row| row[row.len() - 1].0) {
			for row in self.0.iter_mut() {
				row.pop();
			}
		}
	}

	/// Return the height.
	pub fn height(&self) -> u64 {
		return self.0.len() as u64;
	}

	/// Return the width.
	pub fn width(&self) -> u64 {
		if self.0.len() <= 0 {
			return 0;
		}

		return self.0[0].len() as u64;
	}

	/// Create a new `Canvas` with an initial width and height.
	pub fn new(width: u64, height: u64) -> Self {
		let width = width as usize;
		let height = height as usize;

		let mut row = Vec::new();
		row.resize(width, Cell::default());

		let mut out = Vec::new();
		out.resize(height, row);

		return Self(out);
	}

	pub fn get_cells(&self) -> &'_ Vec<Vec<Cell>> {
		&self.0
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
