#[macro_use]
extern crate glium;

mod point;

use glium::*;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use point::Point;
use std::f32;

fn load_file(file_name: &str) -> io::Result<(Vec<Point>, i32)> {
    let f = File::open(file_name)?;
    let bytes: Vec<u8> = f
        .bytes()
        .map(|r| r.unwrap())
        .collect();

    // pack bytes into int for "easy" sorting
    let mut packed_bytes = vec![];
    for i in 0..(bytes.len() / 3) - 1 {
        let mut buf: i32 = bytes[i * 3] as i32;
        buf = (buf << 8) + bytes[i * 3 + 1] as i32;
        buf = (buf << 8) + bytes[i * 3 + 2] as i32;

        packed_bytes.push(buf);
    }

    // sorting will put all duplicate points next to each other
    packed_bytes.sort();

    let mut points: Vec<Point> = vec![];

    let mut dup = 0;
    let mut prev = -1;
    for packed in packed_bytes {
        if packed == prev {
            dup += 1;

            let i = points.len() - 1;
            points[i].count += 1;
        }

        let x = ((packed >> 16) & 0xFF) as f32 / 255.0;
        let y = ((packed >> 8) & 0xFF) as f32 / 255.0;
        let z = (packed & 0xFF) as f32 / 255.0;

        let rot_x = x * (2.0 * f32::consts::PI);
        let rot_y = y * (2.0 * f32::consts::PI);
        let radius = z;

        let x = radius * rot_x.sin() * rot_y.cos();
        let y = radius * rot_x.sin() * rot_y.sin();
        let z = radius * rot_x.cos();

        points.push(Point {
            pos: [x, y, z],
            count: 0
        });

        prev = packed;
    }

    Ok((points, dup))
}

fn main() {
    let name = match std::env::args().nth(1) {
        Some(name) => name,
        None => {
            println!("Usage: [program] [file]");
            return;
        }
    };

    let (points, dup) = match load_file(&name) {
        Ok(tup) => tup,
        Err(e) => {
            println!("Error loading file: {}", e);
            return;
        }
    };

    let display = glutin::WindowBuilder::new().build_glium().unwrap();

    let v_buffer = VertexBuffer::new(&display, &points).unwrap();
    let indices = index::NoIndices(index::PrimitiveType::Points);

    let frag_shader = include_str!("../res/frag.glsl");
    let vert_shader = include_str!("../res/vert.glsl");
    
    let prog = Program::from_source(&display, vert_shader, frag_shader, None).unwrap();

    loop {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        target.draw(&v_buffer, &indices, &prog, &uniforms::EmptyUniforms, &Default::default()).unwrap();
        
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}
