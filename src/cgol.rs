//! # Rules
//! Let `n` be the number of live neighbors.
//! 1. If alive and `n != 2 && n != 3` then die; otherwise remain alive
//! 2. If dead and `n == 3` then become alive



use std::ops::Range;



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
	origin: (isize, isize),
	cells: Vec<Vec<Cell>>,
}

impl Grid {
	fn compute_cell(&self, xy: (usize, usize)) -> Cell {
		let height = self.cells.len();
		let width = self.cells[0].len();

		let offsets: [(isize, isize); 8] = [
			(-1, -1), ( 0, -1), ( 1, -1),
			(-1,  0),           ( 1,  0),
			(-1,  1), ( 0,  1), ( 1,  1),
		];

		let cell = &self.cells[xy.1][xy.0];

		let mut n = 0;
		for (i, j) in offsets {
			// don't count self
			if i == 0 && j == 0{
				continue;
			}

			// ignore outside of canvas
			if (xy.0 == 0 && i < 0) || (xy.0 + 1 >= width && i > 0) {
				continue;
			}
			if (xy.1 == 0 && j < 0) || (xy.1 + 1 >= height && j > 0) {
				continue;
			}

			let other = &self.cells[xy.1.checked_add_signed(j).unwrap()][xy.0.checked_add_signed(i).unwrap() as usize];
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
				scratch[y][x] = self.compute_cell((x, y));
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


	/// Return the height.
	pub fn height(&self) -> usize {
		return self.cells.len();
	}

	/// Return the width.
	pub fn width(&self) -> usize {
		if self.cells.len() <= 0 {
			return 0;
		}

		return self.cells[0].len();
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
	pub fn set_bounds(&mut self, neg_x: usize, neg_y: usize, pos_x: usize, pos_y: usize) {
		while self.origin.0 < neg_x as isize {
			self.left_extend();
		}

		while self.origin.1 < neg_y as isize {
			self.top_extend();
		}

		while self.width().checked_sub_signed(self.origin.0).unwrap() < pos_x {
			self.right_extend();
		}

		while self.height().checked_sub_signed(self.origin.1).unwrap() < pos_y {
			self.bottom_extend();
		}
	}

	/// Shrinks the table to be as small as possible (without loosing any data).
	pub fn shrink(&mut self) {
		if self.cells.len() <= 0 {
			return;
		}

		let (x, y) = self.get_bounding_box();
		dbg!(&x, &y);
		dbg!(x.is_empty(), y.is_empty());
		if x.is_empty() || y.is_empty() {
			self.cells.clear();
			self.origin = (0, 0);
			return;
		}

		if !y.is_empty() {
			self.origin.1 -= y.start as isize;

			drop(self.cells.drain(y.end..));
			drop(self.cells.drain(..y.start));
		}
		if !x.is_empty() {
			self.origin.0 -= x.start as isize;

			for row in self.cells.iter_mut() {
				drop(row.drain(x.end..));
				drop(row.drain(..x.start));
			}
		}
	}

	/// Get the bounding box of alive cells. Returns `(x, y)`.
	pub fn get_bounding_box(&self) -> (Range<usize>, Range<usize>) {
		if self.width() <= 0 || self.height() <= 0 {
			return Default::default();
		}

		dbg!(&self);

		let y_start = self.cells.iter().take_while(|row| !row.iter().any(|c| c.is_alive())).count();
		let y_end = self.height() as usize - self.cells.iter().rev().take_while(|row| !row.iter().any(|c| c.is_alive())).count();
		let y = y_start..y_end;

		let width = self.width() as usize;
		let x_start = (0..width).into_iter()
			.take_while(|i| self.cells.iter().all(|row| !row[            *i].is_alive())).count();
		let x_end = width - (0..width).into_iter()
			.take_while(|i| self.cells.iter().all(|row| !row[width - 1 - *i].is_alive())).count();
		let x = x_start..x_end;

		return (x, y);
	}

	/// Get the origin point of the grid.
	pub fn get_origin(&self) -> (isize, isize) {
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
