#![allow(
    unused_variables,
    non_snake_case,
    non_upper_case_globals,
    unused_imports,
    unused_parens,
    non_camel_case_types,
    unused,
    dead_code
)]
//extern crate sdl3;

use sdl3::Sdl;
use sdl3::VideoSubsystem;
use sdl3::event::Event;
use sdl3::gpu::Device;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::rect::Point;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::render::FPoint;
use sdl3::render::FRect;
use sdl3::sys::gpu::SDL_CreateGPUDevice;
use sdl3::sys::keycode::SDLK_SPACE;
use sdl3::sys::video::SDL_WINDOW_RESIZABLE;
use sdl3::video::Window;
use std::ops::Deref;
use std::ops::Index;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug)]
struct Pixels {
    color: Color,
    points: Vec<Point>,
}

// maybe just rename this to square or something idk
struct IntRect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

struct Whiteboard {
    canvasBounds: IntRect,
    //canvas: Canvas<Window>,
    pixels: Vec<Pixels>,
    bgcolor: Color,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum ToolMode {
    none,
    pencil,
    eraser,
    movingCanvas,
}

trait meow {
    fn isInside(&self, square: &IntRect) -> bool;
}
impl meow for Point {
    fn isInside(&self, square: &IntRect) -> bool {
        return self.x >= square.x1
            && self.y >= square.y1
            && self.x < square.x2
            && self.y < square.y2;
    }
}
impl IntRect {
    fn shift(&mut self, x: i32, y: i32) {
        self.x1 += x;
        self.x2 += x;
        self.y1 += y;
        self.y2 += y;
    }
    fn width(&self) -> u32 {
        return (self.x2 - self.x1) as u32;
    }
    fn height(&self) -> u32 {
        return (self.y2 - self.y1) as u32;
    }
    fn toFRect(&self) -> FRect {
        return FRect::new(
            self.x1 as f32,
            self.y1 as f32,
            self.width() as f32,
            self.height() as f32,
        );
    }
    fn toRect(&self) -> Rect {
        return Rect::new(self.x1, self.y1, self.width(), self.height());
    }
}

// appends all the points between point1 and point2 to list
fn makeLine(list: &mut Vec<Point>, point1: Point, point2: Point) {
    let mut x1 = point1.x;
    let mut y1 = point1.y;
    let x2 = point2.x;
    let y2 = point2.y;
    let sx = if (x1 < x2) { 1 } else { -1 };
    let sy = if (y1 < y2) { 1 } else { -1 };

    // case for vertical line
    if x1 == x2 {
        loop {
            list.push(Point::new(x1, y1));
            if y1 == y2 {
                break;
            }
            y1 += sy;
        }
        return;
    }
    // case for horizontal line
    if y1 == y2 {
        loop {
            list.push(Point::new(x1, y1));
            if x1 == x2 {
                break;
            }
            x1 += sx;
        }
        return;
    }

    let dx = (x2 - x1).abs();
    let dy = -((y2 - y1).abs());
    let mut error = dy + dx;

    loop {
        list.push(Point::new(x1, y1));
        let e2 = error;
        if e2 >= dy {
            if x1 == x2 {
                break;
            };
            error += dy;
            x1 += sx;
        }
        if e2 <= dx {
            if y1 == y2 {
                break;
            }
            error += dx;
            y1 += sy;
        }
    }
}

fn mouseMovement(
    whiteboard: &mut Whiteboard,
    point1: Point,
    point2: Point,
    toolMode: &ToolMode,
    color: Color,
) {
    match toolMode {
        ToolMode::none => {}
        ToolMode::movingCanvas => {
            let dx = point1.x - point2.x;
            let dy = point1.y - point2.y;
            whiteboard.canvasBounds.shift(dx, dy);
        }
        ToolMode::pencil => {
            let mut pointList: Vec<Point> = Vec::new();
            makeLine(&mut pointList, point1, point2);
            for point in pointList {
                if (!point.isInside(&whiteboard.canvasBounds)) {
                    continue;
                }
                let mut pointDrawn = false;
                // i should check for duplicates here
                for pixels in &mut whiteboard.pixels {
                    if (pixels.color == color) {
                        pixels.points.push(Point::new(
                            point.x - whiteboard.canvasBounds.x1,
                            point.y - whiteboard.canvasBounds.y1,
                        ));
                        pointDrawn = true;
                        break;
                    }
                }
                if (!pointDrawn) {
                    let pixels = Pixels {
                        points: vec![Point::new(
                            point.x - whiteboard.canvasBounds.x1,
                            point.y - whiteboard.canvasBounds.y1,
                        )],
                        color,
                    };
                    whiteboard.pixels.push(pixels);
                }
                //whiteboard.canvas.set_draw_color(color);
                //let result = whiteboard.canvas.draw_point(point);
            }
        }
        ToolMode::eraser => {
            // i really hope theres a more pretty way to do this :sob:
            let mut pointList: Vec<Point> = Vec::new();
            makeLine(&mut pointList, point1, point2);
            for eraserPoint in pointList {
                if (!eraserPoint.isInside(&whiteboard.canvasBounds)) {
                    continue;
                };
                //let mut pixelErased = false;
                'inner: for pixels in &mut whiteboard.pixels {
                    for i in 0..pixels.points.len() {
                        if (pixels.points.get(i).is_some()) {
                            let point = pixels.points.get(i).unwrap();
                            if (eraserPoint == *point) {
                                // swap remove is faster and i dont need order
                                pixels.points.swap_remove(i);
                                //whiteboard.canvas.set_draw_color(color);
                                //let result = whiteboard.canvas.draw_point(eraserPoint);
                                break 'inner;
                            }
                        }
                    }
                }
            }
        }
    }
}

