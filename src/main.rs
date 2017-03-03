#[macro_use]
extern crate glium;

mod point;

use glium::*;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use point::Point;
use std::f32;

fn load_file(file_name: &str) -> io::Result<Vec<Point>> {
    let mut f = File::open(file_name)?;
    let mut bytes = vec![0; f.metadata()?.len() as usize];
    f.read_exact(&mut bytes).unwrap();

    println!("Read {} bytes", bytes.len());

    // pack bytes into int for "easy" sorting
    let mut packed_bytes = Vec::with_capacity(bytes.len() / 3);
    for i in 0..(bytes.len() / 3) - 1 {
        let mut buf: i32 = bytes[i * 3] as i32;
        buf = (buf << 8) + bytes[i * 3 + 1] as i32;
        buf = (buf << 8) + bytes[i * 3 + 2] as i32;

        packed_bytes.push(buf);
    }

    // sorting will put all duplicate points next to each other
    packed_bytes.sort();

    let mut points: Vec<Point> = vec![];

    let mut prev = -1;
    for packed in packed_bytes {
        if packed == prev {
            let i = points.len() - 1;
            points[i].count += 1;
            continue;
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
            color: [0.0, 0.0, 0.0],
            count: 0
        });

        prev = packed;
    }

    let len = points.len();
    for i in 0..len {
        let ref mut p = points[i];

        let count = p.count as f32 / 10.0;
        let color = i as f32 / len as f32;

        // same strange coloring scheme
        p.color = [color, 1.0 - count, 1.0 - count * color];
    }

    Ok(points)
}

fn main() {
    let name = match std::env::args().nth(1) {
        Some(name) => name,
        None => {
            println!("Usage: [program] [file]");
            return;
        }
    };

    let points = match load_file(&name) {
        Ok(p) => p,
        Err(e) => {
            println!("Error loading file: {}", e);
            return;
        }
    };

    println!("{} unique points", points.len());

    let display = glutin::WindowBuilder::new().build_glium().unwrap();

    let v_buffer = VertexBuffer::new(&display, &points).unwrap();
    let indices = index::NoIndices(index::PrimitiveType::Points);

    let frag_shader = include_str!("../res/frag.glsl");
    let vert_shader = include_str!("../res/vert.glsl");
    
    let prog = Program::from_source(&display, vert_shader, frag_shader, None).unwrap();

    let mut zoom: f32 = 0.8;
    loop {
        let mut target = display.draw();
        let (width, height) = target.get_dimensions();

        let p = create_perspective(width, height);

        let m = [
            [zoom, 0.0, 0.0, 0.0],
            [0.0, zoom, 0.0, 0.0],
            [0.0, 0.0, zoom, 0.0],
            [0.0, 0.0, 2.0, 1.0f32],
        ];

        target.clear_color(0.0, 0.0, 0.0, 1.0);

        target.draw(&v_buffer, &indices, &prog, &uniform! { P: p, M: m }, &Default::default()).unwrap();
        
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}

fn create_perspective(width: u32, height: u32) -> [[f32; 4]; 4] {
    let aspect_ratio = height as f32 / width as f32;

    let fov: f32 = 3.141592 / 3.0;
    let znear = 0.1;
    let zfar = 1024.0;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [  f  *  aspect_ratio ,    0.0,              0.0              ,   0.0],
        [         0.0         ,     f ,              0.0              ,   0.0],
        [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
        [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
    ]
}