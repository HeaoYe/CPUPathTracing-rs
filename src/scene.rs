use crate::{
    accelerate::{Bounds, Bvh},
    geometry::{Bounded, Centroid, Intersection, Ray, Shape},
    light::{AreaLight, InfiniteLight, Light, UniformInfiniteLight},
    material::Material,
};

pub struct InstanceTransform {
    pub translation: glam::Vec3,
    pub scale: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Default for InstanceTransform {
    fn default() -> Self {
        Self {
            translation: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::IDENTITY,
        }
    }
}

impl InstanceTransform {
    fn into_affine(self) -> glam::Affine3A {
        glam::Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LightId(pub(crate) usize);

#[derive(Clone, Copy)]
pub struct ShapeInstanceId(usize);

pub(crate) struct ShapeInstance<'a> {
    bounds: Bounds,
    shape: &'a dyn Shape,
    material: Material,
    area_light_id: Option<LightId>,
    world_from_object: glam::Affine3A,
    object_from_world: glam::Affine3A,
}

impl<'a> ShapeInstance<'a> {
    pub fn new<T>(
        shape: &'a T,
        material: Material,
        area_light_id: Option<LightId>,
        world_from_object: glam::Affine3A,
    ) -> Self
    where
        T: Shape + Bounded,
    {
        let mut bounds = Bounds::default();
        let bounds_object = shape.bounds();
        for idx in 0..8 {
            let corner_objecr = bounds_object.corner(idx);
            let corner_world = world_from_object.transform_point3(corner_objecr);
            bounds.extend_point(corner_world);
        }

        Self {
            bounds,
            shape,
            material,
            area_light_id,
            world_from_object,
            object_from_world: world_from_object.inverse(),
        }
    }

    pub(crate) fn shape(&self) -> &dyn Shape {
        self.shape
    }

    pub(crate) fn world_from_object(&self) -> glam::Affine3A {
        self.world_from_object
    }

    pub(crate) fn object_from_world(&self) -> glam::Affine3A {
        self.object_from_world
    }
}

impl Bounded for ShapeInstance<'_> {
    fn bounds(&self) -> Bounds {
        self.bounds
    }
}

impl Centroid for ShapeInstance<'_> {
    fn centroid(&self) -> glam::Vec3 {
        (self.bounds.b_min() + self.bounds.b_max()) * 0.5
    }
}

#[derive(Default)]
pub struct SceneBuilder<'a> {
    instances: Vec<ShapeInstance<'a>>,
    lights: Vec<Light>,
}

pub struct Scene<'a> {
    bvh: Bvh<ShapeInstance<'a>>,
    lights: Vec<Light>,
    radius: f32,
}

impl<'a> SceneBuilder<'a> {
    pub fn add_shape<T>(&mut self, shape: &'a T, material: Material, transform: InstanceTransform)
    where
        T: Shape + Bounded,
    {
        let world_from_object = transform.into_affine();
        self.instances
            .push(ShapeInstance::new(shape, material, None, world_from_object));
    }

    pub fn add_area_light<T>(
        &mut self,
        shape: &'a T,
        material: Material,
        transform: InstanceTransform,
        radiance: impl Into<glam::Vec3>,
        double_side: bool,
    ) where
        T: Shape + Bounded,
    {
        assert_eq!(transform.scale, glam::Vec3::ONE);

        let world_from_object = transform.into_affine();
        self.instances.push(ShapeInstance::new(
            shape,
            material,
            Some(LightId(self.lights.len())),
            world_from_object,
        ));
        self.lights.push(Light::Area(AreaLight::new(
            ShapeInstanceId(usize::MAX),
            radiance.into(),
            double_side,
        )));
    }

    pub fn add_uniform_infinite_light(&mut self, radiance: impl Into<glam::Vec3>) {
        self.lights.push(Light::Infinite(InfiniteLight::Uniform(
            UniformInfiniteLight::new(radiance.into()),
        )));
    }

    pub fn build(mut self) -> Scene<'a> {
        let mut bvh = Bvh::new(self.instances);
        for (instance_index, instance) in bvh.ordered_primitives_mut().iter().enumerate() {
            if let Some(LightId(index)) = instance.area_light_id
                && let Light::Area(area_light) = self.lights.get_mut(index).unwrap()
            {
                area_light.shape_instance_id = ShapeInstanceId(instance_index)
            }
        }
        Scene {
            radius: bvh.bounds().diagonal().length() * 0.5,
            bvh,
            lights: self.lights,
        }
    }
}

pub struct HitInfo<'a> {
    pub intersection: Intersection,
    pub material: &'a Material,
    pub area_light: Option<&'a AreaLight>,
}

impl Scene<'_> {
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo<'_>> {
        self.bvh.intersect_with(
            ray,
            t_min,
            t_max,
            |instance, ray, t_min, t_max| {
                let ray_object = ray.transform(instance.object_from_world);
                let intersection = instance.shape.intersect(&ray_object, t_min, t_max);

                #[cfg(debug_assertions)]
                ray.debug_info
                    .borrow_mut()
                    .extend(ray_object.debug_info.into_inner());

                intersection.map(|intersection| (intersection.t, (instance, intersection)))
            },
            |(closest_instance, mut closest_intersection)| {
                closest_intersection.hit_point = closest_instance
                    .world_from_object
                    .transform_point3(closest_intersection.hit_point);
                closest_intersection.normal = closest_instance
                    .object_from_world
                    .matrix3
                    .mul_transpose_vec3(closest_intersection.normal)
                    .normalize();

                let area_light =
                    if let Some(LightId(area_light_index)) = closest_instance.area_light_id {
                        let Light::Area(area_light) = &self.lights[area_light_index] else {
                            unreachable!();
                        };
                        Some(area_light)
                    } else {
                        None
                    };

                HitInfo {
                    intersection: closest_intersection,
                    material: &closest_instance.material,
                    area_light,
                }
            },
        )
    }

    pub(crate) fn light_count(&self) -> usize {
        self.lights.len()
    }

    pub(crate) fn lights(&self) -> impl Iterator<Item = (LightId, &Light)> {
        self.lights
            .iter()
            .enumerate()
            .map(|(index, light)| (LightId(index), light))
    }

    pub(crate) fn infinite_lights(&self) -> impl Iterator<Item = (LightId, &InfiniteLight)> {
        self.lights
            .iter()
            .enumerate()
            .filter_map(|(index, light)| match light {
                Light::Infinite(light) => Some((LightId(index), light)),
                Light::Area(_) => None,
            })
    }

    pub(crate) fn infinite_radiance(&self, _light_direction: glam::Vec3) -> glam::Vec3 {
        self.lights
            .iter()
            .filter_map(|light| match light {
                Light::Infinite(light) => Some(light.radiance()),
                Light::Area(_) => None,
            })
            .sum()
    }

    pub(crate) fn radius(&self) -> f32 {
        self.radius
    }

    pub(crate) fn get_shape_instance(&self, id: ShapeInstanceId) -> Option<&ShapeInstance<'_>> {
        self.bvh.get_primitive(id.0)
    }

    pub(crate) fn get_light(&self, id: LightId) -> Option<&Light> {
        self.lights.get(id.0)
    }

    pub(crate) fn get_area_light_id(&self, id: ShapeInstanceId) -> Option<LightId> {
        self.get_shape_instance(id)?.area_light_id
    }
}
