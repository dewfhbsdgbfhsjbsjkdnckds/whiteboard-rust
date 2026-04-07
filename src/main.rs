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

use sdl3::Sdl;
use sdl3::VideoSubsystem;
use sdl3::event::Event;
use sdl3::event::WindowEvent;
use sdl3::gpu::*;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::rect::Point;
use sdl3::rect::Rect;
use sdl3::render::FRect;
use sdl3::sys::video::SDL_WINDOW_RESIZABLE;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Iter;
//use std::collections::hash_set::Iter;
use std::error::Error;
use std::fmt::Debug;
use std::thread::sleep;
use std::time::Duration;

static vertexShaderCode: &'static [u8] = include_bytes!("shaders/vertex.spv");
static fragShaderCode: &'static [u8] = include_bytes!("shaders/frag.spv");

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UPoint {
    x: u32,
    y: u32,
}

impl UPoint {
    fn new(x: u32, y: u32) -> UPoint {
        return UPoint { x, y };
    }
}

// TODO i should make this myself
//#[derive(new)]
#[derive(Debug)]
struct Pixel {
    color: Color,
    point: UPoint,
}

#[derive(Debug)]
struct Pixels {
    inner: HashMap<UPoint, Color>,
}
#[derive(Debug)]
struct Points {
    inner: HashSet<UPoint>,
}
impl Pixels {
    fn append(&mut self, pixels: &mut Vec<Pixel>) {
        while let Some(pixel) = pixels.pop() {
            self.inner.insert(pixel.point, pixel.color);
        }
    }
    fn new() -> Pixels {
        Pixels {
            inner: HashMap::new(),
        }
    }
}
impl Points {
    fn append(&mut self, points: &mut Vec<UPoint>) {
        while let Some(point) = points.pop() {
            self.inner.insert(point);
        }
    }
    fn new() -> Points {
        Points {
            inner: HashSet::new(),
        }
    }
}

struct PixelsIter<'a> {
    iter: Iter<'a, UPoint, Color>,
}
impl<'a> Iterator for PixelsIter<'a> {
    type Item = (&'a UPoint, &'a Color);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
impl<'a> IntoIterator for &'a Pixels {
    type Item = (&'a UPoint, &'a Color);
    type IntoIter = PixelsIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        PixelsIter {
            iter: self.inner.iter(),
        }
    }
}
struct PointsIter<'a> {
    iter: std::collections::hash_set::Iter<'a, UPoint>
}
impl<'a> Iterator for PointsIter<'a> {
    type Item = &'a UPoint;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
impl<'a> IntoIterator for &'a Points {
    type Item = &'a UPoint;
    type IntoIter = PointsIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        PointsIter {
            iter: self.inner.iter(),
        }
    }
}

impl Pixel {
    fn new(color: Color, point: UPoint) -> Pixel {
        return Pixel { color, point };
    }
}

trait drawPixel<T> {
    fn drawPixel(&mut self, pixel: T);
}
trait erasePoint<T> {
    fn erasePoint(&mut self, point: T);
}
trait drawPixelWidth<T, W> {
    fn drawPixel(&mut self, pixel: T, width: W);
}
trait erasePointWidth<T, W> {
    fn erasePoint(&mut self, point: T, width: W);
}

impl drawPixelWidth<(&UPoint, &Color), usize> for &mut [u8] {
    fn drawPixel(&mut self, pixel: (&UPoint, &Color), width: usize) {
        let point = pixel.0;
        let index: usize = (point.x as usize + point.y as usize * width) * 4;
        setColorAtIndex(index, self, pixel.1);
    }
}
impl drawPixelWidth<&Pixel, usize> for &mut [u8] {
    fn drawPixel(&mut self, pixel: &Pixel, width: usize) {
        let point = &pixel.point;
        let index: usize = (point.x as usize + point.y as usize * width) * 4;
        setColorAtIndex(index, self, &pixel.color);
    }
}
impl erasePointWidth<&UPoint, usize> for &mut [u8] {
    fn erasePoint(&mut self, point: &UPoint, width: usize) {
        self.drawPixel((point, &Color::RGBA(0, 0, 0, 0)), width);
    }
}
impl drawPixel<(&UPoint, &Color)> for WhiteboardData {
    fn drawPixel(&mut self, pixel: (&UPoint, &Color)) {
        setColorAtIndex(self.getIndexOfPoint(pixel.0), &mut self.data, pixel.1);
    }
}
impl drawPixel<&Pixel> for WhiteboardData {
    fn drawPixel(&mut self, pixel: &Pixel) {
        setColorAtIndex(
            self.getIndexOfPoint(&pixel.point),
            &mut self.data,
            &pixel.color,
        );
    }
}
impl erasePoint<&UPoint> for WhiteboardData {
    fn erasePoint(&mut self, point: &UPoint) {
        self.drawPixel((point, &Color::RGBA(0, 0, 0, 0)));
    }
}
impl drawPixel<(&UPoint, &Color)> for Whiteboard {
    fn drawPixel(&mut self, pixel: (&UPoint, &Color)) {
        self.data.drawPixel(pixel);
    }
}
impl drawPixel<&Pixel> for Whiteboard {
    fn drawPixel(&mut self, pixel: &Pixel) {
        self.data.drawPixel(pixel);
    }
}
impl erasePoint<&UPoint> for Whiteboard {
    fn erasePoint(&mut self, point: &UPoint) {
        self.data.erasePoint(point);
    }
}

