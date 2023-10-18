use palette::{okhsv, FromColor, Oklab};
use three_d::{vec3, Context, Mat4, Program, RenderStates, RenderTarget, SquareMatrix, Vec3};

use crate::{
    element::{ColorModel, SpaceModel},
    from_cylindrical,
    geometry::quad_mesh,
    input::InputState,
    mesh::Mesh,
};

pub trait Renderable<'a, T, M> {
    fn model(&'a self, state: &T) -> M;
}

pub trait Renderer<T> {
    fn render(&mut self, target: &RenderTarget, model: &T);
}

impl<'a> Renderer<ColorModel<'a>> for Program {
    fn render(&mut self, target: &RenderTarget, model: &ColorModel) {
        target.write(move || {
            self.use_uniform("view", model.view);
            self.use_uniform("model", model.model);
            self.use_uniform("meta", model.meta);
            self.use_vertex_attribute("position", model.positions);
            self.use_vertex_attribute("embed", model.embed);
            self.draw_elements(model.render_states, target.viewport(), model.indices);
        });
    }
}

impl<'a> Renderer<SpaceModel<'a>> for Program {
    fn render(&mut self, target: &RenderTarget, model: &SpaceModel) {
        target.write(move || {
            self.use_uniform("view", model.view);
            self.use_uniform("model", model.model);
            self.use_uniform("meta", model.meta);
            self.use_vertex_attribute("position", model.positions);
            self.use_vertex_attribute("embed", model.embed);
            self.draw_elements(model.render_states, target.viewport(), model.indices);
        });
    }
}

#[derive(PartialEq)]
pub enum Space {
    Linear,
    Cylindrical,
}

pub struct ColorChip {
    positions: Mesh,
}

impl ColorChip {
    pub fn new(context: &Context) -> Self {
        ColorChip {
            positions: Mesh::from_positions(context, quad_mesh()),
        }
    }
}

impl<'a> Renderable<'a, Vec3, ColorModel<'a>> for ColorChip {
    fn model(&'a self, pos: &Vec3) -> ColorModel<'a> {
        ColorModel {
            positions: &self.positions.vertex_buffer(),
            embed: &self.positions.vertex_buffer(),
            indices: &self.positions.element_buffer(),
            render_states: RenderStates::default(),
            view: Mat4::identity(),
            model: Mat4::from_translation(vec3(0.8, 0.8, 0.0)) * Mat4::from_scale(0.2),
            meta: Mat4::from_translation(*pos) * Mat4::from_scale(0.0),
        }
    }
}

pub fn okhsv_embed_oklab(pos: Vec3) -> Vec3 {
    let hsv = okhsv::Okhsv::new(pos.x * 360.0, pos.z, pos.y);
    let oklab = Oklab::from_color(hsv);
    vec3(oklab.l, oklab.a, oklab.b)
}

pub struct Cursor {
    positions: Mesh,
}

impl Cursor {
    pub fn cube(context: &Context) -> Self {
        let data = three_d::CpuMesh::cube().positions.to_f32();
        Cursor {
            positions: Mesh::from_positions(context, data),
        }
    }
}

impl<'a> Renderable<'a, InputState, ColorModel<'a>> for Cursor {
    fn model(&'a self, state: &InputState) -> ColorModel<'a> {
        ColorModel {
            positions: &self.positions.vertex_buffer(),
            embed: &self.positions.vertex_buffer(),
            indices: &self.positions.element_buffer(),
            render_states: RenderStates::default(),
            view: state.camera.projection() * state.camera.view(),
            model: Mat4::from_translation(from_cylindrical(state.pos)) * Mat4::from_scale(0.05),
            meta: Mat4::from_scale(0.0),
        }
    }
}
