use super::material::Material;
use super::ray::Ray;
use super::shape::{Intersection, Shape};

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

struct ShapeInstance<'a> {
    shape: &'a dyn Shape,
    material: Material,
    world_from_object: glam::Affine3A,
    object_from_world: glam::Affine3A,
}

#[derive(Default)]
pub struct Scene<'a> {
    instances: Vec<ShapeInstance<'a>>,
}

impl<'a> Scene<'a> {
    pub fn add_shape(
        &mut self,
        shape: &'a dyn Shape,
        material: Material,
        transform: InstanceTransform,
    ) {
        let world_from_object = transform.into_affine();
        self.instances.push(ShapeInstance {
            shape,
            material,
            world_from_object,
            object_from_world: world_from_object.inverse(),
        });
    }
}

pub struct HitInfo<'a> {
    pub intersection: Intersection,
    pub material: &'a Material,
}

impl Scene<'_> {
    pub fn intersect(&self, ray: &Ray, t_min: f32, mut t_max: f32) -> Option<HitInfo<'_>> {
        let mut closest = None;
        for instance in &self.instances {
            let ray_object = ray.transform(instance.object_from_world);
            if let Some(intersection) = instance.shape.intersect(&ray_object, t_min, t_max) {
                t_max = intersection.t;
                closest = Some((instance, intersection));
            }
        }

        let (closest_instance, mut closest_intersection) = closest?;

        closest_intersection.hit_point = closest_instance
            .world_from_object
            .transform_point3(closest_intersection.hit_point);
        closest_intersection.normal = closest_instance
            .object_from_world
            .matrix3
            .mul_transpose_vec3(closest_intersection.normal)
            .normalize();

        Some(HitInfo {
            intersection: closest_intersection,
            material: &closest_instance.material,
        })
    }
}
