use crate::{
    accelerate::{Bounds, Bvh},
    geometry::{Bounded, Centroid, Intersection, Ray, Shape},
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

struct ShapeInstance<'a> {
    bounds: Bounds,
    shape: &'a dyn Shape,
    material: Material,
    world_from_object: glam::Affine3A,
    object_from_world: glam::Affine3A,
}

impl<'a> ShapeInstance<'a> {
    pub fn new<T>(shape: &'a T, material: Material, world_from_object: glam::Affine3A) -> Self
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
            world_from_object,
            object_from_world: world_from_object.inverse(),
        }
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
}

pub struct Scene<'a> {
    bvh: Bvh<ShapeInstance<'a>>,
}

impl<'a> SceneBuilder<'a> {
    pub fn add_shape<T>(&mut self, shape: &'a T, material: Material, transform: InstanceTransform)
    where
        T: Shape + Bounded,
    {
        let world_from_object = transform.into_affine();
        self.instances
            .push(ShapeInstance::new(shape, material, world_from_object));
    }

    pub fn build(self) -> Scene<'a> {
        Scene {
            bvh: Bvh::new(self.instances),
        }
    }
}

pub struct HitInfo<'a> {
    pub intersection: Intersection,
    pub material: &'a Material,
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

                HitInfo {
                    intersection: closest_intersection,
                    material: &closest_instance.material,
                }
            },
        )
    }
}
