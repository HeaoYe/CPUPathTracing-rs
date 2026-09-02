// use super::Integrator;
// use crate::{
//     camera::{CameraModel, PixelSample},
//     geometry::Frame,
//     sample::uniform,
//     scene::{HitInfo, Scene},
//     util::Rng,
// };

// pub struct SimpleRtIntegrator;

// impl Integrator for SimpleRtIntegrator {
//     fn integrate(
//         &self,
//         x: usize,
//         y: usize,
//         sample_index: usize,
//         camera: &CameraModel,
//         scene: &Scene,
//     ) -> Option<PixelSample> {
//         let mut rng = Rng::new(0, ((x + 1) * (y + 1) * sample_index) as u64);

//         let mut ray = camera.generate_ray(
//             glam::IVec2::new(x as i32, y as i32),
//             glam::Vec2::new(rng.uniform(), rng.uniform()),
//         );
//         let mut beta = glam::Vec3::ONE;
//         let mut radiance = glam::Vec3::ZERO;

//         let mut depth = 0;

//         loop {
//             depth += 1;
//             if depth > 16 {
//                 break;
//             }

//             let Some(HitInfo {
//                 intersection,
//                 material,
//             }) = scene.intersect(&ray, 1e-5, f32::INFINITY)
//             else {
//                 break;
//             };

//             radiance += beta * material.emissive;
//             beta *= material.albedo;

//             ray.origin = intersection.hit_point;
//             let frame = Frame::new(intersection.normal);
//             let light_direction;
//             if material.is_specular {
//                 let view_direction = frame.local_from_world(-ray.direction);
//                 light_direction =
//                     glam::vec3(-view_direction.x, view_direction.y, -view_direction.z);
//             } else {
//                 light_direction = uniform::hemisphere(rng.uniform(), rng.uniform());
//             }
//             ray.direction = frame.world_from_local(light_direction);
//         }

//         Some(PixelSample::Radiance(radiance))
//     }
// }
