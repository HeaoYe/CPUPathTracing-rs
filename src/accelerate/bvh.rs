use super::Bounds;
use crate::{
    THREAD_POOL,
    geometry::{Bounded, Centroid, Intersection, Ray, Shape},
};
use std::{
    ops::Range,
    sync::atomic::{AtomicUsize, Ordering},
};

struct BvhTreeNodeLeaf {
    bounds: Bounds,
    depth: usize,
    range: Range<usize>,
}
struct BvhTreeNodeInterior {
    bounds: Bounds,
    depth: usize,
    split_axis: usize,
    left: Box<BvhTreeNode>,
    right: Box<BvhTreeNode>,
}

enum BvhTreeNode {
    Leaf(BvhTreeNodeLeaf),
    Interior(BvhTreeNodeInterior),
}

#[derive(Default)]
struct BvhState {
    total_node_count: AtomicUsize,
    leaf_node_count: usize,
    max_leaf_node_primitive_count: usize,
    max_leaf_node_depth: usize,
}

impl BvhState {
    fn add_leaf(&mut self, leaf: &BvhTreeNodeLeaf) {
        self.leaf_node_count += 1;
        self.max_leaf_node_primitive_count =
            self.max_leaf_node_primitive_count.max(leaf.range.len());
        self.max_leaf_node_depth = self.max_leaf_node_depth.max(leaf.depth);
    }
}

pub struct Bvh<P> {
    flattened_nodes: Vec<BvhNode>,
    ordered_primitives: Vec<P>,
}

impl<P: Bounded> Bvh<P> {
    fn update_bounds(&self, node: &mut BvhTreeNode) {
        let bounds = match node {
            BvhTreeNode::Leaf(leaf) => &mut leaf.bounds,
            BvhTreeNode::Interior(interior) => &mut interior.bounds,
        };
        for primitive in &self.ordered_primitives {
            bounds.extend_bounds(primitive.bounds());
        }
    }
}

impl<P: Bounded + Centroid + Send> Bvh<P> {
    pub fn new(primitives: Vec<P>) -> Self {
        let mut bvh = Self {
            flattened_nodes: Vec::new(),
            ordered_primitives: primitives,
        };

        if bvh.ordered_primitives.is_empty() {
            return bvh;
        }

        let mut root = BvhTreeNode::Leaf(BvhTreeNodeLeaf {
            bounds: Default::default(),
            depth: 1,
            range: 0..bvh.ordered_primitives.len(),
        });

        bvh.update_bounds(&mut root);
        let mut state = BvhState::default();

        // SAFETY: 使用 THREAD_POOL.wait() 确保多线程构建时的参数有效。
        unsafe {
            Bvh::recursive_split(&mut root, &mut bvh.ordered_primitives, &state);
        }
        THREAD_POOL.wait();

        let total_node_count = state.total_node_count.load(Ordering::Relaxed);
        bvh.flattened_nodes.reserve(total_node_count);
        bvh.recursive_flatten(root, &mut state);

        println!("Total Node Count: {}", total_node_count);
        println!("Leaf Node Count: {}", state.leaf_node_count);
        println!("Primitive Count: {}", bvh.ordered_primitives.len());
        println!(
            "Mean Leaf Node Primitive Count: {}",
            bvh.ordered_primitives.len() as f32 / state.leaf_node_count as f32
        );
        println!(
            "Max Leaf Node Primitive Count: {}",
            state.max_leaf_node_primitive_count
        );
        println!("Max Leaf Node Depth: {}", state.max_leaf_node_depth);

        bvh
    }
}

#[derive(Default)]
struct BvhSplit {
    cost: f32,
    split_axis: usize,
    split_index: usize,
    left_bounds: Bounds,
    right_bounds: Bounds,
}

#[derive(Default, Clone, Copy)]
struct Bucket {
    bounds: Bounds,
    primitive_count: usize,
}

impl Bucket {
    fn add_primitive(&mut self, bounds: Bounds) {
        self.bounds.extend_bounds(bounds);
        self.primitive_count += 1;
    }

    fn extend(&mut self, bucket: Bucket) {
        self.bounds.extend_bounds(bucket.bounds);
        self.primitive_count += bucket.primitive_count;
    }
}

