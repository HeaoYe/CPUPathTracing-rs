use super::{Intersection, Ray, Shape, Triangle};
use crate::{
    accelerate::Bounds,
    util::{parse_obj, profile},
};

pub struct Model {
    triangles: Vec<Triangle>,
    bounds: Bounds,
}

impl Model {
    pub fn load(filename: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        profile!("Load model {}", filename.as_ref().display());

        let parsed_obj = parse_obj(filename)?;

        let mut triangles = Vec::with_capacity(parsed_obj.triangles.len());
        for indices in parsed_obj.triangles {
            let p0 = parsed_obj.vertices[indices[0].vertex];
            let p1 = parsed_obj.vertices[indices[1].vertex];
            let p2 = parsed_obj.vertices[indices[2].vertex];
            let n0 = parsed_obj.normals[indices[0].normal];
            let n1 = parsed_obj.normals[indices[1].normal];
            let n2 = parsed_obj.normals[indices[2].normal];
            if n0 == glam::Vec3::ZERO || n1 == glam::Vec3::ZERO || n2 == glam::Vec3::ZERO {
                triangles.push(Triangle::from_points(p0, p1, p2));
            } else {
                triangles.push(Triangle {
                    p0,
                    p1,
                    p2,
                    n0,
                    n1,
                    n2,
                });
            }
        }

        let mut model = Self {
            triangles,
            bounds: Default::default(),
        };
        model.build();
        Ok(model)
    }

    fn build(&mut self) {
        for triangle in &self.triangles {
            self.bounds.expand_point(triangle.p0);
            self.bounds.expand_point(triangle.p1);
            self.bounds.expand_point(triangle.p2);
        }
    }
}

impl Shape for Model {
    fn intersect(&self, ray: &Ray, t_min: f32, mut t_max: f32) -> Option<Intersection> {
        if !self.bounds.has_intersection(ray, t_min, t_max) {
            return None;
        }

        let mut closest_intersection = None;
        for triangle in &self.triangles {
            if let Some(intersection) = triangle.intersect(ray, t_min, t_max) {
                t_max = intersection.t;
                closest_intersection = Some(intersection);
            }
        }
        closest_intersection
    }
}
