pub mod s_expressions;

use std::any::Any;

use dyn_clone::DynClone;

use super::renderer::Renderer;
use super::Stub;

pub type Preprocessor = fn(&mut Stub) -> ();

pub trait Renderable<'a>: std::fmt::Debug + DynClone {
    fn render(&self, renderer: &Renderer) -> String;

    fn as_any(&self) -> &(dyn Any + 'a);
}

dyn_clone::clone_trait_object!(Renderable<'_>);