impl<P: Bounded + Centroid + Send> Bvh<P> {
    // SAFETY: 调用者必须使用 THREAD_POOL.wait() 确保多线程构建使用的参数有效。
    unsafe fn recursive_split(node: &mut BvhTreeNode, primitive_slice: &mut [P], state: &BvhState) {
        state.total_node_count.fetch_add(1, Ordering::Relaxed);

        let BvhTreeNode::Leaf(leaf) = node else {
            panic!()
        };

        if primitive_slice.len() <= 4 || leaf.depth == 32 {
            return;
        }

        const BUCKET_COUNT: usize = 12;

        let diag = leaf.bounds.diagonal();
        let mut min_split = BvhSplit {
            cost: f32::INFINITY,
            split_axis: 3,
            ..Default::default()
        };
        let mut buckets = [[Bucket::default(); 3]; BUCKET_COUNT];

        let scale = BUCKET_COUNT as f32 / diag;
        for primitive in primitive_slice.iter() {
            let centroid = primitive.centroid();
            let bounds = primitive.bounds();

            let bucket_index = ((centroid - leaf.bounds.b_min()) * scale)
                .as_uvec3()
                .min(glam::UVec3::splat(BUCKET_COUNT as u32 - 1));

            for axis in 0..3 {
                buckets[bucket_index[axis] as usize][axis].add_primitive(bounds);
            }
        }

        let mut prefixes = [[Bucket::default(); 3]; BUCKET_COUNT - 1];
        let mut acc = [Bucket::default(); 3];
        for prefix_end in 0..BUCKET_COUNT - 1 {
            for axis in 0..3 {
                acc[axis].extend(buckets[prefix_end][axis]);
                prefixes[prefix_end][axis] = acc[axis];
            }
        }

        let mut suffix = [Bucket::default(); 3];

        for suffix_start in (1..BUCKET_COUNT).rev() {
            for axis in 0..3 {
                suffix[axis].extend(buckets[suffix_start][axis]);

                let left = prefixes[suffix_start - 1][axis];
                let right = suffix[axis];

                if left.primitive_count == 0 || right.primitive_count == 0 {
                    continue;
                }

                let cost = left.primitive_count as f32 * left.bounds.area()
                    + right.primitive_count as f32 * right.bounds.area();
                if cost < min_split.cost {
                    min_split = BvhSplit {
                        cost,
                        split_axis: axis,
                        split_index: suffix_start,
                        left_bounds: left.bounds,
                        right_bounds: right.bounds,
                    }
                }
            }
        }

        if min_split.split_axis == 3 {
            return;
        }

        if 0.5 + min_split.cost / leaf.bounds.area() >= primitive_slice.len() as f32 {
            return;
        }

        let mut head = 0;
        let mut tail = primitive_slice.len();

        while head < tail {
            let tail_index = tail - 1;

            let head_primitive = &primitive_slice[head];
            let tail_primitive = &primitive_slice[tail_index];

            let head_centroid = head_primitive.centroid()[min_split.split_axis];
            let tail_centroid = tail_primitive.centroid()[min_split.split_axis];

            let head_bucket_index = (((head_centroid - leaf.bounds.b_min()[min_split.split_axis])
                * scale[min_split.split_axis]) as u32)
                .min(BUCKET_COUNT as u32 - 1);
            let tail_bucket_index = (((tail_centroid - leaf.bounds.b_min()[min_split.split_axis])
                * scale[min_split.split_axis]) as u32)
                .min(BUCKET_COUNT as u32 - 1);

            let head_is_left = head_bucket_index < min_split.split_index as u32;
            let tail_is_left = tail_bucket_index < min_split.split_index as u32;

            if head_is_left && tail_is_left {
                head += 1;
            } else if !head_is_left && !tail_is_left {
                tail -= 1;
            } else if head_is_left && !tail_is_left {
                tail -= 1;
                head += 1;
            } else {
                primitive_slice.swap(head, tail_index);
                tail -= 1;
                head += 1;
            }
        }

        let primitive_count = leaf.range.len();
        let mid = leaf.range.start + head;
        let left = Box::new(BvhTreeNode::Leaf(BvhTreeNodeLeaf {
            bounds: min_split.left_bounds,
            range: leaf.range.start..mid,
            depth: leaf.depth + 1,
        }));
        let right = Box::new(BvhTreeNode::Leaf(BvhTreeNodeLeaf {
            bounds: min_split.right_bounds,
            range: mid..leaf.range.end,
            depth: leaf.depth + 1,
        }));
        *node = BvhTreeNode::Interior(BvhTreeNodeInterior {
            bounds: leaf.bounds,
            depth: leaf.depth,
            split_axis: min_split.split_axis,
            left,
            right,
        });

        let sub_primitive_slice = primitive_slice.split_at_mut(head);
        let BvhTreeNode::Interior(interior) = node else {
            unreachable!()
        };
        let left_node = &mut *interior.left;
        let right_node = &mut *interior.right;
        if primitive_count > 128 * 1024 {
            unsafe {
                THREAD_POOL.add_scope_task_unchecked(Box::new(move || {
                    Bvh::recursive_split(left_node, sub_primitive_slice.0, state);
                }));
                THREAD_POOL.add_scope_task_unchecked(Box::new(move || {
                    Bvh::recursive_split(right_node, sub_primitive_slice.1, state);
                }));
            }
        } else {
            unsafe {
                Bvh::recursive_split(&mut interior.left, sub_primitive_slice.0, state);
                Bvh::recursive_split(&mut interior.right, sub_primitive_slice.1, state);
            }
        }
    }
}

