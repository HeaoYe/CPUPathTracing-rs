use super::{Intersection, Ray, Shape, Triangle};

pub struct Model {
    triangles: Vec<Triangle>,
}

impl Model {
    pub fn load(filename: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        use std::{
            fs::File,
            io::{BufRead, BufReader},
        };

        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let mut triangles = Vec::new();
        let mut positions = Vec::new();
        let mut normals = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let mut tokens = line.split_whitespace();

            let Some(type_) = tokens.next() else {
                continue;
            };
            match type_ {
                "v" => {
                    // v 22 12 12
                    let position = glam::Vec3::new(
                        tokens.next().unwrap().parse().unwrap(),
                        tokens.next().unwrap().parse().unwrap(),
                        tokens.next().unwrap().parse().unwrap(),
                    );
                    positions.push(position);
                }
                "vn" => {
                    // vn 22 12 12
                    let normal = glam::Vec3::new(
                        tokens.next().unwrap().parse().unwrap(),
                        tokens.next().unwrap().parse().unwrap(),
                        tokens.next().unwrap().parse().unwrap(),
                    );
                    normals.push(normal);
                }
                "f" => {
                    // f 1//4  2//3  3//2
                    // T { 0, 1, 2   3, 2, 1}
                    let mut position_idx = glam::USizeVec3::ZERO;
                    let mut normal_idx = glam::USizeVec3::ZERO;
                    for i in 0..3 {
                        let mut part = tokens.next().unwrap().split("//");
                        position_idx[i] = part.next().unwrap().parse().unwrap();
                        normal_idx[i] = part.next().unwrap().parse().unwrap();
                    }
                    triangles.push(Triangle {
                        p0: positions[position_idx[0] - 1],
                        p1: positions[position_idx[1] - 1],
                        p2: positions[position_idx[2] - 1],
                        n0: normals[normal_idx[0] - 1],
                        n1: normals[normal_idx[1] - 1],
                        n2: normals[normal_idx[2] - 1],
                    });
                }
                _ => {
                    continue;
                }
            }
        }

        Ok(Self { triangles })
    }
}

impl Shape for Model {
    fn intersect(&self, ray: &Ray, t_min: f32, mut t_max: f32) -> Option<Intersection> {
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
