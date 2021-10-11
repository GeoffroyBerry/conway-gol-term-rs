mod menu;
mod camera;

use menu::{game_selection, hud};
use camera::ViewRect;

use std::fs;
use std::thread;
use std::io::{self, stdout, Stdout};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::style::{self, Stylize};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{cursor, execute, queue};

#[allow(unused_must_use)]
fn main() {
    let mut speed: f32 = 4.0; // generations per seconds

    let mut count = 0;
    let mut quit = false;

    //start using crossterm
    let mut stdout = stdout();
    enable_raw_mode().unwrap();

    // first the user select the file
    match game_selection(&mut stdout) {
        Ok(mut game) => {
            let mut last_time = std::time::SystemTime::now();
            queue!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0));
            hud(&mut stdout, count, speed);
            let size = terminal::size().unwrap();
            let mut camera = ViewRect::new(
                0,
                0,
                size.0 as isize - 2,
                size.1 as isize - 4,
            );
            let default_move = 2;
            game.show_in_camera(&mut stdout, &camera);
            while game.next() {
                count += 1;

                //making sure the generation rate is constant (if the speed is too high it waits for the
                //code to finish executing
                let elapsed = last_time.elapsed().unwrap().as_millis();
                speed = (speed * 100.0).round() / 100.0;
                let wait_time = 1000.0 / speed;
                let real_wait_time = wait_time - elapsed as f32;

                thread::sleep(std::time::Duration::from_millis(real_wait_time as u64));
                last_time = std::time::SystemTime::now();

                //using result from keys pressed
                while event::poll(std::time::Duration::from_millis(1)).unwrap() {
                    match event::read().unwrap() {
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('x'),
                            modifiers: KeyModifiers::NONE,
                            //clearing the screen and printing our message
                        }) => {
                            if speed >= 1.0 {
                                speed += 1.0;
                            } else {
                                speed *= 2.0;
                            }
                        }
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers: KeyModifiers::NONE,
                        }) => {
                            if speed >= 2.0 {
                                speed -= 1.0;
                            } else if speed >= 1.0 {
                                speed *= 0.5;
                            }
                        }
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('q'),
                            modifiers: KeyModifiers::NONE,
                        }) => quit = true,
                        Event::Key(KeyEvent {
                            code: KeyCode::Up,
                            modifiers: KeyModifiers::NONE,
                        }) => camera.move_up(default_move),
                        Event::Key(KeyEvent {
                            code: KeyCode::Down,
                            modifiers: KeyModifiers::NONE,
                        }) => camera.move_down(default_move),
                        Event::Key(KeyEvent {
                            code: KeyCode::Left,
                            modifiers: KeyModifiers::NONE,
                        }) => camera.move_left(default_move),
                        Event::Key(KeyEvent {
                            code: KeyCode::Right,
                            modifiers: KeyModifiers::NONE,
                        }) => camera.move_right(default_move),
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('u'),
                            modifiers: KeyModifiers::NONE,
                        }) => camera.unzoom(1),
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('z'),
                            modifiers: KeyModifiers::NONE,
                        }) => camera.zoom(1),
                        Event::Resize(x, y) => {
                            camera.x_max = x as isize - 2;
                            camera.y_max = y as isize - 4;
                        },
                        _ => (),
                        }
                    }

                if quit {
                    break;
                }

                //Display the new generation
                queue!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0));
                hud(&mut stdout, count, speed);
                game.show_in_camera(&mut stdout, &camera);
            }

            execute!(
                stdout,
                cursor::MoveToNextLine(1),
                Print(format!("died at generation {}", &count)),
                cursor::MoveToNextLine(1)
            );
        }
        Err(e) => {
            disable_raw_mode().unwrap();
            execute!(stdout, cursor::MoveToNextLine(1));
            eprintln!("{:?}", e);
            execute!(stdout, cursor::MoveToNextLine(1));
        }
    }

    //end using crossterm
    disable_raw_mode().unwrap();
}

#[derive(Clone)]
struct CellRow {
    row: Vec<Cell>,
    y: isize,
}
impl CellRow {
    //base_x is the x of the first cell (based on the one of others)
    fn new(y: isize, base_x: isize) -> CellRow {
        CellRow {
            row: vec![Cell::new(base_x, false)],
            y: y,
        }
    }

