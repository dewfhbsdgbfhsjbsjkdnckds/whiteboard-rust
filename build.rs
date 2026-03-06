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

use std::{
    fs::File, io::Write, os::unix::process::CommandExt, process::Command, thread::sleep,
    time::Duration,
};

use shaderc::ShaderKind;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shadersPath = "src/shaders/".to_string();
    let vertexShaderFilename = "vertex.glsl";
    let fragShaderFilename = "frag.glsl";

    let vertexShaderCode: &'static str = include_str!("src/shaders/vertex.glsl");
    let fragShaderCode: &'static str = include_str!("src/shaders/frag.glsl");

    let compiler: shaderc::Compiler = shaderc::Compiler::new()?;
    let vertexArtifact = compiler.compile_into_spirv(
        vertexShaderCode,
        ShaderKind::Vertex,
        "vertex.glsl",
        "main",
        None,
    )?;
    let fragArtifact = compiler.compile_into_spirv(
        fragShaderCode,
        ShaderKind::Fragment,
        "frag.glsl",
        "main",
        None,
    )?;
    let vertexBinary = vertexArtifact.as_binary_u8();
    let fragBinary = fragArtifact.as_binary_u8();
    let mut vertexFile = File::create(shadersPath.clone().to_owned() + "vertex.spv")?;
    let mut fragFile = File::create(shadersPath.clone().to_owned() + "frag.spv")?;
    vertexFile.write_all(&vertexBinary);
    fragFile.write_all(&fragBinary);
    Ok(())
}
