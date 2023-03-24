mod utils;

use std::collections::HashMap;

use three_d::*;
use wasm_bindgen::prelude::*;
use winit::{event_loop::EventLoop, window::WindowId};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// pub struct RenderContext {
//     window: Box<Window<()>>,
// }

// #[wasm_bindgen]
// impl RenderContext {
//     #[wasm_bindgen(constructor)]
//     pub fn new(width: u32, height: u32) -> RenderContext {
//         RenderContext { window: Box::new(Window::new(WindowSettings {
//             title: "Render Window".to_string(),
//             max_size: Some((width, height)),
//             ..Default::default()
//         }).unwrap()) }
//     }

//     pub fn get(&self) -> Box<Window<()>> {
//         self.window
//     }

//     pub fn set(&mut self, window: Box<Window<()>>) {
//         self.window = window;
//     }
// }

#[wasm_bindgen]
pub struct Rendering {
    event_loop: EventLoop<()>,
    windows: HashMap<
        WindowId,
        Box<
            dyn FnMut(
                &winit::event::Event<()>,
                &winit::event_loop::EventLoopWindowTarget<()>,
                &mut winit::event_loop::ControlFlow,
            ),
        >,
    >,
}

#[wasm_bindgen]
impl Rendering {
    #[wasm_bindgen]
    pub fn new() -> Self {
        utils::set_panic_hook();

        event_loop.spawn(move |event, target, control_flow| {
            event_loop_1(&event, target, control_flow);
            event_loop_2(&event, target, control_flow);
        });

        Self {
            event_loop: EventLoop::new(),
            windows: HashMap::new(),
        }
    }

    fn create_window(&mut self, event_loop: &EventLoop<()>, canvas_id: &str) -> WindowId {
        let websys_window = web_sys::window()
            .ok_or(WindowError::WindowCreation)
            .unwrap();
        let document = websys_window
            .document()
            .ok_or(WindowError::DocumentMissing)
            .unwrap();
        let canvas_element = document
            .get_element_by_id(canvas_id)
            .expect("settings doesn't contain canvas and DOM doesn't have a canvas element either")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|e| WindowError::CanvasConvertFailed(format!("{:?}", e)))
            .unwrap();

        println!("canvas {}", canvas_element.id());

        let window = Window::from_event_loop(
            WindowSettings {
                title: "Instanced Shapes!".to_string(),
                max_size: Some((1280, 720)),
                canvas: Some(canvas_element),
                ..Default::default()
            },
            &event_loop,
        )
        .unwrap();
        let context = window.gl();

        let mut camera = Camera::new_perspective(
            window.viewport(),
            vec3(60.00, 50.0, 60.0), // camera position
            vec3(0.0, 0.0, 0.0),     // camera target
            vec3(0.0, 1.0, 0.0),     // camera up
            degrees(45.0),
            0.1,
            1000.0,
        );
        let mut control = OrbitControl::new(vec3(0.0, 0.0, 0.0), 1.0, 1000.0);

        let light0 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, -0.5, -0.5));
        let light1 = DirectionalLight::new(&context, 1.0, Color::WHITE, &vec3(0.0, 0.5, 0.5));

        // Container for non instanced meshes.
        let mut non_instanced_meshes = Vec::new();

        // Instanced mesh object, initialise with empty instances.
        let mut instanced_mesh = Gm::new(
            InstancedMesh::new(&context, &Instances::default(), &CpuMesh::cube()),
            PhysicalMaterial::new(
                &context,
                &CpuMaterial {
                    albedo: Color {
                        r: 128,
                        g: 128,
                        b: 128,
                        a: 255,
                    },
                    ..Default::default()
                },
            ),
        );
        instanced_mesh.set_animation(|time| Mat4::from_angle_x(Rad(time)));

        // Initial properties of the example, 2 cubes per side and non instanced.
        let side_count = 4;
        let is_instanced = true;

        let event_handler = window.get_render_loop_impl::<(), _>(move |mut frame_input| {
            let viewport = Viewport {
                x: 0,
                y: 0,
                width: frame_input.viewport.width,
                height: frame_input.viewport.height,
            };
            camera.set_viewport(viewport);

            // Camera control must be after the gui update.
            control.handle_events(&mut camera, &mut frame_input.events);

            // Ensure we have the correct number of cubes, does no work if already correctly sized.
            let count = side_count * side_count * side_count;
            if non_instanced_meshes.len() != count {
                non_instanced_meshes.clear();
                for i in 0..count {
                    let mut gm = Gm::new(
                        Mesh::new(&context, &CpuMesh::cube()),
                        PhysicalMaterial::new(
                            &context,
                            &CpuMaterial {
                                albedo: Color {
                                    r: 128,
                                    g: 128,
                                    b: 128,
                                    a: 255,
                                },
                                ..Default::default()
                            },
                        ),
                    );
                    let x = (i % side_count) as f32;
                    let y = ((i as f32 / side_count as f32).floor() as usize % side_count) as f32;
                    let z = (i as f32 / side_count.pow(2) as f32).floor();
                    gm.set_transformation(Mat4::from_translation(
                        3.0 * vec3(x, y, z) - 1.5 * (side_count as f32) * vec3(1.0, 1.0, 1.0),
                    ));
                    gm.set_animation(|time| Mat4::from_angle_x(Rad(time)));
                    non_instanced_meshes.push(gm);
                }
            }

            if instanced_mesh.instance_count() != count as u32 {
                instanced_mesh.set_instances(&Instances {
                    transformations: (0..count)
                        .map(|i| {
                            let x = (i % side_count) as f32;
                            let y = ((i as f32 / side_count as f32).floor() as usize % side_count)
                                as f32;
                            let z = (i as f32 / side_count.pow(2) as f32).floor();
                            Mat4::from_translation(
                                3.0 * vec3(x, y, z)
                                    - 1.5 * (side_count as f32) * vec3(1.0, 1.0, 1.0),
                            )
                        })
                        .collect(),
                    ..Default::default()
                });
            }

            // Always update the transforms for both the normal cubes as well as the instanced versions.
            // This shows that the difference in frame rate is not because of updating the transforms
            // and shows that the performance difference is not related to how we update the cubes.
            let time = (frame_input.accumulated_time * 0.001) as f32;
            instanced_mesh.animate(time);
            non_instanced_meshes
                .iter_mut()
                .for_each(|m| m.animate(time));

            // Then, based on whether or not we render the instanced cubes, collect the renderable
            // objects.
            let render_objects: Vec<&dyn Object> = if is_instanced {
                instanced_mesh.into_iter().collect()
            } else {
                non_instanced_meshes
                    .iter()
                    .map(|x| x as &dyn Object)
                    .collect()
            };

            frame_input
                .screen()
                .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                .render(&camera, render_objects, &[&light0, &light1]);

            FrameOutput::default()
        });

        self.windows
            .insert(window.window.id(), Box::new(event_handler));

        window.window.id()
    }
}
