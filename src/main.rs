#[macro_use]
extern crate glium;
extern crate time;
extern crate cgmath;

mod point;

use glium::*;
use cgmath::prelude::*;
use time::precise_time_s;

use point::Point;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::f32;

fn load_file(file_name: &str) -> io::Result<Vec<Point>> {
    let mut f = File::open(file_name)?;
    let mut bytes = vec![0; f.metadata()?.len() as usize];
    f.read_exact(&mut bytes).unwrap();

    println!("Read {} bytes", bytes.len());

    // pack bytes into int for "easy" sorting
    let mut packed_bytes = Vec::with_capacity(bytes.len() / 3);
    for i in 0..(bytes.len() / 3) - 1 {
        let mut buf: i32 = bytes[i * 3 + 2] as i32;
        buf = (buf << 8) + bytes[i * 3 + 1] as i32;
        buf = (buf << 8) + bytes[i * 3 + 0] as i32;

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

    let mut zoom_target: f32 = 0.8;
    let mut zoom: f32 = 0.8;

    let mut shift_down = false;
    let mut mouse_down = false;
    let mut last_mouse_pos = (0i32, 0i32);

    let mut rot_x: f32 = 0.0;
    let mut rot_y: f32 = 0.0;
    let mut rot_z: f32 = 0.0;

    let mut target_rot_x: f32 = 0.0;
    let mut target_rot_z: f32 = 0.0;

    let mut last_time = precise_time_s();

    loop {
        let now = precise_time_s();
        let delta = now - last_time;

        zoom += (zoom_target - zoom) * delta as f32;
        rot_x += (target_rot_x - rot_x) * delta as f32;
        rot_z += (target_rot_z - rot_z) * delta as f32;

        last_time = now;

        let mut target = display.draw();
        let (width, height) = target.get_dimensions();
        let aspect = width as f32 / height as f32;

        let p: [[f32; 4]; 4] = cgmath::ortho(-aspect, aspect, -1.0, 1.0, -1024.0, 1024.0).into();

        let scale = cgmath::Matrix4::new(
            zoom, 0.0, 0.0, 0.0,
            0.0, zoom, 0.0, 0.0,
            0.0, 0.0, zoom, 0.0,
            0.00, 0.0, 0.0, 1.0f32,
        ); 

        let m_rot_x = cgmath::Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, rot_x.cos(), -rot_x.sin(), 0.0,
            0.0, rot_x.sin(), rot_x.cos(), 0.0, 
            0.0, 0.0, 0.0, 1.0, 
        );

        // let m_rot_y = cgmath::Matrix4::new(
        //     rot_y.cos(), 0.0, rot_y.sin(), 0.0,
        //     0.0, 1.0, 0.0, 0.0,
        //     -rot_y.sin(), 0.0, rot_y.cos(), 0.0, 
        //     0.0, 0.0, 0.0, 1.0, 
        // );

        let m_rot_z = cgmath::Matrix4::new(
            rot_z.cos(), -rot_z.sin(), 0.0, 0.0,
            rot_z.sin(), rot_z.cos(), 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0, 
            0.0, 0.0, 0.0, 1.0, 
        );

        let m: [[f32; 4]; 4] = (scale * m_rot_z * m_rot_x).into();

        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&v_buffer, &indices, &prog, &uniform! { P: p, M: m }, &Default::default()).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            use glutin::{VirtualKeyCode, ElementState, Event, MouseButton};

            match ev {
                Event::Closed => return,
                Event::KeyboardInput(state, _, vk) => {
                    match vk {
                        Some(VirtualKeyCode::LShift) => {
                            shift_down = state == ElementState::Pressed;
                        },
                        Some(VirtualKeyCode::Z) => {
                            zoom_target += if shift_down { -0.5 } else { 0.5 };
                        },
                        Some(VirtualKeyCode::Escape) => {
                            return;
                        },
                        Some(VirtualKeyCode::V) => {
                            rot_x += 0.01;
                            rot_z += 0.01;
                        },

                        _ => {}
                    }
                },

                Event::MouseInput(state, button) => {
                    if button == MouseButton::Left {
                        mouse_down = state == ElementState::Pressed;
                    }
                },

                Event::MouseMoved(x, y) => {
                    if mouse_down {
                        target_rot_z -= (x - last_mouse_pos.0) as f32 / width as f32;
                        target_rot_x -= (y - last_mouse_pos.1) as f32 / height as f32;
                    }

                    last_mouse_pos = (x, y);
                },

                _ => ()
            }
        }
    }
}
