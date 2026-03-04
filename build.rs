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

use std::{os::unix::process::CommandExt, process::Command, thread::sleep, time::Duration};

fn main() {
    // glslc -fshader-stage=vertex -o vertex.spv vertex.glsl
    // glslc -fshader-stage=fragment -o frag.spv frag.glsl
    let shadersPath = "src/shaders/".to_string();
    // this code is quite repetitive, i could change it but it works fine for now i think
    let _ = Command::new("glslc")
        .args([
            "-fshader-stage=vertex",
            "-o",
            &(shadersPath.clone().to_owned() + "vertex.spv"),
            &(shadersPath.clone().to_owned() + "vertex.glsl"),
        ])
        .output()
        .expect("i failed");
    let _ = Command::new("glslc")
        .args([
            "-fshader-stage=fragment",
            "-o",
            &(shadersPath.clone().to_owned() + "frag.spv"),
            &(shadersPath.clone().to_owned() + "frag.glsl"),
        ])
        .exec();
    sleep(Duration::from_secs(10));
    println!("is this even working???");
}
