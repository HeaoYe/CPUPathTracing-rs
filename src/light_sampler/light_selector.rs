use super::MisCompensation;
use crate::{
    scene::{LightId, Scene},
    util::Rng,
};

pub struct LightSelection {
    pub id: LightId,
    pub pmf: f32,
}

pub trait LightSelector {
    fn new(scene: &Scene, mis_compensation: MisCompensation) -> Self;

    fn sample_light_source(&self, rng: &mut Rng) -> Option<LightSelection>;

    fn pmf(&self, light_id: LightId) -> f32;
}
