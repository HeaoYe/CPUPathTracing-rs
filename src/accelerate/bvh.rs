use super::Bounds;
use crate::geometry::{Bounded, Centroid, Intersection, Ray, Shape};

struct BvhTreeNode<P> {
    bounds: Bounds,
    primitives: Vec<P>,
    left: Option<Box<BvhTreeNode<P>>>,
    right: Option<Box<BvhTreeNode<P>>>,
}

impl<P: Bounded> BvhTreeNode<P> {
    fn update_bounds(&mut self) {
        for primitive in &self.primitives {
            self.bounds.extend_bounds(primitive.bounds());
        }
    }
}

pub struct Bvh<P> {
    flattened_nodes: Vec<BvhNode<P>>,
}

impl<P: Bounded + Centroid> Bvh<P> {
    pub fn new(primitives: Vec<P>) -> Self {
        let mut root = BvhTreeNode {
            bounds: Default::default(),
            primitives,
            left: None,
            right: None,
        };
        root.update_bounds();

        Bvh::recursive_split(&mut root);

        let mut flattened_nodes = Vec::new();
        Bvh::recursive_flatten(root, &mut flattened_nodes);

        Bvh { flattened_nodes }
    }

    fn recursive_split(node: &mut BvhTreeNode<P>) {
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

        let mut left = Box::new(BvhTreeNode {
            bounds: Default::default(),
            primitives: left_primitives,
            left: None,
            right: None,
        });
        let mut right = Box::new(BvhTreeNode {
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

struct BvhNode<P> {
    bounds: Bounds,
    primitives: Vec<P>,
    right_child_index: usize,
}

impl<P> Bvh<P> {
    fn recursive_flatten(
        tree_node: BvhTreeNode<P>,
        flattened_nodes: &mut Vec<BvhNode<P>>,
    ) -> usize {
        let BvhTreeNode {
            bounds,
            primitives,
            left,
            right,
        } = tree_node;
        let node = BvhNode {
            bounds,
            primitives,
            right_child_index: 0,
        };
        let parent_index = flattened_nodes.len();
        let is_leaf = !node.primitives.is_empty();
        flattened_nodes.push(node);
        if !is_leaf {
            if let Some(left) = left {
                Bvh::recursive_flatten(*left, flattened_nodes);
            }
            if let Some(right) = right {
                let right_child_index = Bvh::recursive_flatten(*right, flattened_nodes);
                flattened_nodes[parent_index].right_child_index = right_child_index;
            }
        }
        parent_index
    }
}

impl<P: Shape> Shape for Bvh<P> {
    fn intersect(&self, ray: &Ray, t_min: f32, mut t_max: f32) -> Option<Intersection> {
        let mut closest_intersection = None;

        let mut stack = [0u32; 32];
        let mut ptr = 0;
        let mut current_node = 0;

        loop {
            let node = &self.flattened_nodes[current_node];

            if !node.bounds.has_intersection(ray, t_min, t_max) {
                if ptr == 0 {
                    break;
                }
                ptr -= 1;
                current_node = stack[ptr] as usize;
                continue;
            }

            if node.primitives.is_empty() {
                stack[ptr] = node.right_child_index as u32;
                ptr += 1;
                current_node += 1;
            } else {
                for primitive in &node.primitives {
                    if let Some(intersection) = primitive.intersect(ray, t_min, t_max) {
                        t_max = intersection.t;
                        closest_intersection = Some(intersection);
                    }
                }
                if ptr == 0 {
                    break;
                }
                ptr -= 1;
                current_node = stack[ptr] as usize;
            }
        }

        closest_intersection
    }
}

impl<P> Bounded for Bvh<P> {
    fn bounds(&self) -> Bounds {
        self.flattened_nodes.first().unwrap().bounds
    }
}
