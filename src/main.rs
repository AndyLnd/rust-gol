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

struct Cells {
    current: Vec<bool>,
    next: Vec<bool>,
    width: u32,
    height: u32,
    size: u32,
}

impl Cells {
    fn new(width: u32, height: u32) -> Self {
        let size = width * height;
        Cells {
            current: Self::generate_random(size),
            next: vec![false; size as usize],
            width: width,
            height: height,
            size: size,
        }
    }
    fn swap(&mut self) {
        ::std::mem::swap(&mut self.current, &mut self.next);
    }
    fn borrow_cells<'a>(&'a self) -> &'a Vec<bool> {
        &self.current
    }
    fn update(&mut self) {
        self.step();
        self.swap();
    }
    fn randomize(&mut self) {
        self.next = (0..self.size)
            .map(|_| rand::random::<bool>())
            .collect::<Vec<bool>>();
    }
    fn step(&mut self) {
        self.next = (0..self.size)
            .map(|index| {
                let n = |x: u32, y: u32| self.current[(x + y * self.width) as usize] as i32;
                let x0 = (index + self.width - 1) % self.width;
                let x1 = index % self.width;
                let x2 = (index + 1) % self.width;
                let y0 = ((index + self.size - self.width) / self.width) % self.height;
                let y1 = index / self.width;
                let y2 = ((index + self.width) / self.width) % self.height;
                let neighbor_count = n(x0, y0)
                    + n(x1, y0)
                    + n(x2, y0)
                    + n(x0, y1)
                    + n(x2, y1)
                    + n(x0, y2)
                    + n(x1, y2)
                    + n(x2, y2);
                let is_alive = self.current[(x1 + y1 * self.width) as usize];
                match neighbor_count {
                    n if n < 2 => false,
                    2 if is_alive => true,
                    3 => true,
                    _ => false,
                }
            }).collect();
    }
    fn generate_random(size: u32) -> Vec<bool> {
        (0..size)
            .map(|_| rand::random::<bool>())
            .collect::<Vec<bool>>()
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

fn draw_gol(ctx: &Context, cells: &[bool]) -> gtk::Inhibit {
    for (i, item) in cells.iter().enumerate() {
        if *item {
            draw_cell(ctx, i as i32 % 128, i as i32 / 128, 5);
        }
    }
    Inhibit(false)
}

fn draw_cell(ctx: &Context, x: i32, y: i32, size: i32) {
    let f_size: f64 = size.into();
    let f_x: f64 = x.into();
    let f_y: f64 = y.into();
    ctx.set_source_rgb(0.0, 0.0, 0.0);
    ctx.rectangle(f_x * f_size, f_y * f_size, f_size, f_size);
    ctx.fill();
}
