extern crate rand;
extern crate sdl2;

use rand::Rng;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::collections::HashSet;
use std::time::Duration;

#[derive(Debug)]
struct Bounds {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl Bounds {
    pub fn new() -> Bounds {
        Bounds {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
        }
    }

    pub fn from_dimensions(width: f32, height: f32) -> Bounds {
        Bounds {
            x1: 0.0,
            y1: 0.0,
            x2: width,
            y2: height,
        }
    }

    pub fn update(&mut self, x: f32, y: f32) {
        self.x1 = self.x1.min(x);
        self.y1 = self.y1.min(y);
        self.x2 = self.x2.max(x);
        self.y2 = self.y2.max(y);
    }

    pub fn project(&self, x: f32, y: f32, onto: &Bounds) -> (i32, i32) {
        (
            self.map(x, self.x1, self.x2, onto.x1, onto.x2) as i32,
            self.map(y, self.y1, self.y2, onto.y1, onto.y2) as i32,
        )
    }

    fn map(&self, value: f32, start1: f32, stop1: f32, start2: f32, stop2: f32) -> f32 {
        let size1 = stop1 - start1;
        let size2 = stop2 - start2;
        let position = (start1 - value).abs() / size1;
        size2 * position
    }
}

struct BarnsleyIFS {
    coefficients: Vec<Vec<f32>>,
    probabilities: Vec<f32>,
}

impl BarnsleyIFS {
    fn new(co: Vec<Vec<f32>>, prob: Vec<f32>) -> BarnsleyIFS {
        // [0.01, 0.85, 0.07, 0.07] -> [0.01, 0.86, 0.93, 1.0]
        let mut op = vec![];
        for p in prob {
            let l = *op.last().unwrap_or(&0.0);
            &op.push(p + l);
        }

        BarnsleyIFS {
            coefficients: co,
            probabilities: op,
        }
    }

    fn fern() -> BarnsleyIFS {
        let c = vec![
            vec![0.0, 0.0, 0.0, 0.16, 0.0, 0.0],
            vec![0.85, 0.04, -0.04, 0.85, 0.0, 1.6],
            vec![0.2, -0.26, 0.23, 0.22, 0.0, 1.6],
            vec![-0.15, 0.28, 0.26, 0.24, 0.0, 0.44],
        ];
        let p = vec![0.01, 0.85, 0.07, 0.07];
        BarnsleyIFS::new(c, p)
    }

    fn maple_leaf() -> BarnsleyIFS {
        let c = vec![
            vec![0.14, 0.01, 0.0, 0.51, -0.08, -1.31],
            vec![0.43, 0.52, -0.45, 0.50, 1.49, -0.75],
            vec![0.45, -0.49, 0.47, 0.47, -1.62, -0.74],
            vec![0.49, 0.0, 0.0, 0.51, 0.02, 1.62],
        ];
        let p = vec![0.1, 0.35, 0.35, 0.2];
        BarnsleyIFS::new(c, p)
    }

    fn sierpenski() -> BarnsleyIFS {
        let c = vec![
            vec![0.5, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0],
            vec![0.5, 0.0, 0.0, 0.5, 0.25, 0.5 * 3.0f32.sqrt() / 2.0],
            vec![0.5, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0],
        ];
        let p = vec![0.33, 0.34, 0.33];
        BarnsleyIFS::new(c, p)
    }

    fn next(&self, x: f32, y: f32, rnd: f32) -> (f32, f32) {
        let mut slot = 0;
        for p in &self.probabilities {
            if rnd > *p {
                slot += 1;
            }
        }

        if let Some(t) = self.coefficients.get(slot) {
            let new_x = t[0] * x + t[1] * y + t[4];
            let new_y = t[2] * x + t[3] * y + t[5];
            return (new_x, new_y);
        } else {
            return (0.0, 0.0);
        }
    }
}

pub fn main() {
    let mut rng = rand::thread_rng();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("fracti", 1280, 720)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let bgcolor = Color::RGB(32, 32, 32);
    let mut pxcolor = Color::RGB(0, 96, 0);
    let mut scale = 1.0;

    canvas.set_draw_color(bgcolor);
    canvas.clear();
    canvas.present();

    let ifs = BarnsleyIFS::fern();
    //    let ifs = BarnsleyIFS::maple_leaf();
    //    let ifs = BarnsleyIFS::sierpenski();
    let size = canvas.output_size().unwrap();
    let screen_bounds = Bounds::from_dimensions(size.0 as f32, size.1 as f32);
    let mut x = 0.0;
    let mut y = 0.0;

    // generate image
    let mut computed_points = Vec::<(i32, i32)>::new();
    let mut bounds = Bounds::new();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(bgcolor);
        canvas.clear();
        canvas.set_draw_color(pxcolor);

        if computed_points.len() < 1_000_000 {
            for _ in 1..10_000 {
                let next = ifs.next(x, y, rng.gen());
                x = next.0;
                y = next.1;

                // project onto 1000 times field
                let pos_x = x * 1000.0;
                let pos_y = y * 1000.0;
                computed_points.push((pos_x as i32, pos_y as i32));

                // update bounds
                bounds.update(pos_x, pos_y);
            }
        }

        for p in &computed_points {
            let projected_point = bounds.project(p.0 as f32, p.1 as f32, &screen_bounds);
            let result = canvas.draw_point(Point::new(projected_point.0, projected_point.1));
            match result {
                Ok(_) => (),
                Err(_err) => (),
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => pxcolor.g = (pxcolor.g as i16 - 10).max(0) as u8,
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => pxcolor.g = (pxcolor.g as i16 + 10).min(255) as u8,
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => scale = (scale + 0.1f32).min(5.0),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => scale = (scale - 0.1f32).max(0.0),
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        match canvas.set_scale(scale, scale) {
            Ok(_) => (),
            Err(_err) => (),
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