    fn append_cell(&mut self, alive: bool) {
        let x = self.row.last().unwrap().x + 1;
        self.row.push(Cell::new(x, alive));
    }

    fn prepend_cell(&mut self, alive: bool) {
        let x = self.row.first().unwrap().x - 1;
        self.row.insert(0, Cell::new(x, alive));
    }

    fn get_cell(&mut self, x: isize) -> Option<&mut Cell> {
        let base_x = self.row.first().unwrap().x;
        let dif = x - base_x;
        self.row.get_mut(dif as usize)
    }
}

#[derive(Clone)]
struct UniqueCoordinates {
    coords: Vec<(isize, isize)>,
}
impl UniqueCoordinates {
    fn push(&mut self, coords: (isize, isize)) {
        let can_push = !self
            .coords
            .iter_mut()
            .any(|c| c.0 == coords.0 && c.1 == coords.1);
        if can_push {
            self.coords.push(coords);
        }
    }
    fn remove(&mut self, coords: (isize, isize)) {
        let index = self
            .coords
            .iter()
            .position(|c| c.0 == coords.0 && c.1 == coords.1);
        if let Some(i) = index {
            self.coords.remove(i);
        }
    }
}

#[derive(Clone)]
struct GameGrid {
    grid: Vec<CellRow>,
    alive_cells: UniqueCoordinates,
}
impl GameGrid {
    fn new() -> Self {
        GameGrid {
            grid: vec![CellRow::new(0, 0)],
            alive_cells: UniqueCoordinates { coords: vec![] },
        }
    }
    fn get_neighbours_coords(&mut self, x: isize, y: isize) -> Vec<(isize, isize)> {
        let mut r: Vec<(isize, isize)> = vec![];

        for j in -1..=1 {
            for i in -1..=1 {
                if i != 0 || j != 0 {
                    if let Some(_) = self.get_cell(x + i, y + j) {
                        r.push((x + i, y + j));
                    }
                }
            }
        }

        r
    }
    fn count_neighbours(&mut self, x: isize, y: isize) -> u8 {
        let mut count: u8 = 0;

        for j in -1..=1 {
            for i in -1..=1 {
                if i != 0 || j != 0 {
                    if let Some(c) = self.get_cell(x + i, y + j) {
                        if c.is_alive {
                            count += 1;
                        }
                    }
                }
            }
        }

        count
    }

    //whenever there is an alive cell on an edge expend the edge by 1 (row or cell for each row)
    //+when edge row or column is dead and so is its neighbour remove it
    //To call right after next (or at the end of it)
    //TODO try to do similar function that updates only part of edges?
    fn update_edges(&mut self) {
        if self.is_alive_left() {
            for r in self.grid.iter_mut() {
                r.prepend_cell(false);
            }
        } else if !self.is_alive_column(1) {
            self.remove_first_column();
        }
        if self.is_alive_right() {
            for r in self.grid.iter_mut() {
                r.append_cell(false);
            }
        } else if !self.is_alive_column(self.grid[0].row.len() - 2) {
            self.remove_last_column();
        }
        if self.is_alive_bottom() {
            self.append_row_and_fill();
        } else if !self.is_alive_row(self.grid.len() - 2) {
            self.remove_last_row();
        }
        if self.is_alive_top() {
            self.prepend_row_and_fill();
        } else if !self.is_alive_row(1) {
            self.remove_first_row();
        }
    }

    fn get_cell(&mut self, x: isize, y: isize) -> Option<&mut Cell> {
        self.get_row(y)?.get_cell(x)
    }

    //input is the real index the artificial one
    fn is_alive_row(&self, index: usize) -> bool {
        self.grid.get(index).unwrap().row.iter().any(|c| c.is_alive)
    }
    fn is_alive_bottom(&self) -> bool {
        self.grid.last().unwrap().row.iter().any(|c| c.is_alive)
    }
    fn is_alive_top(&self) -> bool {
        self.grid.first().unwrap().row.iter().any(|c| c.is_alive)
    }
    fn is_alive_column(&self, index: usize) -> bool {
        self.grid.iter().any(|r| r.row.get(index).unwrap().is_alive)
    }
    fn is_alive_left(&self) -> bool {
        self.grid.iter().any(|r| r.row.first().unwrap().is_alive)
    }
    fn is_alive_right(&self) -> bool {
        self.grid.iter().any(|r| r.row.last().unwrap().is_alive)
    }

