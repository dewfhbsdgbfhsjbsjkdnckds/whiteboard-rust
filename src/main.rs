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

//use sdl3::Error;
use sdl3::Sdl;
use sdl3::VideoSubsystem;
use sdl3::event::Event;
use sdl3::event::WindowEvent;
use sdl3::gpu::Buffer;
use sdl3::gpu::BufferBinding;
use sdl3::gpu::BufferRegion;
use sdl3::gpu::BufferUsageFlags;
use sdl3::gpu::CommandBuffer;
use sdl3::gpu::CopyPass;
use sdl3::gpu::CullMode;
use sdl3::gpu::Device;
use sdl3::gpu::FillMode;
use sdl3::gpu::Filter;
use sdl3::gpu::GraphicsPipeline;
use sdl3::gpu::GraphicsPipelineBuilder;
use sdl3::gpu::GraphicsPipelineTargetInfo;
use sdl3::gpu::IndexElementSize;
use sdl3::gpu::LoadOp;
use sdl3::gpu::RasterizerState;
use sdl3::gpu::SampleCount;
use sdl3::gpu::SamplerAddressMode;
use sdl3::gpu::SamplerCreateInfo;
use sdl3::gpu::SamplerMipmapMode;
use sdl3::gpu::Shader;
use sdl3::gpu::ShaderFormat;
use sdl3::gpu::ShaderStage;
use sdl3::gpu::Texture;
use sdl3::gpu::TextureRegion;
use sdl3::gpu::TextureSamplerBinding;
use sdl3::gpu::TextureTransferInfo;
use sdl3::gpu::TextureType;
use sdl3::gpu::TransferBuffer;
use sdl3::gpu::TransferBufferLocation;
use sdl3::gpu::TransferBufferUsage;
use sdl3::gpu::VertexAttribute;
use sdl3::gpu::VertexBufferDescription;
use sdl3::gpu::VertexElementFormat;
use sdl3::gpu::VertexInputRate;
use sdl3::gpu::VertexInputState;
use sdl3::gpu::Viewport;
use sdl3::gpu::{
    ColorTargetInfo, DepthStencilTargetInfo, StoreOp, TextureCreateInfo, TextureFormat,
    TextureUsage,
};
use sdl3::hint::Hint;
use sdl3::hint::names;
use sdl3::hint::set_with_priority;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::pixels::PixelFormat;
use sdl3::rect::Point;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::render::FPoint;
use sdl3::render::FRect;
use sdl3::sys::gpu::SDL_GPUColorTargetInfo;
use sdl3::sys::keycode::SDLK_SPACE;
use sdl3::sys::video::SDL_WINDOW_RESIZABLE;
use sdl3::video::Window;
use std::error::Error;
use std::ffi::CStr;
use std::fmt::Debug;
use std::ops::Index;
use std::thread::sleep;
use std::time::Duration;

static vertexShaderCode: &'static [u8] = include_bytes!("shaders/vertex.spv");
static fragShaderCode: &'static [u8] = include_bytes!("shaders/frag.spv");

#[derive(Debug)]
struct UPoint {
    x: u32,
    y: u32,
}

impl UPoint {
    fn new(x: u32, y: u32) -> UPoint {
        return UPoint { x, y };
    }
}

//i should make this myself
//#[derive(new)]
#[derive(Debug)]
struct Pixel {
    color: Color,
    point: UPoint,
}

struct Pixels {
    color: Color,
    points: Vec<UPoint>,
}

impl Pixel {
    fn new(color: Color, point: UPoint) -> Pixel {
        return Pixel { color, point };
    }
}

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

#[derive(Debug)]
enum Change {
    pencil { old: Vec<Pixel>, new: Vec<Pixel> },
    // obviously, new is RGBA 0 0 0 0
    eraser { old: Vec<Pixel> },
}

// Changes need to be undone, and redone
// For pencil, i need to store the position and color of each pixel
// same for eraser
// i need to be able to just call whiteboard.changes.undo()
// and whiteboard.changes.redo()

