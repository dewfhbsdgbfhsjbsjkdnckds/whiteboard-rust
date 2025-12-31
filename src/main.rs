#![allow(unused_variables, non_snake_case, non_upper_case_globals, unused_imports, unused_parens)]
//extern crate sdl3;

use sdl3::pixels::Color;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::Sdl;
use sdl3::VideoSubsystem;
use sdl3::rect::Point;
use sdl3::sys::video::SDL_SetWindowResizable;
use sdl3::sys::video::SDL_WINDOW_RESIZABLE;
use std::time::Duration;

struct Pixel {
    point: Point,
    color: Color,
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


fn main() {
    let sdl_context: Sdl = sdl3::init().unwrap();
    let video_subsystem:VideoSubsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("whiteboard", 1000, 700)
        .set_window_flags(SDL_WINDOW_RESIZABLE as u32) // casting? still works?
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut mouseHeldDown = false;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let /*mut*/ currentColor = Color::RGB(255, 255, 255);
    // resizing the screen gets rid of everything
    // this means i need to store all the data in an arraylist
    let mut whiteBoardData: Vec<Pixel> = Vec::new();
    let mut point1: Option<Point>;
    let mut point2: Option<Point> = None;
    'running: loop {
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
                Event::MouseMotion {x, y, ..} => {
                    if (mouseHeldDown){
                        let mut pointList: Vec<Point> = Vec::new();
                        point1 = Some(Point::new(x as i32, y as i32));
                        if (point1.is_some() && point2.is_some()) {
                            makeLine(&mut pointList, point1.unwrap(), point2.unwrap());
                            for point in pointList.into_iter() {
                                let pixel: Pixel = Pixel { point, color: (currentColor) };
                                whiteBoardData.push(pixel);
                            }
                        }
                        point2 = point1;
                    }
                },
                _ => {}

            }
        }
        for pixel in &whiteBoardData {
            canvas.set_draw_color(pixel.color);
            let result = canvas.draw_point(pixel.point);
        }
        // call at the end of every loop
        canvas.present();
    }
}