    //add new row to the end
    fn append_row(&mut self) {
        let previous_row = self.grid.last().unwrap();
        let y = previous_row.y + 1;
        let base_x = previous_row.row[0].x;
        let cr = CellRow::new(y, base_x);
        self.grid.push(cr);
    }
    fn append_row_and_fill(&mut self) {
        let previous_row = self.grid.last().unwrap();
        let y = previous_row.y + 1;
        let base_x = previous_row.row[0].x;
        let len_x = previous_row.row.len();
        let mut cr = CellRow::new(y, base_x);
        while cr.row.len() < len_x {
            cr.append_cell(false);
        }
        self.grid.push(cr);
    }

    //add new row to begining
    #[allow(dead_code)]
    fn prepend_row(&mut self) {
        let previous_row = self.grid.first().unwrap();
        let y = previous_row.y - 1;
        let base_x = previous_row.row[0].x;
        let cr = CellRow::new(y, base_x);
        self.grid.insert(0, cr);
    }
    fn prepend_row_and_fill(&mut self) {
        let previous_row = self.grid.first().unwrap();
        let y = previous_row.y - 1;
        let base_x = previous_row.row[0].x;
        let len_x = previous_row.row.len();
        let mut cr = CellRow::new(y, base_x);
        while cr.row.len() < len_x {
            cr.append_cell(false);
        }
        self.grid.insert(0, cr);
    }

    fn remove_first_row(&mut self) {
        self.grid.remove(0);
    }
    fn remove_last_row(&mut self) {
        self.grid.pop();
    }
    fn remove_first_column(&mut self) {
        for r in self.grid.iter_mut() {
            r.row.remove(0);
        }
    }
    fn remove_last_column(&mut self) {
        for r in self.grid.iter_mut() {
            r.row.pop();
        }
    }

    fn get_row(&mut self, y: isize) -> Option<&mut CellRow> {
        let base_y = self.grid.first().unwrap().y;
        let dif = y - base_y;
        self.grid.get_mut(dif as usize)
    }

    //use to make it so all rows have the same length by adding at the end
    //only use during initialisation phase?
    fn fix_grid_size(&mut self) {
        let max = self
            .grid
            .iter()
            .max_by(|x, y| x.row.len().cmp(&y.row.len()))
            .unwrap()
            .row
            .len();

        for i in 0..self.grid.len() {
            while self.grid[i].row.len() < max {
                self.grid[i].append_cell(false);
            }
        }
    }

    fn init_alive_cells(&mut self) {
        for r in self.grid.iter_mut() {
            for c in r.row.iter_mut() {
                if c.is_alive {
                    self.alive_cells.push((c.x, r.y));
                }
            }
        }
    }

    fn next(&mut self) -> bool {
        let mut clo = self.clone();

        let ac = self.alive_cells.clone();
        let mut change_cells = UniqueCoordinates { coords: vec![] };

        //get cells coords to change
        for coords in ac.coords.iter() {
            change_cells.push((coords.0, coords.1));
            for n in self.get_neighbours_coords(coords.0, coords.1) {
                change_cells.push((n.0, n.1));
            }
        }

        //update the cells
        for co in change_cells.coords.iter() {
            let c = self.get_cell(co.0, co.1).unwrap();
            c.update(clo.count_neighbours(co.0, co.1));
        }

        let mut changed = false;
        //go to next cells
        for co in change_cells.coords.iter() {
            let c = self.get_cell(co.0, co.1).unwrap();
            let was_alive = c.is_alive;
            if c.next() {
                changed = true;
                if was_alive {
                    self.alive_cells.remove(*co);
                } else {
                    self.alive_cells.push(*co);
                }
            }
        }

        // makes sure next generation will have enough space
        self.update_edges();

        changed
    }

