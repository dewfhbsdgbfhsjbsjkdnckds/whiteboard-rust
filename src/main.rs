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

use sdl3::Error;
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
use sdl3::gpu::GraphicsPipeline;
use sdl3::gpu::GraphicsPipelineBuilder;
use sdl3::gpu::GraphicsPipelineTargetInfo;
use sdl3::gpu::IndexElementSize;
use sdl3::gpu::LoadOp;
use sdl3::gpu::RasterizerState;
use sdl3::gpu::Shader;
use sdl3::gpu::ShaderFormat;
use sdl3::gpu::ShaderStage;
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
use sdl3::rect::Point;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::render::FPoint;
use sdl3::render::FRect;
use sdl3::sys::gpu::SDL_GPUColorTargetInfo;
use sdl3::sys::keycode::SDLK_SPACE;
use sdl3::sys::video::SDL_WINDOW_RESIZABLE;
use sdl3::video::Window;
use std::ffi::CStr;
use std::fmt::Debug;
use std::ops::Index;
use std::thread::sleep;
use std::time::Duration;

static vertexShaderCode: &'static [u8] = include_bytes!("shaders/vertex.spv");
static fragShaderCode: &'static [u8] = include_bytes!("shaders/frag.spv");

enum myoption<T> {
    NotAny,
    Exists(T),
}

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
trait ColorThing {
    fn toF32Arr4(&self) -> [f32; 4];
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
impl divide<&u32, f32> for i32 {
    fn divide(&self, denominator: &u32) -> f32 {
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

fn loadShader(
    device: &Device,
    code: &[u8],
    shaderStage: ShaderStage,
    samplerCount: u32,
    uniformBufferCount: u32,
    storageBufferCount: u32,
    storageTextureCount: u32,
) -> Result<Shader, String> {
    let entrypoint;
    let shader_format = device.get_shader_formats();
    match shader_format {
        ShaderFormat::SPIRV => entrypoint = c"main",
        _ => return Err("Unrecognised shader format".to_string()),
    }
    let shader = device
        .create_shader()
        .with_samplers(samplerCount)
        .with_uniform_buffers(uniformBufferCount)
        .with_storage_buffers(storageBufferCount)
        .with_storage_textures(storageTextureCount)
        .with_entrypoint(entrypoint)
        .with_code(device.get_shader_formats(), code, shaderStage)
        .build();
    if let Ok(shader) = shader {
        return Ok(shader);
    } else {
        return Err("shader failed to build".to_string());
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

fn createBufferWithData<T: Copy + Debug>(
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
    println!("{:?}", map);
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //set_with_priority(names::VIDEO_DRIVER, "wayland", &Hint::Override);
    let sdl_context: Sdl = sdl3::init()?;
    let video_subsystem: VideoSubsystem = sdl_context.video()?;

    const width: u32 = 900;
    const height: u32 = 500;
    let window = video_subsystem
        .window("window", width, height)
        .set_window_flags(SDL_WINDOW_RESIZABLE as u32) // casting? still works?
        .position_centered()
        .build()?;

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
        bgcolor: Color::RGB(80, 80, 80),
    };
    let mut toolMode = ToolMode::pencil;
    let mut oldTool = ToolMode::none;

    // needs shader format, such as SPIRV, MSL, etc
    let device = Device::new(sdl3::gpu::ShaderFormat::SPIRV | ShaderFormat::DXIL, true)?
        .with_window(&window)?;

    //println!("{:?}", vertexShaderCode);
    let ColorTargetDescriptions: &[sdl3::gpu::ColorTargetDescription] =
        &[sdl3::gpu::ColorTargetDescription::new()
            .with_format(device.get_swapchain_texture_format(&window))];
    // send a matrix as a uniform to transform the rectangle
    // x1, y1, x2, and y2 would be obtained from dividing the int position by the window size
    let vertexShader = loadShader(&device, vertexShaderCode, ShaderStage::Vertex, 0, 1, 0, 0)?;
    // uniform buffer for the colour because i think the colour can change
    let fragShader = loadShader(&device, fragShaderCode, ShaderStage::Fragment, 0, 1, 0, 0)?;
    let pipelineCreateInfo: GraphicsPipelineBuilder = device
        .create_graphics_pipeline()
        .with_vertex_shader(&vertexShader)
        .with_fragment_shader(&fragShader)
        .with_target_info(
            GraphicsPipelineTargetInfo::new()
                .with_color_target_descriptions(ColorTargetDescriptions),
        )
        .with_vertex_input_state(
            VertexInputState::new()
                .with_vertex_attributes(&[VertexAttribute::new()
                    .with_format(VertexElementFormat::Float3)
                    .with_buffer_slot(0)
                    .with_location(0)
                    .with_offset(0)])
                .with_vertex_buffer_descriptions(&[VertexBufferDescription::new()
                    .with_slot(0)
                    .with_instance_step_rate(0)
                    // size of 1 vertex i think
                    .with_pitch(size_of::<f32>() as u32 * 3)
                    .with_input_rate(VertexInputRate::Vertex)]),
        )
        .with_primitive_type(sdl3::gpu::PrimitiveType::TriangleList)
        .with_fill_mode(FillMode::Fill)
        .with_rasterizer_state(
            RasterizerState::new()
                .with_fill_mode(FillMode::Fill)
                .with_cull_mode(CullMode::None),
        );
    let pipeline: GraphicsPipeline = pipelineCreateInfo.build()?;
    drop(vertexShader);
    drop(fragShader);

    // copying vertex data to gpu
    let mut commandBuffer = device.acquire_command_buffer().unwrap();
    let copyPass = device.begin_copy_pass(&commandBuffer)?;
    //#[rustfmt::skip]
    //let vertexData: &[f32] = &[
    //     0.5, 0.5, 0.0,
    //     0.5, -0.5, 0.0,
    //    -0.5, -0.5, 0.0,
    //    -0.5, 0.5, 0.0
    //];
    #[rustfmt::skip]
    let vertexData: &[f32] = &[
         1.0,  1.0, 0.0,
         1.0, -1.0, 0.0,
        -1.0, -1.0, 0.0,
        -1.0,  1.0, 0.0
    ];

    #[rustfmt::skip]
    let vertexIndicies: &[u16] = &[
        //0, 1, 2
        // must have 3, cant have 1
        //2, 3, 0
        //0, 2, 1
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

    //whiteboard.canvas.set_draw_color(Color::RGB(0, 0, 0));
    //whiteboard.canvas.clear();
    //whiteboard.canvas.present();

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
                Event::Window {
                    win_event: WindowEvent::Resized(x, y),
                    ..
                } => {}
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

        data[0] = whiteboard.canvasBounds.width().divide(&window.size().0);
        data[5] = whiteboard.canvasBounds.height().divide(&window.size().1);
        // canvasBounds.y1 gets larger as it goes down the screen
        data[3] = 2.0 * whiteboard.canvasBounds.x1.divide(&window.size().0);
        // negative 2 because up is negative / positive in different coordinate systems lmao
        data[7] = -2.0 * whiteboard.canvasBounds.y1.divide(&window.size().1);

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
        let textureCreateInfo = TextureCreateInfo::new()
            // the texture should be the same width and height as the render target
            .with_width(window.size().0)
            .with_height(window.size().1)
            .with_layer_count_or_depth(1)
            .with_num_levels(1)
            .with_usage(TextureUsage::DEPTH_STENCIL_TARGET)
            // 24 bits for depth, 8 bits for stencil
            .with_format(TextureFormat::D24UnormS8Uint);
        let mut depthStencilTexture: sdl3::gpu::Texture =
            device.create_texture(textureCreateInfo).unwrap();
        let depthStencilTarget = DepthStencilTargetInfo::new()
            .with_texture(&mut depthStencilTexture)
            //.with_cycle(true)
            //.with_stencil_load_op(LoadOp::CLEAR)
            //.with_stencil_store_op(StoreOp::STORE)
            //.with_clear_depth(-1.0)
            //.with_clear_stencil(1)
            ;
        //renderPass.set_scissor(Rect::new(320, 240, 320, 240));
        //device.set_viewport(
        //    &renderPass,
        //    Viewport::new(
        //        0.0,
        //        0.0,
        //        window.size().0 as f32,
        //        window.size().1 as f32,
        //        0.1,
        //        1.0,
        //    ),
        //);
        //println!("im drawing those primitives");
        //let myBuffer = [[

        //], []]

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
        commandBuffer.push_vertex_uniform_data(0, &data);
        // idk if i need to push this every frame
        // perhaps i can make a condition to only push when this is update
        // first number should be the same as the binding i think
        commandBuffer.push_fragment_uniform_data(0, &myInto::into(&whiteboard.bgcolor));
        //let fragData: f32 = 0.2;
        //commandBuffer.push_fragment_uniform_data(0, &fragData);
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
        // call at the end of every loop
        let targetFPS = 144;
        sleep(Duration::new(0, 1_000_000_000u32 / targetFPS));
        //sdl3::timer::delay(16);
    }
    return Ok(());
}
