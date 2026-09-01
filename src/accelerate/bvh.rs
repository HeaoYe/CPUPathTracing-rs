use super::Bounds;
use crate::geometry::{Bounded, Centroid, Intersection, Ray, Shape};

struct BvhNode<P> {
    bounds: Bounds,
    primitives: Vec<P>,
    left: Option<Box<BvhNode<P>>>,
    right: Option<Box<BvhNode<P>>>,
}

impl<P: Bounded> BvhNode<P> {
    fn update_bounds(&mut self) {
        for primitive in &self.primitives {
            self.bounds.extend_bounds(primitive.bounds());
        }
    }
}

pub struct Bvh<P> {
    root: BvhNode<P>,
}

impl<P: Bounded + Centroid> Bvh<P> {
    pub fn new(primitives: Vec<P>) -> Self {
        let mut root = BvhNode {
            bounds: Default::default(),
            primitives,
            left: None,
            right: None,
        };
        root.update_bounds();

        let mut bvh = Bvh { root };
        Bvh::recursive_split(&mut bvh.root);
        bvh
    }

    fn recursive_split(node: &mut BvhNode<P>) {
        if node.primitives.len() == 1 {
            return;
        }

        let diag = node.bounds.diag();
        let max_axis = diag.max_position();
        let mid = node.bounds.b_min()[max_axis] + diag[max_axis] * 0.5;

        let mut left_primitives = Vec::new();
        let mut right_primitives = Vec::new();
        for primitive in node.primitives.drain(..) {
            let primitive_centroid = primitive.centroid();
            if primitive_centroid[max_axis] < mid {
                left_primitives.push(primitive);
            } else {
                right_primitives.push(primitive);
            }
        }

        if left_primitives.is_empty() || right_primitives.is_empty() {
            node.primitives = left_primitives;
            node.primitives.extend(right_primitives);
            return;
        }

        let mut left = Box::new(BvhNode {
            bounds: Default::default(),
            primitives: left_primitives,
            left: None,
            right: None,
        });
        let mut right = Box::new(BvhNode {
            bounds: Default::default(),
            primitives: right_primitives,
            left: None,
            right: None,
        });
        left.update_bounds();
        right.update_bounds();

        node.left = Some(left);
        node.right = Some(right);
        Bvh::recursive_split(node.left.as_mut().unwrap());
        Bvh::recursive_split(node.right.as_mut().unwrap());
    }
}

impl<P: Shape> Bvh<P> {
    fn recursive_intersect(
        node: &BvhNode<P>,
        ray: &Ray,
        t_min: f32,
        t_max: &mut f32,
        closest_intersection: &mut Option<Intersection>,
    ) {
        if !node.bounds.has_intersection(ray, t_min, *t_max) {
            return;
        }

        if node.primitives.is_empty() {
            if let Some(left) = &node.left {
                Bvh::recursive_intersect(left, ray, t_min, t_max, closest_intersection);
            }

            if let Some(right) = &node.right {
                Bvh::recursive_intersect(right, ray, t_min, t_max, closest_intersection);
            }
        } else {
            for primitive in &node.primitives {
                if let Some(intersection) = primitive.intersect(ray, t_min, *t_max) {
                    *t_max = intersection.t;
                    *closest_intersection = Some(intersection);
                }
            }
        }
    }
}

impl<P: Shape> Shape for Bvh<P> {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let mut closest_intersection = None;
        let mut t_max = t_max;
        Bvh::recursive_intersect(
            &self.root,
            ray,
            t_min,
            &mut t_max,
            &mut closest_intersection,
        );
        closest_intersection
    }
}

impl<P> Bounded for Bvh<P> {
    fn bounds(&self) -> Bounds {
        self.root.bounds
    }
}
