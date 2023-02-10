mod wasm4;
use std::sync::Mutex;

use lazy_static::lazy_static;
use wasm4::*;

trait Render {
    fn render(&self);
}

trait Move {
    fn shift(&mut self, x: i32, y: i32);
}

enum Orientation {
    Vertical,
    Horizontal,
}

trait Collide {
    fn collides(&self, other: &Self) -> bool;
    fn collision(&self, other: &Self) -> Option<Orientation>;
}

struct Rect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    colors: u16,
}

impl Render for Rect {
    fn render(&self) {
        unsafe { *DRAW_COLORS = self.colors }
        rect(self.x, self.y, self.width, self.height);
    }
}

impl Move for Rect {
    fn shift(&mut self, x: i32, y: i32) {
        self.x += x;
        self.y += y;
    }
}

impl Collide for Rect {
    fn collides(&self, other: &Self) -> bool {
        self.x < other.x + other.width as i32
            && self.x + self.width as i32 > other.x
            && self.y < other.y + other.height as i32
            && self.y + self.height as i32 > other.y
    }
    fn collision(&self, other: &Self) -> Option<Orientation> {
        if !self.collides(other) {
            None
        } else {
            Some(
                if (self.x + self.width as i32 - 2) < other.x
                    || (other.x + other.width as i32 - 2) < self.x
                {
                    Orientation::Vertical
                } else {
                    Orientation::Horizontal
                },
            )
        }
    }
}

struct Ball {
    model: Rect,
    movement: [i32; 2],
}

struct Wall {
    model: Rect,
    orientation: Orientation,
}

struct Brick {
    model: Rect,
}

impl Brick {
    fn new(x: i32, y: i32) -> Self {
        Self {
            model: Rect {
                x: x * (4 + 1) + 2,
                y: y * (4 + 1) + 2,
                width: 8 + 1,
                height: 4,
                colors: 0x32,
            },
        }
    }
    fn new_vert(x: i32, y: i32) -> Self {
        Self {
            model: Rect {
                x: x * (4 + 1) + 2,
                y: y * (4 + 1) + 2,
                width: 4,
                height: 8 + 1,
                colors: 0x32,
            },
        }
    }
    fn explode(&mut self) {
        self.model.x = -32;
        self.model.y = -32;
    }
}

struct Game {
    paddle: Rect,
    ball: Ball,
    walls: [Wall; 4],
    bricks: [Brick; 13],
}

impl Render for Game {
    fn render(&self) {
        for i in self.walls.as_slice() {
            i.model.render();
        }
        for i in self.bricks.as_slice() {
            i.model.render();
        }
        self.paddle.render();
        self.ball.model.render();
    }
}

impl Game {
    fn update(&mut self) {
        for i in self.walls.as_slice() {
            if self.ball.model.collides(&i.model) {
                match i.orientation {
                    Orientation::Vertical => self.ball.movement[0] = -self.ball.movement[0],
                    Orientation::Horizontal => self.ball.movement[1] = -self.ball.movement[1],
                }
            }
        }
        match self.ball.model.collision(&self.paddle) {
            Some(Orientation::Vertical) => self.ball.movement[0] = -self.ball.movement[0],
            Some(Orientation::Horizontal) => self.ball.movement[1] = -self.ball.movement[1],
            None => (),
        }
        for i in self.bricks.as_mut() {
            match self.ball.model.collision(&i.model) {
                Some(orientation) => {
                    i.explode();
                    match orientation {
                        Orientation::Vertical => self.ball.movement[0] = -self.ball.movement[0],
                        Orientation::Horizontal => self.ball.movement[1] = -self.ball.movement[1],
                    }
                },
                None => (),
            }
        }

        self.ball
            .model
            .shift(self.ball.movement[0], self.ball.movement[1]);
    }
}

lazy_static! {
    static ref GAME: Mutex<Game> = Mutex::new(Game {
        paddle: Rect {
            x: 80 - 7,
            y: 80 - 7,
            width: 14,
            height: 14,
            colors: 0x32
        },
        ball: Ball {
            model: Rect {
                x: 80,
                y: 30,
                width: 4,
                height: 4,
                colors: 0x32
            },
            movement: [1, 1],
        },
        walls: [
            Wall {
                model: Rect {
                    x: 0,
                    y: 0,
                    width: 160,
                    height: 1,
                    colors: 0x4,
                },
                orientation: Orientation::Horizontal
            },
            Wall {
                model: Rect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 160,
                    colors: 0x4,
                },
                orientation: Orientation::Vertical
            },
            Wall {
                model: Rect {
                    x: 0,
                    y: 159,
                    width: 160,
                    height: 1,
                    colors: 0x4,
                },
                orientation: Orientation::Horizontal
            },
            Wall {
                model: Rect {
                    x: 159,
                    y: 0,
                    width: 1,
                    height: 160,
                    colors: 0x4,
                },
                orientation: Orientation::Vertical
            },
        ],
        bricks: [
            Brick::new(0, 0),                             Brick::new(2, 0), Brick::new(4, 0), Brick::new_vert(6, 0), Brick::new_vert(7, 0),
            Brick::new(0, 1),                             Brick::new(2, 1), Brick::new(4, 1),
            Brick::new_vert(0, 2), Brick::new_vert(1, 2), Brick::new(2, 2), Brick::new(4, 2), Brick::new(6, 2),
        ],
    });
}

#[no_mangle]
fn update() {
    let gamepad = unsafe { *GAMEPAD1 };
    if gamepad & BUTTON_RIGHT != 0 {
        GAME.lock().unwrap().paddle.shift(1, 0);
    }
    if gamepad & BUTTON_LEFT != 0 {
        GAME.lock().unwrap().paddle.shift(-1, 0);
    }
    if gamepad & BUTTON_DOWN != 0 {
        GAME.lock().unwrap().paddle.shift(0, 1);
    }
    if gamepad & BUTTON_UP != 0 {
        GAME.lock().unwrap().paddle.shift(0, -1);
    }

    GAME.lock().unwrap().update();
    GAME.lock().unwrap().render();
}
