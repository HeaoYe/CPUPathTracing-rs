mod frame;
mod model;
mod plane;
mod ray;
mod shape;
mod sphere;
mod triangle;

pub use frame::Frame;
pub use model::Model;
pub use plane::Plane;
pub use ray::Ray;
pub use shape::{Bounded, Centroid, Intersection, Shape};
pub use sphere::Sphere;
pub use triangle::Triangle;

#[cfg(debug_assertions)]
pub use shape::IntersectionDebugInfo;
