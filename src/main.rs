extern crate cairo;
extern crate gdk;
extern crate gio;
extern crate gtk;
extern crate rand;

use std::cell::RefCell;
use std::env::args;
use std::rc::Rc;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::DrawingArea;

use cairo::Context;

type CellMap = Vec<Vec<bool>>;

struct Cells {
    current: CellMap,
    next: CellMap,
    width: u32,
    height: u32,
}

impl Cells {
    fn new(width: u32, height: u32) -> Self {
        Cells {
            current: Self::generate_random(width, height),
            next: vec![vec![false; height as usize]; width as usize],
            width: width,
            height: height,
        }
    }
    fn swap(&mut self) {
        ::std::mem::swap(&mut self.current, &mut self.next);
    }
    fn borrow_cells<'a>(&'a self) -> &'a CellMap {
        &self.current
    }
    fn update(&mut self) {
        self.step();
        self.swap();
    }
    fn step(&mut self) {
        self.next = (0..self.width)
            .map(|x1| {
                (0..self.height)
                    .map(|y1| {
                        let x0 = (x1 + self.width - 1) % self.width;
                        let x2 = (x1 + 1) % self.width;
                        let y0 = (y1 + 1) % self.height;
                        let y2 = (y1 + self.height - 1) % self.height;

                        let n = |x: u32, y: u32| self.current[x as usize][y as usize] as i32;
                        let n_count = n(x0, y0)
                            + n(x1, y0)
                            + n(x2, y0)
                            + n(x0, y1)
                            + n(x2, y1)
                            + n(x0, y2)
                            + n(x1, y2)
                            + n(x2, y2);
                        let is_alive = self.current[x1 as usize][y1 as usize];
                        match n_count {
                            2 if is_alive => true,
                            3 => true,
                            _ => false,
                        }
                    }).collect()
            }).collect();
    }
    fn generate_random(width: u32, height: u32) -> CellMap {
        (0..width)
            .map(|_| (0..height).map(|_| rand::random::<bool>()).collect())
            .collect()
    }
}

fn main() {
    let application = gtk::Application::new("com.github.cairotest", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");

    let cells = Rc::new(RefCell::new(Cells::new(128, 128)));
    application.connect_startup(move |app| {
        start_gol(app, cells.clone());
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}

fn start_gol<'a>(application: &'a gtk::Application, cells: Rc<RefCell<Cells>>) {
    let window = gtk::ApplicationWindow::new(application);
    let drawing_area = Box::new(DrawingArea::new)();
    drawing_area.connect_draw({
        move |_, ctx| {
            let mut cells = cells.borrow_mut();
            cells.update();
            draw_gol(ctx, cells.borrow_cells());
            Inhibit(false)
        }
    });

    drawing_area.add_tick_callback(move |view, _| {
        view.queue_draw();
        Continue(true)
    });

    window.set_default_size(640, 640);

    window.connect_delete_event(move |win, _| {
        win.destroy();
        Inhibit(false)
    });
    window.add(&drawing_area);
    window.show_all();
}

fn draw_gol(ctx: &Context, cells: &CellMap) -> gtk::Inhibit {
    for (x, col) in cells.iter().enumerate() {
        for (y, cell) in col.iter().enumerate() {
            if *cell {
                draw_cell(ctx, x as f64, y as f64, 5_f64);
            }
        }
    }
    Inhibit(false)
}

fn draw_cell(ctx: &Context, x: f64, y: f64, size: f64) {
    ctx.set_source_rgb(0.0, 0.0, 0.0);
    ctx.rectangle(x * size, y * size, size, size);
    ctx.fill();
}
