extern crate console_error_panic_hook;
extern crate wasm_bindgen;
extern crate web_sys;
use renders::{ColorSpace, Cursor, AxisInput};
use winit::window::WindowBuilder;
mod geometry;
use geometry::{cylinder_mesh, quad_mesh, tube_mesh};
mod renders;

use three_d::{
    renderer::{control::Event, render_states::*, *},
    FrameOutput, SurfaceSettings, Window,
};
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

use crate::renders::{InputState, Renderable, Renderer};
// use

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// struct Uniform<T: UniformDataType> {
//     name: String,
//     value: T,
// }

struct Model {
    positions: VertexBuffer,
    embed: Option<VertexBuffer>,
    transform: Mat4,
}

// trait Renderable {
//     fn positions() -> Vec<Vec3>;
// }

fn to_cylindrical(v: Vec3) -> Vec3 {
    let angle = v.z.atan2(v.x);
    let radius = vec2(v.x, v.z).magnitude();
    let y = v.y;
    vec3(angle, radius, y)
}

fn from_cylindrical(v: Vec3) -> Vec3 {
    let x = v.x.cos() * v.y;
    let z = v.x.sin() * v.y;
    let y = v.z;
    vec3(x, y, z)
}

#[wasm_bindgen]
pub struct ColorView {
    // winit_window: winit::window::Window,
    // context: WindowedContext,
    // event_loop: EventLoop<()>,
    window: Window,
    width: u32,
    height: u32,
    control: OrbitControl,
    selection: Vec3,
    hover: Vec3,
    chunk: Vec3,
    position: Vec2,
    // tags: Vec<Box<dyn FnMut(&mut ColorView, Vec3)->()>,
    // camera: Camera,
    state: InputState,
    color_scene: Program,
    pos_scene: Program,
    cursor: Cursor,
    cylinder: ColorSpace,
    axes: [AxisInput; 3],
    on_select: Option<Box<dyn FnMut(f32, f32, f32) -> ()>>,
    // on_hover: Option<Box<dyn FnMut(f32, f32, f32) -> ()>>,
}

fn color_shader(string: &str) -> String {
    include_str!("color.frag").replace("// REPLACE", string)
}

fn color_program(context: &Context, string: &str) -> Program {
    let src = color_shader(string);
    Program::from_source(&context, include_str!("color.vert"), &src).unwrap()
}

#[wasm_bindgen]
impl ColorView {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(width: u32, height: u32) -> Self {
        let window_builder = winit::window::WindowBuilder::new()
            .with_title("winit window")
            .with_min_inner_size(winit::dpi::LogicalSize::new(width, height))
            .with_maximized(true);
        ColorView::build(window_builder, width, height)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(canvas: HtmlCanvasElement, width: u32, height: u32) -> Self {
        let window_builder = match canvas.dyn_into::<HtmlCanvasElement>() {
            Ok(canvas) => {
                use winit::platform::web::WindowBuilderExtWebSys;
                winit::window::WindowBuilder::new()
                    .with_canvas(Some(canvas))
                    .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            }
            _ => panic!("ColorView::new must be passed a canvas!"),
        };
        ColorView::build(window_builder, width, height)
    }