#[derive(Debug)]
struct IntRect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

#[repr(packed)]
#[derive(Clone, Copy)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    u: f32,
    v: f32,
}

// maybe i could do this with dynamic dispatch instead of enums
#[derive(Debug)]
enum Change {
    pencil { old: Pixels, new: Pixels },
    // obviously, new is RGBA 0 0 0 0
    eraser { new: Points, old: Pixels },
}

// Changes need to be undone, and redone
// For pencil, i need to store the position and color of each pixel
// same for eraser
// i need to be able to just call whiteboard.changes.undo()
// and whiteboard.changes.redo()

#[derive(Debug)]
struct WhiteboardData {
    canvasBounds: IntRect,
    data: Vec<u8>,
}
impl WhiteboardData {
    fn getIndexOfPoint(&self, point: &UPoint) -> usize {
        return (point.x as usize + (point.y as usize * self.canvasBounds.width() as usize)) * 4;
    }
}

#[derive(Debug)]
struct Whiteboard {
    data: WhiteboardData,
    bgcolor: Color,
    changes: Vec<Change>,
    // starts at 0
    changesIndex: usize,
}

impl Whiteboard {
    // this should return a result or something lol
    fn getIndexOfPoint(&self, point: &UPoint) -> usize {
        self.data.getIndexOfPoint(point)
    }
    fn getColorAtPoint(&self, point: &UPoint) -> Option<Color> {
        return getColorAtIndex(self.getIndexOfPoint(point), &self.data.data);
    }
    fn width(&self) -> u32 {
        return self.data.canvasBounds.width();
    }
    fn height(&self) -> u32 {
        return self.data.canvasBounds.height();
    }
    fn x1(&self) -> i32 {
        return self.data.canvasBounds.x1;
    }
    fn x2(&self) -> i32 {
        return self.data.canvasBounds.x2;
    }
    fn y1(&self) -> i32 {
        return self.data.canvasBounds.y1;
    }
    fn y2(&self) -> i32 {
        return self.data.canvasBounds.y2;
    }
    fn x1_mut(&mut self) -> &mut i32 {
        return &mut self.data.canvasBounds.x1;
    }
    fn x2_mut(&mut self) -> &mut i32 {
        return &mut self.data.canvasBounds.x2;
    }
    fn y1_mut(&mut self) -> &mut i32 {
        return &mut self.data.canvasBounds.y1;
    }
    fn y2_mut(&mut self) -> &mut i32 {
        return &mut self.data.canvasBounds.y2;
    }
    fn centreF32(&self) -> (f32, f32) {
        return self.data.canvasBounds.centreF32();
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum ToolMode {
    none,
    pencil,
    eraser,
    movingCanvas,
}

//trait undo {
//}
trait Undo {
    fn undo(&mut self, textureMem: &mut [u8]);
    fn redo(&mut self, textureMem: &mut [u8]);
}

impl Undo for Whiteboard {
    fn undo(&mut self, mut textureMem: &mut [u8]) {
        // if index is 0, there are no changes
        if self.changesIndex != 0
            && let Some(change) = self.changes.get(self.changesIndex - 1)
        {
            self.changesIndex -= 1;
            match change {
                Change::pencil { old, new } => {
                    for pixel in old {
                        let width = self.width() as usize;
                        self.data.drawPixel(pixel);
                        textureMem.drawPixel(pixel, width);
                    }
                }
                Change::eraser { old, .. } => {
                    // restore old colors of everything
                    for pixel in old {
                        let width = self.width() as usize;
                        self.data.drawPixel(pixel);
                        textureMem.drawPixel(pixel, width);
                    }
                }
            }
        }
    }
    fn redo(&mut self, mut textureMem: &mut [u8]) {
        if let Some(change) = self.changes.get(self.changesIndex) {
            self.changesIndex += 1;
            match change {
                Change::pencil { new, .. } => {
                    for pixel in new {
                        let width = self.width() as usize;
                        self.data.drawPixel(pixel);
                        textureMem.drawPixel(pixel, width);
                    }
                }
                // redoing will just set all of them back to 0 0 0 0
                Change::eraser { new, .. } => {
                    for point in new {
                        let width = self.width() as usize;
                        self.data.erasePoint(point);
                        textureMem.erasePoint(point, width);
                    }
                }
            }
        }
    }
}

fn setColorAtIndex(index: usize, data: &mut [u8], color: &Color) {
    if let Some(element) = data.get_mut(index) {
        *element = color.r;
    }
    if let Some(element) = data.get_mut(index + 1) {
        *element = color.g;
    }
    if let Some(element) = data.get_mut(index + 2) {
        *element = color.b;
    }
    if let Some(element) = data.get_mut(index + 3) {
        *element = color.a;
    }
}

trait meow {
    fn isInside(&self, square: &IntRect) -> bool;
}
trait DebugPrint {
    fn debugPrint(&self);
}
impl<T: Debug> DebugPrint for T {
    fn debugPrint(&self) {
        println!("{:?}", self);
    }
}

// i could probably write a macro for this
trait divide<Rhs, Out> {
    fn divide(&self, denominator: Rhs) -> Out;
}
impl divide<&u8, f32> for u8 {
    fn divide(&self, denominator: &u8) -> f32 {
        return *self as f32 / *denominator as f32;
    }
}
impl divide<&u32, f32> for u32 {
    fn divide(&self, denominator: &u32) -> f32 {
        return *self as f32 / *denominator as f32;
    }
}
impl divide<i32, f32> for u32 {
    fn divide(&self, denominator: i32) -> f32 {
        return *self as f32 / denominator as f32;
    }
}
impl divide<&u32, f32> for i32 {
    fn divide(&self, denominator: &u32) -> f32 {
        return *self as f32 / *denominator as f32;
    }
}
impl divide<&i32, f32> for i32 {
    fn divide(&self, denominator: &i32) -> f32 {
        return *self as f32 / *denominator as f32;
    }
}

trait myInto<T> {
    fn into(&self) -> T;
}
impl myInto<[f32; 4]> for Color {
    fn into(&self) -> [f32; 4] {
        return [
            self.r.divide(&u8::MAX),
            self.g.divide(&u8::MAX),
            self.b.divide(&u8::MAX),
            self.a.divide(&u8::MAX),
        ];
    }
}

impl meow for Point {
    fn isInside(&self, square: &IntRect) -> bool {
        return self.x >= square.x1
            && self.y >= square.y1
            && self.x < square.x2
            && self.y < square.y2;
    }
}
impl meow for UPoint {
    fn isInside(&self, square: &IntRect) -> bool {
        return (self.x as i32) >= square.x1
            && (self.y as i32) >= square.y1
            && (self.x as i32) < square.x1
            && (self.y as i32) < square.y2;
    }
}
fn getWidth(rect: &IntRect) -> u32 {
    return (rect.x2 - rect.x1) as u32;
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
    fn centre(&self) -> (i32, i32) {
        let x = self.x1 + self.width() as i32 / 2;
        let y = self.y1 + self.height() as i32 / 2;
        return (x, y);
    }
    fn centreF32(&self) -> (f32, f32) {
        let x = self.x1 as f32 + self.width() as f32 / 2.0;
        let y = self.y1 as f32 + self.height() as f32 / 2.0;
        return (x, y);
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
fn makeLine(list: &mut Vec<UPoint>, point1: Point, point2: Point) {
    let mut x1 = point1.x;
    let mut y1 = point1.y;
    let x2 = point2.x;
    let y2 = point2.y;
    let sx = if (x1 < x2) { 1 } else { -1 };
    let sy = if (y1 < y2) { 1 } else { -1 };

    // case for vertical line
    if x1 == x2 {
        loop {
            list.push(UPoint::new(x1 as u32, y1 as u32));
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
            list.push(UPoint::new(x1 as u32, y1 as u32));
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
        list.push(UPoint::new(x1 as u32, y1 as u32));
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

fn makeLineInRect(list: &mut Vec<UPoint>, point1: Point, point2: Point, rect: &IntRect) {
    let mut x1 = point1.x;
    let mut y1 = point1.y;
    let x2 = point2.x;
    let y2 = point2.y;
    let sx = if (x1 < x2) { 1 } else { -1 };
    let sy = if (y1 < y2) { 1 } else { -1 };

    // case for vertical line
    if x1 == x2 {
        loop {
            if Point::new(x1, y1).isInside(rect) {
                list.push(UPoint::new((x1 - rect.x1) as u32, (y1 - rect.y1) as u32));
            }
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
            if Point::new(x1, y1).isInside(rect) {
                list.push(UPoint::new((x1 - rect.x1) as u32, (y1 - rect.y1) as u32));
            }
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
        if Point::new(x1, y1).isInside(rect) {
            list.push(UPoint::new((x1 - rect.x1) as u32, (y1 - rect.y1) as u32));
        }
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

fn loadShader(
    device: &Device,
    code: &[u8],
    shaderStage: ShaderStage,
    samplerCount: u32,
    uniformBufferCount: u32,
    storageBufferCount: u32,
    storageTextureCount: u32,
) -> Result<Shader, sdl3::Error> {
    // todo i dont really need this function since im just doing "include bytes"
    let entrypoint;
    let shader_format = device.get_shader_formats();
    match shader_format {
        ShaderFormat::SPIRV => entrypoint = c"main",
        _ => entrypoint = c"main",
    }
    let shader = device
        .create_shader()
        .with_samplers(samplerCount)
        .with_uniform_buffers(uniformBufferCount)
        .with_storage_buffers(storageBufferCount)
        .with_storage_textures(storageTextureCount)
        .with_entrypoint(entrypoint)
        .with_code(device.get_shader_formats(), code, shaderStage)
        .build()?;
    return Ok(shader);
}

fn writeToTexture<T: Copy + Sized>(
    device: &Device,
    copyPass: &CopyPass,
    data: &[T],
    texture: &Texture,
    transferBuffer: &TransferBuffer,
) -> Result<(), sdl3::Error> {
    let mut bufferMem = transferBuffer.map(device, false);
    // copy from slice is laggy. Instead i should make a function to only push changes
    bufferMem.mem_mut().copy_from_slice(data);
    bufferMem.unmap();

    copyPass.upload_to_gpu_texture(
        TextureTransferInfo::new()
            .with_transfer_buffer(transferBuffer)
            .with_offset(0),
        TextureRegion::new()
            .with_texture(&texture)
            .with_width(texture.width())
            .with_height(texture.height())
            .with_depth(1),
        false,
    );
    return Ok(());
}

fn mouseMovement(
    whiteboard: &mut Whiteboard,
    point1: Point,
    point2: Point,
    toolMode: &ToolMode,
    color: Color,
    mut textureMem: &mut [u8],
    change: Option<&mut Change>,
) {
    match toolMode {
        ToolMode::none => {}
        ToolMode::movingCanvas => {
            let dx = point1.x - point2.x;
            let dy = point1.y - point2.y;
            whiteboard.data.canvasBounds.shift(dx, dy);
        }
        ToolMode::pencil => {
            let mut pointList: Vec<UPoint> = Vec::new();
            makeLineInRect(
                &mut pointList,
                point1,
                point2,
                &whiteboard.data.canvasBounds,
            );
            let mut oldPixels: Vec<Pixel> = Vec::with_capacity(pointList.len());
            let mut newPixels: Vec<Pixel> = Vec::with_capacity(pointList.len());
            for point in pointList {
                textureMem.drawPixel((&point, &color), whiteboard.width() as usize);
                let oldColor = whiteboard.getColorAtPoint(&point);
                if let Some(oldColor) = oldColor
                    && (color != oldColor)
                {
                    oldPixels.push(Pixel::new(oldColor, point.clone()));
                    newPixels.push(Pixel::new(color, point));
                }
            }
            if let Some(Change::pencil { old, new }) = change {
                old.append(&mut oldPixels);
                new.append(&mut newPixels);
            }
        }
        // todo fix
        ToolMode::eraser => {
            let mut pointList: Vec<UPoint> = Vec::new();
            makeLineInRect(
                &mut pointList,
                point1,
                point2,
                &whiteboard.data.canvasBounds,
            );
            let mut oldPixels: Vec<Pixel> = Vec::with_capacity(pointList.len());
            //let mut erasedPoints: Vec<UPoint> = Vec::with_capacity(pointList.len());
            for eraserPoint in &pointList {
                textureMem.erasePoint(&eraserPoint, whiteboard.width() as usize);
                let oldColor = whiteboard.getColorAtPoint(&eraserPoint);
                if let Some(oldColor) = oldColor
                    && (oldColor != Color::RGBA(0, 0, 0, 0))
                {
                    oldPixels.push(Pixel::new(oldColor, eraserPoint.clone()));
                }
            }
            if let Some(Change::eraser { old, new }) = change {
                old.append(&mut oldPixels);
                new.append(&mut pointList);
            }
        }
    }
}

fn getColorAtIndex(index: usize, data: &[u8]) -> Option<Color> {
    let r = *data.get(index)?;
    let g = *data.get(index + 1)?;
    let b = *data.get(index + 2)?;
    let a = *data.get(index + 3)?;
    return Some(Color::RGBA(r, g, b, a));
}

fn createBufferWithData<T: Copy>(
    device: &Device,
    copyPass: &CopyPass,
    transferBuffer: &TransferBuffer,
    bufferUsageFlags: BufferUsageFlags,
    data: &[T],
) -> Result<Buffer, sdl3::Error> {
    let dataSize = size_of_val(data) as u32;
    let buffer = device
        .create_buffer()
        .with_size(dataSize)
        .with_usage(bufferUsageFlags)
        .build()?;

    let mut bufferMemMap = transferBuffer.map(device, true);
    let map: &mut [T] = bufferMemMap.mem_mut();
    //map.clone_from_slice(data);
    // done copying to memory
    // maybe test to see if this works better
    for (index, &value) in data.iter().enumerate() {
        map[index] = value;
    }
    // todo clone from slice should work, and it might be faster as well
    //println!("{:?}", map);
    bufferMemMap.unmap();

    copyPass.upload_to_gpu_buffer(
        TransferBufferLocation::new()
            .with_transfer_buffer(&transferBuffer)
            .with_offset(0),
        BufferRegion::new()
            .with_buffer(&buffer)
            // todo add variable for this later or something
            .with_offset(0)
            .with_size(dataSize),
        true,
    );

    return Ok(buffer);
}

// todo
// undo feature
// different brush sizes
// change brush colour
// add inserting text
fn main() -> Result<(), Box<dyn Error>> {
    //set_with_priority(names::VIDEO_DRIVER, "wayland", &Hint::Override);
    let sdl_context: Sdl = sdl3::init()?;
    let video_subsystem: VideoSubsystem = sdl_context.video()?;

    const width: u32 = 900;
    const height: u32 = 500;
    let window = video_subsystem
        .window("window", width, height)
        // todo find the rust version of this flag
        .set_window_flags(SDL_WINDOW_RESIZABLE as u32) // casting? still works?
        .position_centered()
        .build()?;

    let canvasBounds: IntRect = IntRect {
        x1: 0,
        y1: 0,
        x2: width as i32,
        y2: height as i32,
    };

    let bgcolor = Color::RGB(80, 80, 80);
    // vec len is 1.8 million
    let mut data: Vec<u8> =
        Vec::with_capacity(canvasBounds.width() as usize * canvasBounds.height() as usize * 4);
    for _ in 0..(canvasBounds.width() * canvasBounds.height()) {
        data.push(bgcolor.r);
        data.push(bgcolor.g);
        data.push(bgcolor.b);
        data.push(bgcolor.a);
    }
    //(canvasBounds.width() * canvasBounds.height() * 4).debugPrint();
    let mut whiteboard = Whiteboard {
        data: WhiteboardData {
            canvasBounds: canvasBounds,
            data: data
        },
        bgcolor: bgcolor,
        changes: Vec::new(),
        changesIndex: 0,
    };
    let mut toolMode = ToolMode::pencil;
    let mut oldTool = ToolMode::none;

    // needs shader format, such as SPIRV, MSL, etc
    let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&window)?;

    let ColorTargetDescriptions: &[sdl3::gpu::ColorTargetDescription] =
        &[sdl3::gpu::ColorTargetDescription::new()
            .with_format(device.get_swapchain_texture_format(&window))];
    // send a matrix as a uniform to transform the rectangle
    // x1, y1, x2, and y2 would be obtained from dividing the int position by the window size
    let vertexShader = loadShader(&device, vertexShaderCode, ShaderStage::Vertex, 0, 1, 0, 0)?;
    // uniform buffer for the colour because i think the colour can change
    let fragShader = loadShader(&device, fragShaderCode, ShaderStage::Fragment, 1, 1, 0, 0)?;
    let pipeline = device
        .create_graphics_pipeline()
        .with_vertex_shader(&vertexShader)
        .with_fragment_shader(&fragShader)
        .with_target_info(
            GraphicsPipelineTargetInfo::new()
                .with_color_target_descriptions(ColorTargetDescriptions),
        )
        .with_vertex_input_state(
            VertexInputState::new()
                .with_vertex_buffer_descriptions(&[VertexBufferDescription::new()
                    .with_slot(0)
                    .with_instance_step_rate(0)
                    .with_pitch(size_of::<Vertex>() as u32)
                    .with_input_rate(VertexInputRate::Vertex)])
                .with_vertex_attributes(&[
                    VertexAttribute::new()
                        .with_format(VertexElementFormat::Float3)
                        .with_buffer_slot(0)
                        .with_location(0)
                        .with_offset(0),
                    VertexAttribute::new()
                        .with_format(VertexElementFormat::Float2)
                        .with_location(1)
                        .with_buffer_slot(0)
                        .with_offset((size_of::<f32>() * 3) as u32),
                ]),
        )
        .with_primitive_type(sdl3::gpu::PrimitiveType::TriangleList)
        .with_fill_mode(FillMode::Fill)
        .with_rasterizer_state(
            RasterizerState::new()
                .with_fill_mode(FillMode::Fill)
                .with_cull_mode(CullMode::None),
        )
        .build()?;
    drop(vertexShader);
    drop(fragShader);

    // copying vertex data to gpu
    let commandBuffer = device.acquire_command_buffer()?;
    let copyPass = device.begin_copy_pass(&commandBuffer)?;
    #[rustfmt::skip]
    let vertexData: &[Vertex] = &[
        Vertex {x:  1.0, y:  1.0, z: 0.0, u: 1.0, v: 0.0},
        Vertex {x:  1.0, y: -1.0, z: 0.0, u: 1.0, v: 1.0},
        Vertex {x: -1.0, y: -1.0, z: 0.0, u: 0.0, v: 1.0},
        Vertex {x: -1.0, y:  1.0, z: 0.0, u: 0.0, v: 0.0}
    ];

    #[rustfmt::skip]
    let vertexIndicies: &[u16] = &[
        0, 1, 3,
        1, 2, 3
    ];
    let transferBuffer = device
        .create_transfer_buffer()
        .with_usage(TransferBufferUsage::UPLOAD)
        .with_size((size_of_val(vertexIndicies) as u32).max(size_of_val(vertexData) as u32))
        .build()?;
    let vertexBuffer = createBufferWithData(
        &device,
        &copyPass,
        &transferBuffer,
        BufferUsageFlags::VERTEX,
        vertexData,
    )?;
    let indexBuffer = createBufferWithData(
        &device,
        &copyPass,
        &transferBuffer,
        BufferUsageFlags::INDEX,
        vertexIndicies,
    )?;
    drop(transferBuffer);
    device.end_copy_pass(copyPass);
    commandBuffer.submit()?;

    let mut mouseHeldDown = false;
    let mut event_pump = sdl_context.event_pump()?;
    let /*mut*/ currentColor = Color::RGB(255, 255, 255);
    let mut point1: Point;
    let mut point2: Option<Point> = None;

    // uniform buffer data
    //let translateX = 0.0;
    //let translateY = 0.0;
    // important!!!
    // 32 bit floats are used (for some reason)
    #[rustfmt::skip]
    let mut data: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    ];

    let pixelsTexture = device.create_texture(
        TextureCreateInfo::new()
            .with_num_levels(1)
            .with_layer_count_or_depth(1)
            .with_usage(TextureUsage::SAMPLER)
            .with_format(TextureFormat::R8g8b8a8Unorm)
            .with_width(whiteboard.width())
            .with_height(whiteboard.height())
            .with_type(TextureType::_2D)
            .with_sample_count(SampleCount::NoMultiSampling),
    )?;
    let pixelsTextureTransferBuffer = device
        .create_transfer_buffer()
        .with_usage(TransferBufferUsage::UPLOAD)
        .with_size(whiteboard.data.data.len() as u32)
        .build()?;

    {
        let commandBuffer = device.acquire_command_buffer()?;
        let copyPass = device.begin_copy_pass(&commandBuffer)?;
        // theres an error here lol
        let x = whiteboard.data.data.as_slice();
        writeToTexture(
            &device,
            &copyPass,
            x,
            &pixelsTexture,
            &pixelsTextureTransferBuffer,
        )?;
        device.end_copy_pass(copyPass);
        commandBuffer.submit()?;
    }

    let pixelsTextureSampler = device.create_sampler(
        SamplerCreateInfo::new()
            .with_min_filter(Filter::Nearest)
            .with_mag_filter(Filter::Nearest)
            .with_mipmap_mode(SamplerMipmapMode::Nearest)
            .with_address_mode_u(SamplerAddressMode::ClampToEdge)
            .with_address_mode_v(SamplerAddressMode::ClampToEdge)
            .with_address_mode_w(SamplerAddressMode::ClampToEdge),
    )?;

    let mut textureBufferMemMap = pixelsTextureTransferBuffer.map(&device, false);
    textureBufferMemMap
        .mem_mut()
        .copy_from_slice(&whiteboard.data.data);

    let mut change: Option<Box<Change>> = None;
    'running: loop {
        let mut needsDraw = false;
        let mut needsClear = false;
        let mut canvasMoved = false;

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
                    match toolMode {
                        ToolMode::pencil => {
                            if change.is_none() {
                                change = Some(Box::new(Change::pencil {
                                    old: Pixels::new(),
                                    new: Pixels::new(),
                                }));
                            }
                        }
                        ToolMode::eraser => {
                            if change.is_none() {
                                change = Some(Box::new(Change::eraser {
                                    old: Pixels::new(),
                                    new: Points::new()
                                }));
                            }
                        }
                        _ => {}
                    }
                }
                Event::MouseButtonUp { .. } => {
                    mouseHeldDown = false;
                    point2 = None;
                    // take ownership and reassign
                    if let Some(meowtuah) = &change {
                        match **meowtuah {
                            Change::pencil { ref old, ref new } => {
                                // we dont push the change to the canvas itself until you release mouse click
                                // this makes it so undo will bring you back to the state before you did anything
                                for pixel in new {
                                    whiteboard.drawPixel(pixel);
                                }
                                if let Some(idk) = change {
                                    // take ownership as late as possible
                                    // this logic is wrong lolll
                                    // you wanna insert at whiteboard.changesIndex
                                    if whiteboard.changesIndex == whiteboard.changes.len() {
                                        whiteboard.changes.push(*idk);
                                        whiteboard.changesIndex += 1;
                                    } else if let Some(change) =
                                        whiteboard.changes.get_mut(whiteboard.changesIndex)
                                    {
                                        *change = *idk;
                                        whiteboard.changes.truncate(whiteboard.changesIndex + 1);
                                        whiteboard.changesIndex += 1;
                                    }
                                }
                                change = None;
                            }
                            Change::eraser { ref old, ref new} => {
                                for point in new {
                                    whiteboard.erasePoint(point);
                                }
                                if let Some(idk) = change {
                                    whiteboard.changes.push(*idk);
                                    whiteboard.changesIndex += 1;
                                }
                                change = None;
                            }
                        }
                    }
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
                Event::Window {
                    win_event: WindowEvent::Resized(..),
                    ..
                } => {
                    //assert_eq!(x as u32, window.size().0);
                    //todo make old size var to make (the middle of) the whiteboard stay in the same position
                    let widthMeow = whiteboard.width();
                    let heightMeow = whiteboard.height();
                    *whiteboard.x1_mut() = 0;
                    *whiteboard.y1_mut() = 0;
                    *whiteboard.x2_mut() = widthMeow as i32;
                    *whiteboard.y2_mut() = heightMeow as i32;
                    // i love rust fmt
                    data[3] = ((whiteboard.x1() as f32
                        + whiteboard.width().divide(2))
                        / window.size().0 as f32)
                        * 2.0
                        - 1.0;
                    data[7] = -(((whiteboard.y1() as f32
                        + whiteboard.height().divide(2))
                        / window.size().1 as f32)
                        * 2.0
                        - 1.0);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::U),
                    ..
                } => {
                    whiteboard.undo(textureBufferMemMap.mem_mut());
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    whiteboard.redo(textureBufferMemMap.mem_mut());
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
                    if let Some(point2) = point2 && mouseHeldDown {
                    //if (mouseHeldDown && let Some(point2) = point2) {
                        if (toolMode == ToolMode::pencil || toolMode == ToolMode::eraser) {
                            needsDraw = true;
                        } else if (toolMode == ToolMode::movingCanvas) {
                            canvasMoved = true;
                        }
                        mouseMovement(
                            &mut whiteboard,
                            point1,
                            point2,
                            &toolMode,
                            currentColor,
                            textureBufferMemMap.mem_mut(),
                            if let Some(ref mut x) = change {
                                Some(x.as_mut())
                            } else {
                                None
                            },
                        );
                    }
                    point2 = Some(point1);
                }
                _ => {}
            }
        }

        // submit undo and redo changes
        {
            let commandBuffer = device.acquire_command_buffer()?;
            let copyPass = device.begin_copy_pass(&commandBuffer)?;
            copyPass.upload_to_gpu_texture(
                TextureTransferInfo::new()
                    .with_transfer_buffer(&pixelsTextureTransferBuffer)
                    .with_offset(0),
                TextureRegion::new()
                    .with_texture(&pixelsTexture)
                    .with_height(pixelsTexture.height())
                    .with_width(pixelsTexture.width())
                    .with_depth(1),
                false,
            );
            device.end_copy_pass(copyPass);
            commandBuffer.submit()?;
        }

        // scaling
        data[0] = whiteboard.width().divide(&window.size().0);
        data[5] = whiteboard.height().divide(&window.size().1);
        if (canvasMoved) {
            // translation
            data[3] = (whiteboard.centreF32().0 / window.size().0 as f32) * 2.0 - 1.0;
            data[7] =
                -((whiteboard.centreF32().1 / window.size().1 as f32) * 2.0 - 1.0);
        }

        // copy pixels to texture
        //if (needsDraw) {
        //    let commandBuffer = device.acquire_command_buffer()?;
        //    let copyPass = device.begin_copy_pass(&commandBuffer)?;
        //    writeToTexture(
        //        &device,
        //        &copyPass,
        //        whiteboard.data.as_slice(),
        //        &pixelsTexture,
        //        &pixelsTextureTransferBuffer,
        //    );
        //    device.end_copy_pass(copyPass);
        //    commandBuffer.submit()?;
        //}

        // DRAWING
        let mut commandBuffer = device.acquire_command_buffer()?;

        let swapchainTexture = commandBuffer
            .wait_and_acquire_swapchain_texture(&window)?;
        let targetInfo: ColorTargetInfo = ColorTargetInfo::default()
            .with_texture(&swapchainTexture)
            .with_clear_color(Color::BLACK)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE);

        // begin render pass
        let renderPass = device.begin_render_pass(
            &commandBuffer,
            &[targetInfo],
            /*Some(&depthStencilTarget)*/ None,
        )?;
        renderPass.bind_graphics_pipeline(&pipeline);

        renderPass.bind_vertex_buffers(
            0,
            &[BufferBinding::new()
                .with_buffer(&vertexBuffer)
                .with_offset(0)],
        );
        renderPass.bind_index_buffer(
            &BufferBinding::new()
                .with_buffer(&indexBuffer)
                .with_offset(0),
            IndexElementSize::_16BIT,
        );
        renderPass.bind_fragment_samplers(
            0,
            &[TextureSamplerBinding::new()
                .with_texture(&pixelsTexture)
                .with_sampler(&pixelsTextureSampler)],
        );
        // first number should be the same as the binding i think
        commandBuffer.push_vertex_uniform_data(0, &data);
        commandBuffer.push_fragment_uniform_data(0, &myInto::into(&whiteboard.bgcolor));
        renderPass.draw_indexed_primitives(vertexIndicies.len() as u32, 1, 0, 0, 0);
        //renderPass.draw_primitives(3, 1, 0, 0);
        //renderPass.draw_indexed_primitives(6, 1, 0, 0, 0);

        device.end_render_pass(renderPass);
        commandBuffer.submit()?;

        //let texInfo = TextureCreateInfo::new()
        //    // sometimes the texture should be smaller because not all of the canvas is on screen
        //    .with_width(whiteboard.canvasBounds.width())
        //    .with_height(whiteboard.canvasBounds.height())
        //    .with_num_levels(1)
        //    .with_layer_count_or_depth(1)
        //    .with_usage(TextureUsage::SAMPLER)
        //    .with_format(TextureFormat::R8g8b8a8UnormSrgb);

        // render a single quad that covers the screen and give it a texture?
        // color is rgba, 8 bits each, 32 bits in total, u32
        if (needsClear) {}
        if (needsDraw) {}
        //whiteboard.pixels.debugPrint();

        // call at the end of every loop
        let targetFPS = 500;
        sleep(Duration::new(0, 1_000_000_000u32 / targetFPS));
        //sdl3::timer::delay(16);
    }
    return Ok(());
}
