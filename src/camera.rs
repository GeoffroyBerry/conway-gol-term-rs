pub struct ViewRect {
    pub x: isize,
    pub y: isize,
    pub x_len: isize,
    pub y_len: isize,
    pub x_max: isize,
    pub y_max: isize,
}

const X_INIT_RATIO: f64 = 0.8;
const Y_INIT_RATIO: f64 = 0.8;
impl ViewRect {
    pub fn new(x: isize, y: isize, x_max: isize, y_max: isize) -> Self {
        let x_len = (x_max as f64 * X_INIT_RATIO) as isize;
        let y_len = (y_max as f64 * Y_INIT_RATIO) as isize;
        Self {
            x: x,
            y: y,
            x_len: x_len,
            y_len: y_len,
            x_max: x_max,
            y_max: y_max,
        }
    }

    pub fn zoom_x(&mut self, amount: isize) {
        let future_len = self.x_len - 2*amount;
        if amount > 0 || (amount < 0 && future_len <= self.x_max) {
            self.x += amount;
            self.x_len = future_len;
        }
    }
    pub fn zoom_y(&mut self, amount: isize) {
        let future_len = self.y_len - 2*amount;
        if amount > 0 || (amount < 0 && future_len <= self.y_max) {
            self.y += amount;
            self.y_len = future_len;
        }
    }
    pub fn zoom(&mut self, amount: isize) {
        self.zoom_x(amount);
        self.zoom_y(amount);
    }
    pub fn unzoom(&mut self, amount: isize) {
        self.zoom(-amount);
    }

    pub fn move_left(&mut self, amount: isize) {
        self.x -= amount;
    }
    pub fn move_right(&mut self, amount: isize) {
        self.x += amount;
    }
    pub fn move_up(&mut self, amount: isize) {
        self.y -= amount;
    }
    pub fn move_down(&mut self, amount: isize) {
        self.y += amount;
    }
}
