mod bsp_reader;

extern crate byteorder;
#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate time;

use bsp_reader::BSPReader;
use glium::{DisplayBuild, Surface};
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use glium::glutin::*;
use cgmath::*;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
}

implement_vertex!(Vertex, position, normal, color);

fn main() {
    let mut bsp = BSPReader::new("data/maps/q3dm0.bsp");
    let header = bsp.read_header();

    let display = glium::glutin::WindowBuilder::new()
                      .with_title("Guac - Quake III in shit".to_owned())
                      .with_dimensions(1280, 720)
                      .with_depth_buffer(24)
                      .build_glium()
                      .unwrap();

    let window = display.get_window().unwrap();

    let models = bsp.read_models(&header.direntries);
    let faces = bsp.read_faces(&header.direntries);
    let vertexes = bsp.read_vertexes(&header.direntries);
    let meshverts = bsp.read_meshverts(&header.direntries);

    println!("{:#?}", models);
    let model = &models[0];

    let model_faces = &faces[model.face as usize..(model.face + model.n_faces) as usize];
    let mut vertex_buffer: Vec<Vertex> = Vec::new();
    for vertex in vertexes {
        vertex_buffer.push(Vertex {
            position: vertex.position,
            normal: vertex.normal,
            color: [vertex.color[0] as f32 / 256.0,
                    vertex.color[1] as f32 / 256.0,
                    vertex.color[2] as f32 / 256.0,
                    vertex.color[3] as f32 / 256.0],
        });
    }
    let vertex_buffer = glium::VertexBuffer::new(&display, &vertex_buffer).unwrap();
    let mut index_buffer: Vec<u32> = Vec::new();

    for face in model_faces {
        if face.f_type == 1 || face.f_type == 3 {
            for relative_vertex_index in
                &meshverts[face.meshvert as usize..(face.meshvert + face.n_meshverts) as usize] {
                index_buffer.push((relative_vertex_index + face.vertex) as u32);
            }
        }
    }

    let index_buffer = glium::index::IndexBuffer::new(&display,
                                                      glium::index::PrimitiveType::TrianglesList,
                                                      &index_buffer)
                           .unwrap();

    let vertex_shader_src = read_shader("src/shaders/world.vert");

    let fragment_shader_src = read_shader("src/shaders/world.frag");

    let program = glium::Program::from_source(&display,
                                              &vertex_shader_src,
                                              &fragment_shader_src,
                                              None)
                      .unwrap();

    let mut camera_pos = Point3::new(0.0, 0.0, 0.0);
    let mut pitch = deg(0.0);
    let mut yaw = deg(180.0);

    let mut cursor_caught = false;

    let mut cursor_dx = 0;
    let mut cursor_dy = 0;

    let mut pressed_keys = HashSet::new();

    let mut last_time = time::precise_time_ns();
    let mut last_fps_update = last_time;

    loop {
        let current_time = time::precise_time_ns();
        let dt = (current_time - last_time) as f32 / 1e9f32;
        last_time = current_time;

        if current_time - last_fps_update > 1e9 as u64 {
            println!("FPS: {}", 1.0 / dt);
            last_fps_update = current_time;
        }

        let camera_direction = Vector3::new(pitch.cos() * yaw.cos(),
                                            pitch.sin(),
                                            pitch.cos() * yaw.sin());

        let camera_sideways = camera_direction.cross(Vector3::new(0.0, 1.0, 0.0)).normalize();

        let model_m: [[f32; 4]; 4] = Matrix4::from(Matrix3::from_angle_x(Rad::from(deg(-90.0))))
                                         .into();
        let view_m: [[f32; 4]; 4] = Matrix4::look_at(camera_pos,
                                                     camera_pos + camera_direction,
                                                     Vector3::new(0.0, 1.0, 0.0))
                                        .into();
        let perspective_m: [[f32; 4]; 4] = perspective(deg(45.0), 1280.0 / 720.0, 1.0, 10000.0)
                                               .into();

        let mut target = display.draw();
        target.clear_color_and_depth((0.8, 0.8, 1.0, 1.0), 1.0);

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        target.draw(&vertex_buffer,
                    &index_buffer,
                    &program,
                    &uniform!{model: model_m, view: view_m, perspective: perspective_m},
                    &params)
              .unwrap();

        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape)) => {
                    window.set_cursor_state(CursorState::Normal);
                    cursor_caught = false;
                }
                Event::KeyboardInput(state, _, Some(key)) => {
                    if state == ElementState::Pressed {
                        pressed_keys.insert(key);
                    } else {
                        pressed_keys.remove(&key);
                    }
                }
                Event::MouseInput(_, _) => {
                    window.set_cursor_state(CursorState::Grab);
                    cursor_caught = true;
                }
                Event::MouseMoved((x, y)) => {
                    if cursor_caught {
                        cursor_dx = x - 1280 / 2;
                        cursor_dy = y - 720 / 2;

                        window.set_cursor_position(1280 / 2, 720 / 2);
                    }
                }
                Event::Closed => return,
                _ => (),
            }
        }

        yaw = yaw + deg(cursor_dx as f32 * 0.6);
        pitch = pitch - deg(cursor_dy as f32 * 0.6);

        if pitch > deg(89.0) {
            pitch = deg(89.0);
        }

        if pitch < deg(-89.0) {
            pitch = deg(-89.0);
        }

        if pressed_keys.contains(&VirtualKeyCode::W) {
            camera_pos = camera_pos + camera_direction * 500.0 * dt;
        }

        if pressed_keys.contains(&VirtualKeyCode::S) {
            camera_pos = camera_pos + camera_direction * (-500.0) * dt;
        }

        if pressed_keys.contains(&VirtualKeyCode::D) {
            camera_pos = camera_pos + camera_sideways * 500.0 * dt;
        }

        if pressed_keys.contains(&VirtualKeyCode::A) {
            camera_pos = camera_pos + camera_sideways * (-500.0) * dt;
        }

        cursor_dx = 0;
        cursor_dy = 0;
    }
}

fn read_shader(path: &str) -> String {
    let mut string = String::new();
    File::open(path).unwrap().read_to_string(&mut string);
    string
}