struct Whiteboard {
    canvasBounds: IntRect,
    bgcolor: Color,
    data: Vec<u8>,
    changes: Vec<Change>,
    // starts at 0
    changesIndex: usize,
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
    fn undo(&mut self, textureMem: &mut [u8]) {
        // if index is 0, there are no changes
        if self.changesIndex != 0 && let Some(change) = self.changes.get(self.changesIndex - 1) {
            self.changesIndex -= 1;
            match change {
                Change::pencil { old, new } => {
                    for pixel in old {
                        // i should make this a function or something lol, for index
                        let index: usize = (pixel.point.x as usize
                            + pixel.point.y as usize * self.canvasBounds.width() as usize) * 4;
                        setColorAtIndex(index, textureMem, &pixel.color);
                        setColorAtIndex(index, &mut self.data, &pixel.color);
                    }
                    //textureMem.debugPrint();
                    for i in (0..self.canvasBounds.width()*self.canvasBounds.height()) {
                        let index = i as usize * 4;
                        let color = getColorAtIndex(index, self);
                        if let Some(color) = color {
                            if color.r == 255 {
                                println!("FUCK THIS IS BAD");
                            }
                        }
                    }
                }
                Change::eraser { old } => {
                    // restore old colors of everything
                    for pixel in old {
                        let index: usize = (pixel.point.x as usize
                            + pixel.point.y as usize * self.canvasBounds.width() as usize) * 4;
                        setColorAtIndex(index, textureMem, &pixel.color);
                        setColorAtIndex(index, &mut self.data, &pixel.color);
                    }
                }
            }
        }
    }
    fn redo(&mut self, textureMem: &mut [u8]) {
        if let Some(change) = self.changes.get(self.changesIndex) {
            match change {
                Change::pencil { new, .. } => {
                    for pixel in new {
                        let index: usize = (pixel.point.x as usize
                            + pixel.point.y as usize * self.canvasBounds.width() as usize) * 4;
                        setColorAtIndex(index, textureMem, &pixel.color);
                        setColorAtIndex(index, &mut self.data, &pixel.color);
                    }
                }
                // redoing will just set all of them back to 0 0 0 0
                Change::eraser { old } => {
                    for pixel in old {
                        let index: usize = (pixel.point.x as usize
                            + pixel.point.y as usize * self.canvasBounds.width() as usize) * 4;
                        let color = Color::RGBA(0, 0, 0, 0);
                        setColorAtIndex(index, textureMem, &color);
                        setColorAtIndex(index, &mut self.data, &color);
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

fn applyChangesToTexture(
    whiteboard: &mut Whiteboard,
    device: &Device,
    copyPass: &CopyPass,
    texture: &Texture,
    transferBuffer: &TransferBuffer,
) -> Result<(), sdl3::Error> {
    Ok(())
}

fn writeToTexture<T: Copy>(
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
    textureMem: &mut [u8],
    change: Option<&mut Change>,
) {
    match toolMode {
        ToolMode::none => {}
        ToolMode::movingCanvas => {
            let dx = point1.x - point2.x;
            let dy = point1.y - point2.y;
            whiteboard.canvasBounds.shift(dx, dy);
        }
        ToolMode::pencil => {
            let mut pointList: Vec<UPoint> = Vec::new();
            makeLineInRect(&mut pointList, point1, point2, &whiteboard.canvasBounds);
            let mut oldPixels: Vec<Pixel> = Vec::with_capacity(pointList.len());
            let mut newPixels: Vec<Pixel> = Vec::with_capacity(pointList.len());
            for point in pointList {
                let index = (point.x as usize
                    + (point.y as usize * whiteboard.canvasBounds.width() as usize))
                    * 4;
                let oldColor = getColorAtIndex(index, whiteboard);
                // should always be true i think
                if let Some(oldColor) = oldColor {
                    oldPixels.push(Pixel {
                        color: oldColor,
                        point: UPoint {
                            x: point.x,
                            y: point.y
                        },
                    });
                }
                setColorAtIndex(index, &mut whiteboard.data, &color);
                setColorAtIndex(index, textureMem, &color);
                newPixels.push(Pixel::new(color, point));
            }
            //whiteboard.changes.last()
            if let Some(Change::pencil { old, new }) = change {
                old.append(&mut oldPixels);
                new.append(&mut newPixels);
            }
        }
        // todo fix
        ToolMode::eraser => {
            // i really hope theres a more pretty way to do this :sob:
            let mut pointList: Vec<UPoint> = Vec::new();
            makeLine(&mut pointList, point1, point2);
            for eraserPoint in pointList {
                if (!eraserPoint.isInside(&whiteboard.canvasBounds)) {
                    continue;
                };
                let normalX = (eraserPoint.x as i32 - whiteboard.canvasBounds.x1) as usize;
                let normalY = (eraserPoint.y as i32 - whiteboard.canvasBounds.y1) as usize;
                let index = (normalX + (normalY * whiteboard.canvasBounds.width() as usize)) * 4;
                setColorAtIndex(index, &mut whiteboard.data, &color);
                setColorAtIndex(index, textureMem, &color);
            }
        }
    }
}

fn getColorAtIndex(index: usize, whiteboard: &Whiteboard) -> Option<Color> {
    let r = *whiteboard.data.get(index)?;
    let g = *whiteboard.data.get(index + 1)?;
    let b = *whiteboard.data.get(index + 2)?;
    let a = *whiteboard.data.get(index + 3)?;
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
    let mut data: Vec<u8> =
        Vec::with_capacity(canvasBounds.width() as usize * canvasBounds.height() as usize * 4);
    for _ in 0..(canvasBounds.width() * canvasBounds.height()) {
        data.push(bgcolor.r);
        data.push(bgcolor.g);
        data.push(bgcolor.b);
        data.push(bgcolor.a);
    }
    let mut whiteboard = Whiteboard {
        canvasBounds,
        bgcolor: bgcolor,
        data: data,
        changes: Vec::new(),
        changesIndex: 0,
    };
    let mut toolMode = ToolMode::pencil;
    let mut oldTool = ToolMode::none;

    // needs shader format, such as SPIRV, MSL, etc
    let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&window)?;

    //println!("{:?}", vertexShaderCode);
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
    let mut commandBuffer = device.acquire_command_buffer()?;
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
    let mut event_pump = sdl_context.event_pump().unwrap();
    let /*mut*/ currentColor = Color::RGB(255, 255, 255);
    let mut point1: Point;
    let mut point2: Option<Point> = None;

    // uniform buffer data
    let translateX = 0.0;
    let translateY = 0.0;
    // important!!!
    // 32 bit floats are used (for some reason)
    #[rustfmt::skip]
        let mut data: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ];

    let mut pixelsTexture = device.create_texture(
        TextureCreateInfo::new()
            .with_num_levels(1)
            .with_layer_count_or_depth(1)
            .with_usage(TextureUsage::SAMPLER)
            .with_format(TextureFormat::R8g8b8a8Unorm)
            .with_width(whiteboard.canvasBounds.width())
            .with_height(whiteboard.canvasBounds.height())
            .with_type(TextureType::_2D)
            .with_sample_count(SampleCount::NoMultiSampling),
    )?;
    let pixelsTextureTransferBuffer = device
        .create_transfer_buffer()
        .with_usage(TransferBufferUsage::UPLOAD)
        .with_size(whiteboard.data.len() as u32)
        .build()?;

    {
        let commandBuffer = device.acquire_command_buffer()?;
        let copyPass = device.begin_copy_pass(&commandBuffer)?;
        writeToTexture(
            &device,
            &copyPass,
            whiteboard.data.as_slice(),
            &pixelsTexture,
            &pixelsTextureTransferBuffer,
        );
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

    //let undoTransferBuffer = device
    //    .create_transfer_buffer()
    //    .with_size(whiteboard.canvasBounds.width() * whiteboard.canvasBounds.height() * 4)
    //    .with_usage(TransferBufferUsage::UPLOAD)
    //    .build()?;
    let mut textureBufferMemMap = pixelsTextureTransferBuffer.map(&device, false);
    textureBufferMemMap
        .mem_mut()
        .copy_from_slice(&whiteboard.data);

    let mut change: Option<Box<Change>> = None;
    'running: loop {
        let mut needsDraw = false;
        let mut needsClear = false;
        let mut canvasMoved = false;
        // here i need some "change" variable
        // when you for example, release left click,
        // itll push the pencil changes to the changes variable

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
                                let mut tmp = Change::pencil {
                                    old: Vec::new(),
                                    new: Vec::new(),
                                };
                                change = Some(Box::new(tmp));
                            }
                        }
                        ToolMode::eraser => {
                            if change.is_none() {
                                change = Some(Box::new(Change::eraser { old: Vec::new() }));
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
                                //whiteboard.changes.push(value);
                                if let Some(idk) = change { // take ownership as late as possible
                                    // this logic is wrong lolll
                                    // you wanna insert at whiteboard.changesIndex
                                    whiteboard.changes.push(*idk);
                                    whiteboard.changesIndex += 1;
                                }
                                change = None;
                            }
                            Change::eraser { ref old } => {
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
                    win_event: WindowEvent::Resized(x, y),
                    ..
                } => {
                    //assert_eq!(x as u32, window.size().0);
                    //todo make old size var to make (the middle of) the whiteboard stay in the same position
                    let widthMeow = whiteboard.canvasBounds.width();
                    let heightMeow = whiteboard.canvasBounds.height();
                    whiteboard.canvasBounds.x1 = 0;
                    whiteboard.canvasBounds.y1 = 0;
                    whiteboard.canvasBounds.x2 = widthMeow as i32;
                    whiteboard.canvasBounds.y2 = heightMeow as i32;
                    // i love rust fmt
                    data[3] = ((whiteboard.canvasBounds.x1 as f32
                        + whiteboard.canvasBounds.width().divide(2))
                        / window.size().0 as f32)
                        * 2.0
                        - 1.0;
                    data[7] = -(((whiteboard.canvasBounds.y1 as f32
                        + whiteboard.canvasBounds.height().divide(2))
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
                    if (mouseHeldDown && point2.is_some()) {
                        if (toolMode == ToolMode::pencil || toolMode == ToolMode::eraser) {
                            needsDraw = true;
                        } else if (toolMode == ToolMode::movingCanvas) {
                            canvasMoved = true;
                        }
                        mouseMovement(
                            &mut whiteboard,
                            point1,
                            point2.unwrap(),
                            &toolMode,
                            currentColor,
                            textureBufferMemMap.mem_mut(),
                            if let Some(ref mut x) = change {
                                Some(x.as_mut())
                            } else {
                                None
                            }
                        );
                        //println!("hi");
                        //println!("{:?}", change);
                    }
                    point2 = Some(point1);
                }
                _ => {}
            }
        }

        // submit undo and redo changes
        //textureBufferMemMap.unmap();
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
        data[0] = whiteboard.canvasBounds.width().divide(&window.size().0);
        data[5] = whiteboard.canvasBounds.height().divide(&window.size().1);
        if (canvasMoved) {
            // translation
            data[3] = (whiteboard.canvasBounds.centreF32().0 / window.size().0 as f32) * 2.0 - 1.0;
            data[7] = -((whiteboard.canvasBounds.centreF32().1 / window.size().1 as f32) * 2.0 - 1.0);
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

        let mut swapchainTexture = commandBuffer
            .wait_and_acquire_swapchain_texture(&window)
            .unwrap();
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
