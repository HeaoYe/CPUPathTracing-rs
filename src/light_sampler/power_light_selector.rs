use super::{LightSelection, LightSelector};
use crate::{
    light::Light,
    sample::{AliasTable, AliasTableSample},
    scene::{LightId, Scene},
    util::Rng,
};

pub struct PowerLightSelector {
    alias_table: AliasTable,
}

impl LightSelector for PowerLightSelector {
    fn new(scene: &Scene) -> Self {
        let mut powers = Vec::with_capacity(scene.lights().len());
        for light in scene.lights() {
            powers.push(match light {
                Light::Area(light) => {
                    let shape_instance = scene.get_shape_instance(light.shape_instance_id).unwrap();
                    light.power(shape_instance.shape())
                }
                Light::UniformInfinite(light) => light.power(scene.radius()),
            });
        }
        Self {
            alias_table: AliasTable::new(&powers),
        }
    }

    fn sample_light_source(&self, rng: &mut Rng) -> Option<super::light_selector::LightSelection> {
        let AliasTableSample { index, pmf } = self.alias_table.sample(rng.uniform())?;
        Some(LightSelection {
            id: LightId(index),
            pmf,
        })
    }

    fn pmf(&self, light_source: LightId) -> f32 {
        self.alias_table.pmf(light_source.0)
    }
}