    fn add_text(&mut self, text: &str, row_min: usize, col_min: usize) {
        //TODO add way to have a minimum of columns
        let n_row = text.trim().chars().filter(|x| *x == '\n').count();
        let mut row_diff = 0;
        if row_min > n_row {
            row_diff = row_min - n_row;
        }
        let mut line_start = 0;
        if row_diff > 1 {
            line_start = (row_diff - (row_diff % 2)) / 2;
        }

        while self.grid.len() < row_min {
            self.append_row();
        }

        let mut line_count = line_start;
        for c in text.trim().chars() {
            if c == '\n' {
                line_count += 1;
                if line_count >= self.grid.len() {
                    self.append_row();
                }
            } else {
                //add cell which is alive if char in str is 'a'
                self.get_row(line_count as isize)
                    .unwrap()
                    .append_cell(c == 'a');
            }
        }
    }
}

pub struct GameOfLife {
    game_grid: GameGrid,
}
impl GameOfLife {
    fn init(path: &str) -> Result<GameOfLife, io::Error> {
        let contents = fs::read_to_string(path)?;
        let mut g = GameGrid::new();

        g.add_text(&contents, 0, 0);
        g.fix_grid_size();
        g.init_alive_cells();

        Ok(GameOfLife { game_grid: g })
    }

    fn from_word(s: &str) -> Result<GameOfLife, io::Error> {
        //Read the word
        let base_dir = "./letters/".to_string();
        let mut g = GameGrid::new();

        //let spaced_s = "  ".to_string() + s + "  ";
        for i in 0..s.len() {
            //Get File corresponding to letter
            let mut c = &s[i..i + 1];
            if c == " " {
                c = "empty";
            }
            let file: String = "".to_string() + &base_dir + c + ".gol";
            let letter = std::fs::read_to_string(file)?;

            g.add_text(&letter, 16, 0);
        }
        g.fix_grid_size();
        g.init_alive_cells();

        Ok(GameOfLife { game_grid: g })
    }

    fn next(&mut self) -> bool {
        self.game_grid.next()
    }

    #[allow(unused_must_use)]
    fn show_in_camera(&mut self, so: &mut Stdout, camera: &ViewRect) {
        //, rect: ViewRect) {
        let border_style = style::PrintStyledContent("█".dark_green());
        let alive_style = style::PrintStyledContent("█".dark_cyan());
        //TOP BORDER
        queue!(so, cursor::MoveToNextLine(1));
        for _ in 0..camera.x_len + 2 {
            queue!(so, &border_style);
        }
        //MIDDLE
        for y in camera.y..camera.y + camera.y_len {
            //LEFT BORDER
            queue!(so, cursor::MoveToNextLine(1), &border_style);
            //CONTENT
            for x in camera.x..camera.x + camera.x_len {
                let cell_option = self.game_grid.get_cell(x, y);
                match cell_option {
                    Some(c) => {
                        match c.is_alive {
                            true => queue!(so, &alive_style),
                            false => queue!(so, style::PrintStyledContent("+".magenta())), //cursor::MoveRight(1)),
                        };
                    }
                    None => {
                        queue!(so, style::PrintStyledContent("-".dark_red()));
                    }
                };
            }
            //RIGHT BORDER
            queue!(so, &border_style);
        }
        //BOTTOM BORDER
        queue!(so, cursor::MoveToNextLine(1));
        for _ in 0..camera.x_len + 2 {
            queue!(so, &border_style);
        }

        execute!(so);
    }
}


#[derive(Debug, Clone)]
struct Cell {
    is_alive: bool,
    neighbours: u8,
    x: isize,
    //y: isize
}
impl Cell {
    fn new(x: isize, is_alive: bool) -> Cell {
        Cell {
            is_alive: is_alive,
            neighbours: 0,
            x: x,
            //y: y,
        }
    }

    fn update(&mut self, neighbours: u8) {
        self.neighbours = neighbours;
    }

    fn next(&mut self) -> bool {
        if self.is_alive {
            if self.neighbours <= 1 || self.neighbours >= 4 {
                self.is_alive = false;
                return true;
            }
        } else {
            if self.neighbours == 3 {
                self.is_alive = true;
                return true;
            }
        }
        false
    }
}
