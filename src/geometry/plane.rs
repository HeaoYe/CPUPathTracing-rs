use super::{Bounded, Centroid, Intersection, Ray, Sampleable, Shape, SurfaceSample};
use crate::{accelerate::Bounds, sample::uniform, util::Rng};

pub struct Plane {
    point: glam::Vec3,
    normal: glam::Vec3,
    radius: f32,
    bounds: Bounds,
    x_axis: glam::Vec3,
    z_axis: glam::Vec3,
}

impl Plane {
    pub fn new(point: glam::Vec3, normal: glam::Vec3, radius: f32) -> Self {
        let y_axis = normal.normalize();
        let up = if y_axis.y.abs() < 0.99999 {
            glam::Vec3::Y
        } else {
            glam::Vec3::Z
        };
        let x_axis = y_axis.cross(up).normalize();
        let z_axis = x_axis.cross(y_axis).normalize();

        let bounds_local = Bounds::new(
            glam::vec3(-radius, -0.001, -radius),
            glam::vec3(radius, 0.001, radius),
        );
        let mut bounds = Bounds::default();

        for idx in 0..8 {
            let corner_object = bounds_local.corner(idx);
            bounds.extend_point(
                point
                    + corner_object.x * x_axis
                    + corner_object.y * y_axis
                    + corner_object.z * z_axis,
            );
        }

        Self {
            point,
            normal,
            radius,
            bounds,
            x_axis,
            z_axis,
        }
    }
}

impl Shape for Plane {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let hit_t = self.normal.dot(self.point - ray.origin) / self.normal.dot(ray.direction);
        let hit_point_to_center = ray.at(hit_t) - self.point;

        if hit_t > t_min
            && hit_t < t_max
            && hit_point_to_center.length_squared() < self.radius * self.radius
        {
            Some(Intersection::new(ray, hit_t, self.normal))
        } else {
            None
        }
    }
}

impl Bounded for Plane {
    fn bounds(&self) -> Bounds {
        self.bounds
    }
}

impl Centroid for Plane {
    fn centroid(&self) -> glam::Vec3 {
        self.point
    }
}

impl Sampleable for Plane {
    fn area(&self) -> f32 {
        std::f32::consts::PI * self.radius * self.radius
    }

    fn sample(&self, rng: &mut Rng) -> Option<SurfaceSample> {
        let position = uniform::disk(rng.uniform(), rng.uniform()) * self.radius;
        let position = self.point + position.x * self.x_axis + position.y * self.z_axis;
        Some(SurfaceSample {
            position,
            normal: self.normal,
            pdf: 1.0 / self.area(),
        })
    }
}
