#![allow(unused_variables, non_snake_case, non_upper_case_globals, unused_imports, unused_parens)]
//extern crate sdl3;

use sdl3::pixels::Color;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::Sdl;
use sdl3::VideoSubsystem;
use sdl3::rect::Point;
use sdl3::rect::Rect;
use sdl3::render::FPoint;
use sdl3::render::FRect;
use sdl3::sys::keycode::SDLK_SPACE;
use sdl3::sys::video::SDL_WINDOW_RESIZABLE;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug)]
struct Pixels {
    color: Color,
    points: Vec<FPoint>,
}

struct CanvasSize {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl CanvasSize {
    fn shift(&mut self, x: i32, y: i32){
        self.x1 += x;
        self.x2 += x;
        self.y1 += y;
        self.y2 += y;
    }
    fn isInside(&self, point: Point) -> bool {
        return point.x >= self.x1 && point.y >= self.y1 && 
            point.x < self.x2 && point.y < self.y2
    }
    fn width(&self) -> i32 {
        return self.x2 - self.x1;
    }
    fn height(&self) -> i32 {
        return self.y2 - self.y1;
    }
}

// appends all the points between point1 and point2 to list
fn makeLine(list: &mut Vec<Point>, point1: Point, point2: Point){
    let mut x1 = point1.x;
    let mut y1 = point1.y;
    let x2 = point2.x;
    let y2 = point2.y;
    let sx = if (x1 < x2) {1} else {-1};
    let sy = if (y1 < y2) {1} else {-1};

    // case for vertical line
    if x1 == x2 {
        loop {
            list.push(Point::new(x1, y1));
            if y1 == y2 {break;}
            y1 += sy;
        }
        return;
    }
    // case for horizontal line
    if y1 == y2 {
        loop {
            list.push(Point::new(x1, y1));
            if x1 == x2 {break;}
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
            if x1 == x2 {break};
            error += dy;
            x1 += sx;
        }
        if e2 <= dx {
            if y1 == y2 {break}
            error += dx;
            y1 += sy;
        }
    }
}


// todo
// undo feature
// different brush sizes
// change brush colour
// add inserting text

// i need to optimise this
// it uses more cpu the more things are drawn on
fn main() {
    let sdl_context: Sdl = sdl3::init().unwrap();
    let video_subsystem:VideoSubsystem = sdl_context.video().unwrap();

    const width: u32 = 1800;
    const height: u32 = 1000;
    const bgcolor: Color = Color::RGB(40, 40, 40);
    let window = video_subsystem.window("whiteboard", width, height)
        .set_window_flags(SDL_WINDOW_RESIZABLE as u32) // casting? still works?
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();
    let mut canvasBounds: CanvasSize = CanvasSize { x1: (0), y1: (0), x2: (width as i32), y2: (height as i32) };

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut mouseHeldDown = false;
    let mut isMovingCanvas = false;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let /*mut*/ currentColor = Color::RGB(255, 255, 255);
    // maybe i could optimise on space by not storing the colour of every single pixel
    // i could do like struct thing{ color: Color, Vec<Pixels,}
    // Vec<thing>
    // that way if the user does a lot of drawing in a small amoutn of colors then it will store
    // much less data
    // however, if you use a lot of different colors, then a lot of different structs will be stored
    // maybe i could switch between different storage modes based on the sizeof a vs sizeof b
    // idk if using two different data structures and switching between them is a good idea
    // has one struct Pixels per colour in the canvas
    let mut PixelsVec: Vec<Pixels> = Vec::new();
    let mut point1: Option<Point>;
    let mut point2: Option<Point> = None;
    'running: loop {
        let mut needsDraw = false;
        let mut needsClear = false;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'running;
                },
                Event::MouseButtonDown {..} => {
                    mouseHeldDown = true;
                },
                Event::MouseButtonUp {..} => {
                    mouseHeldDown = false;
                    point2 = None;
                },
                Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
                    isMovingCanvas = true;
                },
                Event::KeyUp { keycode: Some(Keycode::Space), ..} => {
                    isMovingCanvas = false;
                },
                Event::KeyDown {keycode: Some(Keycode::P), ..} => {
                    println!("length of pixels is {}", PixelsVec.len());
                    println!("contents of pixels is {:?}", PixelsVec);
                }
                Event::MouseMotion {x, y, ..} => 'mouse: {
                    // this might be a little too much indentation lol
                    if (!mouseHeldDown){break 'mouse;}
                    point1 = Some(Point::new(x as i32, y as i32));
                    if (isMovingCanvas) {
                        needsDraw = true;
                        needsClear = true;
                        if (point2.is_some()){
                            let dx = point1.unwrap().x - point2.unwrap().x;
                            let dy = point1.unwrap().y - point2.unwrap().y;
                            canvasBounds.shift(dx, dy);
                        }
                        point2 = point1;
                        break 'mouse;
                    }
                    point1 = Some(Point::new(x as i32, y as i32));
                    if (point1.is_some() && point2.is_some()) {
                        let mut pointList: Vec<Point> = Vec::new();
                        makeLine(&mut pointList, point1.unwrap(), point2.unwrap());
                        for point in pointList {
                            if (canvasBounds.isInside(point)) {
                                let mut pushedPoint = point.clone();
                                pushedPoint.x -= canvasBounds.x1;
                                pushedPoint.y -= canvasBounds.y1;
                                let mut pointDrawn = false;
                                for pixels in &mut PixelsVec {
                                    if (pixels.color == currentColor) {
                                        pixels.points.push(pushedPoint.into());
                                        pointDrawn = true;
                                        break;
                                    }
                                }
                                if (!pointDrawn) {
                                    let pixels = Pixels {points: vec![pushedPoint.into()], color: currentColor};
                                    PixelsVec.push(pixels);
                                }
                                canvas.set_draw_color(currentColor);
                                let result = canvas.draw_point(point);
                            }
                        }
                    }
                    point2 = point1;
                },
                _ => {}
            }
        }
        if (needsClear){
            canvas.set_draw_color(bgcolor);
            canvas.clear();
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            let result = canvas.fill_rect(FRect::new(canvasBounds.x1 as f32, canvasBounds.y1 as f32, canvasBounds.width() as f32, canvasBounds.height() as f32));
        }
        if (needsDraw){
            let oldViewport = canvas.viewport();
            canvas.set_viewport(Rect::new(canvasBounds.x1, canvasBounds.y1, canvasBounds.width() as u32, canvasBounds.height() as u32));
            for pixels in &PixelsVec {
                canvas.set_draw_color(pixels.color);
                let result2 = canvas.draw_points(pixels.points.as_slice());
            }
            canvas.set_viewport(oldViewport);
        }
        // call at the end of every loop
        canvas.present();
        sleep(Duration::new(0, 500_000_000u32 / 60));
    }
}
