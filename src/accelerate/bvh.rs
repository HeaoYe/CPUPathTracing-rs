use super::Bounds;
use crate::geometry::{Bounded, Centroid, Intersection, Ray, Shape};

struct BvhTreeNode<P> {
    bounds: Bounds,
    primitives: Vec<P>,
    left: Option<Box<BvhTreeNode<P>>>,
    right: Option<Box<BvhTreeNode<P>>>,
    depth: usize,
}

impl<P: Bounded> BvhTreeNode<P> {
    fn update_bounds(&mut self) {
        for primitive in &self.primitives {
            self.bounds.extend_bounds(primitive.bounds());
        }
    }
}

pub struct Bvh<P> {
    flattened_nodes: Vec<BvhNode>,
    ordered_primitives: Vec<P>,
}

impl<P: Bounded + Centroid> Bvh<P> {
    pub fn new(primitives: Vec<P>) -> Self {
        let mut root = BvhTreeNode {
            bounds: Default::default(),
            primitives,
            left: None,
            right: None,
            depth: 1,
        };
        root.update_bounds();

        Bvh::recursive_split(&mut root);

        let mut flattened_nodes = Vec::new();
        let mut ordered_primitives = Vec::new();
        Bvh::recursive_flatten(root, &mut flattened_nodes, &mut ordered_primitives);

        Bvh {
            flattened_nodes,
            ordered_primitives,
        }
    }

    fn recursive_split(node: &mut BvhTreeNode<P>) {
        if node.primitives.len() <= 4 || node.depth == 32 {
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
            depth: node.depth + 1,
        });
        let mut right = Box::new(BvhTreeNode {
            bounds: Default::default(),
            primitives: right_primitives,
            left: None,
            right: None,
            depth: node.depth + 1,
        });
        left.update_bounds();
        right.update_bounds();

        node.left = Some(left);
        node.right = Some(right);
        Bvh::recursive_split(node.left.as_mut().unwrap());
        Bvh::recursive_split(node.right.as_mut().unwrap());
    }
}

#[repr(C, align(32))]
struct BvhNode {
    bounds: Bounds,
    index: u32,
    primitive_count: u16,
    depth: u8,
    _padding: u8,
}

const _: [(); 32] = [(); std::mem::size_of::<BvhNode>()];
const _: [(); 32] = [(); std::mem::align_of::<BvhNode>()];

impl<P> Bvh<P> {
    fn recursive_flatten(
        tree_node: BvhTreeNode<P>,
        flattened_nodes: &mut Vec<BvhNode>,
        ordered_primitives: &mut Vec<P>,
    ) -> usize {
        let BvhTreeNode {
            bounds,
            primitives,
            left,
            right,
            depth,
        } = tree_node;
        let node = BvhNode {
            bounds,
            index: 0,
            primitive_count: u16::try_from(primitives.len()).unwrap(),
            depth: depth as u8,
            _padding: 0,
        };
        let parent_index = flattened_nodes.len();
        flattened_nodes.push(node);
        if primitives.is_empty() {
            if let Some(left) = left {
                Bvh::recursive_flatten(*left, flattened_nodes, ordered_primitives);
            }
            if let Some(right) = right {
                let right_child_index =
                    Bvh::recursive_flatten(*right, flattened_nodes, ordered_primitives);
                flattened_nodes[parent_index].index = right_child_index as u32;
            }
        } else {
            flattened_nodes[parent_index].index = ordered_primitives.len() as u32;
            ordered_primitives.extend(primitives);
        }
        parent_index
    }
}

impl<P: Shape> Shape for Bvh<P> {
    fn intersect(&self, ray: &Ray, t_min: f32, mut t_max: f32) -> Option<Intersection> {
        let mut closest_intersection = None;

        #[cfg(debug_assertions)]
        let mut debug_info = crate::geometry::IntersectionDebugInfo::default();

        let mut stack = [0u32; 32];
        let mut ptr = 0;
        let mut current_node = 0;

        loop {
            let node = &self.flattened_nodes[current_node];

            #[cfg(debug_assertions)]
            {
                debug_info.bounds_test_count += 1;
            }

            if !node.bounds.has_intersection(ray, t_min, t_max) {
                if ptr == 0 {
                    break;
                }
                ptr -= 1;
                current_node = stack[ptr] as usize;
                continue;
            }

            if node.primitive_count == 0 {
                stack[ptr] = node.index as u32;
                ptr += 1;
                current_node += 1;
            } else {
                #[cfg(debug_assertions)]
                {
                    debug_info.triangle_test_count += node.primitive_count as usize;
                }
                for primitive in &self.ordered_primitives
                    [node.index as usize..(node.index + node.primitive_count as u32) as usize]
                {
                    if let Some(intersection) = primitive.intersect(ray, t_min, t_max) {
                        t_max = intersection.t;
                        closest_intersection = Some(intersection);
                        #[cfg(debug_assertions)]
                        {
                            debug_info.bvh_depth = node.depth as usize;
                        }
                    }
                }
                if ptr == 0 {
                    break;
                }
                ptr -= 1;
                current_node = stack[ptr] as usize;
            }
        }

        let closest_intersection = closest_intersection?;
        Some(Intersection {
            #[cfg(debug_assertions)]
            debug_info,
            ..closest_intersection
        })
    }
}

impl<P> Bounded for Bvh<P> {
    fn bounds(&self) -> Bounds {
        self.flattened_nodes.first().unwrap().bounds
    }
}