    fn build(window_builder: WindowBuilder, width: u32, height: u32) -> ColorView {
        let event_loop = winit::event_loop::EventLoop::new();
        let winit_window = window_builder.build(&event_loop).unwrap();
        let window =
            Window::from_winit_window(winit_window, event_loop, SurfaceSettings::default(), false)
                .unwrap();
        let context = window.gl();
        // let context = WindowedContext::from_winit_window(&winit_window, SurfaceSettings::default()).unwrap();

        let camera = Camera::new_perspective(
            Viewport::new_at_origo(1, 1),
            vec3(0.0, 2.0, 4.0),
            vec3(0.0, 0.5, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            10.0,
        );
        let control = OrbitControl::new(*camera.target(), 1.0, 100.0);

        let color_scene = color_program(&context, "color = vec4(hsv2rgb(xyz2hsv(pos.xyz)), 1.0);");
        let pos_scene = color_program(&context, "color = vec4(pos.xyz, tag);");
        // let tags = vec![
        //     Box::new(|view: &mut Self, color: Vec3| {

        //     })
        // ];
        let view = ColorView {
            window,
            // winit_window,
            // context,
            // event_loop,
            width,
            height,
            control,
            selection: vec3(0.0, 1.0, 1.0),
            hover: vec3(0.0, 1.0, -1.0),
            chunk: vec3(0.0, 1.0, 1.0),
            position: vec2(0.0, 0.0),
            // tags,
            on_select: None,
            // on_hover: None,
            state: InputState::new(vec3(0.0, 1.0, 1.0), camera),
            color_scene,
            pos_scene,
            cursor: Cursor::cube(&context),
            cylinder: ColorSpace::cylinder(&context),
            axes: [
                AxisInput::new(&context, 0),
                AxisInput::new(&context, 1),
                AxisInput::new(&context, 2),
            ]
        };
        view
    }

    fn initialize_models(context: &Context) -> (Model, Model, Model, Model) {
        let cube = VertexBuffer::new_with_data(&context, &CpuMesh::cube().positions.to_f32());
        let cylinder = VertexBuffer::new_with_data(&context, &cylinder_mesh(64));
        let quad = VertexBuffer::new_with_data(&context, &quad_mesh());
        let tube = tube_mesh(64);
        let tube_wrap: Vec<Vec3> = tube
            .iter()
            .map(|pos| {
                let flat = vec2(pos.x, pos.z);
                let mut angle = -flat.y.atan2(flat.x) / std::f32::consts::PI / 2.0;
                if angle < 0.0 {
                    angle += 1.0;
                }
                // let radius = flat.magnitude2().sqrt();
                vec3(angle, pos.y, 0.0)
            })
            .collect();
        for i in 0..18 {
            log(&format!("{:?}", tube_wrap[i]));
        }
        log("STEP");
        for i in (0..(64 * 6)).step_by(8) {
            log(&format!("{:?}", tube_wrap[i]));
        }
        let tube = VertexBuffer::new_with_data(&context, &tube);
        let tube_wrap = VertexBuffer::new_with_data(&context, &tube_wrap);
        (
            Model {
                positions: cube,
                embed: None,
                transform: Mat4::from_scale(0.5) * Mat4::from_translation(Vec3::new(1.0, 1.0, 1.0)),
            },
            Model {
                positions: cylinder,
                embed: None,
                transform: Mat4::from_translation(vec3(0.0, 0.0, 0.0)),
            },
            Model {
                positions: quad,
                embed: None,
                transform: Mat4::identity(),
            },
            Model {
                positions: tube_wrap,
                embed: Some(tube),
                transform: Mat4::identity(),
            },
        )
    }

    pub fn render_loop(mut self) {
        let context = self.window.gl();

        self.window.render_loop(move |mut input| {
            let mut press = false;
            for event in input.events.iter() {
                match event {
                    Event::MouseMotion { position: pos, .. } => {
                        self.position = Vec2::new(pos.x, pos.y);
                    }
                    Event::MousePress {
                        button,
                        position: pos,
                        ..
                    } => {
                        self.position = Vec2::new(pos.x, pos.y);
                        press = *button == MouseButton::Left;
                    }
                    _ => {}
                }
            }
            self.control
                .handle_events(&mut self.state.camera, &mut input.events);
            let screen = input.screen();
            let state = &self.state;
            screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 0.0, 1.0));
            let cylinder = &self.cylinder.model(state);
            self.color_scene.render(&screen, cylinder);
            self.color_scene
                .render(&screen, &self.cursor.model(state));
            for i in 0..3 {
                self.color_scene
                    .render(&screen, &self.axes[i].model(state));
            }

            let mut texture = Texture2D::new_empty::<[f32; 4]>(
                &context,
                self.width,
                self.height,
                Interpolation::Nearest,
                Interpolation::Nearest,
                None,
                Wrapping::ClampToEdge,
                Wrapping::ClampToEdge,
            );
            let mut depth_texture = DepthTexture2D::new::<f32>(
                &context,
                self.width,
                self.height,
                Wrapping::ClampToEdge,
                Wrapping::ClampToEdge,
            );
            let pos_target = RenderTarget::new(
                texture.as_color_target(None),
                depth_texture.as_depth_target(),
            );
            pos_target.clear(ClearState::depth(1.0));
            self.pos_scene.render(&pos_target, cylinder);
            for i in 0..3 {
                self.pos_scene
                    .render(&pos_target, &self.axes[i].model(state));
            }
            let position = self.position;
            let scissor_box = ScissorBox {
                x: position.x as i32,
                y: (self.height as i32) - (position.y as i32),
                width: 1,
                height: 1,
            };
            let pos = pos_target.read_color_partially::<[f32; 4]>(scissor_box)[0];
            let tag = pos[3] as u8;
            let pos = to_cylindrical(vec3(pos[0], pos[1], pos[2]));
            let pos = match tag {
                1 => vec3(pos.x, self.state.cylindrical.y, self.state.cylindrical.z),
                2 => vec3(self.state.cylindrical.x, pos.y, self.state.cylindrical.z),
                3 => vec3(self.state.cylindrical.x, self.state.cylindrical.y, pos.z),
                7 => pos,
                _ => self.state.saved_cylindrical,
            };
            if press {
                self.state.saved_cylindrical = pos;
            }
            self.state.cylindrical = pos;
            self.state.pos = from_cylindrical(pos);
            FrameOutput::default()
        });
    }
}