// todo
// undo feature
// different brush sizes
// change brush colour
// add inserting text
fn main() {
    let sdl_context: Sdl = sdl3::init().unwrap();
    let video_subsystem: VideoSubsystem = sdl_context.video().unwrap();

    const width: u32 = 900;
    const height: u32 = 500;
    let window = video_subsystem
        .window("whiteboard", width, height)
        .set_window_flags(SDL_WINDOW_RESIZABLE as u32) // casting? still works?
        .position_centered()
        .build()
        .unwrap();
    //let canvas = window.into_canvas();

    // needs shader format, such as SPIRV, MSL, etc
    let device = Device::new(sdl3::gpu::ShaderFormat::SPIRV, true).unwrap();
    let result = device.with_window(&window);

    let canvasBounds: IntRect = IntRect {
        x1: 0,
        y1: 0,
        x2: width as i32,
        y2: height as i32,
    };
    let mut whiteboard = Whiteboard {
        canvasBounds,
        //canvas,
        pixels: Vec::with_capacity(10_000),
        bgcolor: Color::RGB(40, 40, 40),
    };
    let mut toolMode = ToolMode::pencil;
    let mut oldTool = ToolMode::none;

    //whiteboard.canvas.set_draw_color(Color::RGB(0, 0, 0));
    //whiteboard.canvas.clear();
    //whiteboard.canvas.present();

    let mut mouseHeldDown = false;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let /*mut*/ currentColor = Color::RGB(255, 255, 255);
    let mut point1: Point;
    let mut point2: Option<Point> = None;
    'running: loop {
        let /*mut*/ needsDraw = false;
        let mut needsClear = false;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::MouseButtonDown { .. } => {
                    mouseHeldDown = true;
                }
                Event::MouseButtonUp { .. } => {
                    mouseHeldDown = false;
                    point2 = None;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => {
                    toolMode = ToolMode::eraser;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::B),
                    ..
                } => {
                    toolMode = ToolMode::pencil;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    if (toolMode != ToolMode::movingCanvas) {
                        oldTool = toolMode;
                    }
                    toolMode = ToolMode::movingCanvas;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    toolMode = oldTool;
                }
                Event::MouseMotion { x, y, .. } => {
                    point1 = Point::new(x as i32, y as i32);
                    if (mouseHeldDown && point2.is_some()) {
                        if (toolMode == ToolMode::movingCanvas) {
                            needsClear = true;
                        }
                        mouseMovement(
                            &mut whiteboard,
                            point1,
                            point2.unwrap(),
                            &toolMode,
                            currentColor,
                        );
                    }
                    point2 = Some(point1);
                }
                _ => {}
            }
        }
        if (needsClear) {
            //whiteboard.canvas.set_draw_color(whiteboard.bgcolor);
            //whiteboard.canvas.clear();
            //whiteboard.canvas.set_draw_color(Color::RGB(0, 0, 0));
            //let result = whiteboard
            //    .canvas
            //    .fill_rect(whiteboard.canvasBounds.toFRect());
        }
        if (needsDraw) {
            //let oldViewport = whiteboard.canvas.viewport();
            //whiteboard
            //    .canvas
            //    .set_viewport(whiteboard.canvasBounds.toRect());
            //    //for pixels in &PixelsVec {
            //    //    canvas.set_draw_color(pixels.color);
            //    //    //let result2 = canvas.draw_points(pixels.points.as_slice());
            //    //}
            //whiteboard.canvas.set_viewport(oldViewport);
        }
        // call at the end of every loop
        //whiteboard.canvas.present();
        sleep(Duration::new(0, 500_000_000u32 / 60));
    }
}