#[repr(C, align(32))]
struct BvhNode {
    bounds: Bounds,
    index: u32,
    primitive_count: u16,
    depth: u8,
    split_axis: u8,
}

const _: [(); 32] = [(); std::mem::size_of::<BvhNode>()];
const _: [(); 32] = [(); std::mem::align_of::<BvhNode>()];

impl<P> Bvh<P> {
    fn recursive_flatten(&mut self, tree_node: BvhTreeNode, state: &mut BvhState) -> usize {
        match tree_node {
            BvhTreeNode::Leaf(leaf) => {
                state.add_leaf(&leaf);
                let index = self.flattened_nodes.len();
                self.flattened_nodes.push(BvhNode {
                    bounds: leaf.bounds,
                    index: leaf.range.start as u32,
                    primitive_count: u16::try_from(leaf.range.len()).unwrap(),
                    depth: leaf.depth as u8,
                    split_axis: 0,
                });
                index
            }
            BvhTreeNode::Interior(interior) => {
                let index = self.flattened_nodes.len();
                self.flattened_nodes.push(BvhNode {
                    bounds: interior.bounds,
                    index: 0,
                    primitive_count: 0,
                    depth: interior.depth as u8,
                    split_axis: interior.split_axis as u8,
                });
                self.recursive_flatten(*interior.left, state);
                self.flattened_nodes[index].index =
                    self.recursive_flatten(*interior.right, state) as u32;
                index
            }
        }
    }
}

impl<P> Bvh<P> {
    pub fn intersect_with<'a, Intersect, Finalize, HitData, Output>(
        &'a self,
        ray: &Ray,
        t_min: f32,
        mut t_max: f32,
        intersect: Intersect,
        finalize: Finalize,
    ) -> Option<Output>
    where
        Intersect: Fn(&'a P, &Ray, f32, f32) -> Option<(f32, HitData)>,
        Finalize: FnOnce(HitData) -> Output,
    {
        if self.ordered_primitives.is_empty() {
            return None;
        }

        let mut closest_hit_data = None;

        #[cfg(debug_assertions)]
        let mut bounds_test_count = 0usize;
        #[cfg(debug_assertions)]
        let mut primitive_test_count = 0usize;

        let mut stack = [0u32; 32];
        let mut ptr = 0;
        let mut current_node = 0u32;

        let dir_is_neg = [
            ray.direction.x < 0.0,
            ray.direction.y < 0.0,
            ray.direction.z < 0.0,
        ];

        let inv_direction = 1.0 / ray.direction;

        loop {
            let node = &self.flattened_nodes[current_node as usize];

            #[cfg(debug_assertions)]
            {
                bounds_test_count += 1;
            }

            if !node
                .bounds
                .has_intersection_inv_dir(ray.origin, inv_direction, t_min, t_max)
            {
                if ptr == 0 {
                    break;
                }
                ptr -= 1;
                current_node = stack[ptr];
                continue;
            }

            if node.primitive_count == 0 {
                if dir_is_neg[node.split_axis as usize] {
                    stack[ptr] = current_node + 1;
                    ptr += 1;
                    current_node = node.index;
                } else {
                    current_node += 1;
                    stack[ptr] = node.index;
                    ptr += 1;
                }
            } else {
                #[cfg(debug_assertions)]
                {
                    primitive_test_count += node.primitive_count as usize;
                }
                for primitive in &self.ordered_primitives
                    [node.index as usize..(node.index + node.primitive_count as u32) as usize]
                {
                    if let Some((t, custom_data)) = intersect(primitive, ray, t_min, t_max) {
                        t_max = t;
                        closest_hit_data = Some(custom_data);
                    }
                }
                if ptr == 0 {
                    break;
                }
                ptr -= 1;
                current_node = stack[ptr];
            }
        }

        #[cfg(debug_assertions)]
        {
            let mut ray_debug_info = ray.debug_info.borrow_mut();
            ray_debug_info.bounds_test_count += bounds_test_count;
            ray_debug_info.primitive_test_count += primitive_test_count;
        }

        let closest_hit_data = closest_hit_data?;
        Some(finalize(closest_hit_data))
    }
}

impl<P: Shape> Shape for Bvh<P> {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        self.intersect_with(
            ray,
            t_min,
            t_max,
            |primitive, ray, t_min, t_max| {
                let intersection = primitive.intersect(ray, t_min, t_max)?;
                Some((intersection.t, intersection))
            },
            |data| data,
        )
    }
}

impl<P> Bounded for Bvh<P> {
    fn bounds(&self) -> Bounds {
        self.flattened_nodes
            .first()
            .map_or(Default::default(), |node| node.bounds)
    }
}
