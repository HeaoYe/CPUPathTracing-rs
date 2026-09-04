use crate::{
    THREAD_POOL,
    color::{ColorSpace, EncodedRgb, LinearRgb, SRGB, Xyz},
    spectrum::{CIE_STD_ILLUMNT_D65, LAMBDA_MAX, LAMBDA_MIN, Spectrum, X_CMF, Y_CMF, Z_CMF},
    util::{Progress, profile},
};
use std::sync::LazyLock;

fn sigmoid_and_derivate(x: f64) -> (f64, f64) {
    let a = 1.0 + x * x;
    let t = a.sqrt();
    (0.5 * x / t + 0.5, 1.0 / (2.0 * t * a))
}

fn cielab_f(t: f64) -> f64 {
    const DELTA: f64 = 6.0 / 29.0;
    if t > DELTA * DELTA * DELTA {
        t.cbrt()
    } else {
        t / (3.0 * DELTA * DELTA) + 4.0 / 29.0
    }
}

fn cielab_f_and_derivate(t: f64) -> (f64, f64) {
    const DELTA: f64 = 6.0 / 29.0;
    if t > DELTA * DELTA * DELTA {
        let x = t.cbrt();
        (x, 1.0 / (3.0 * x * x))
    } else {
        (
            t / (3.0 * DELTA * DELTA) + 4.0 / 29.0,
            1.0 / (3.0 * DELTA * DELTA),
        )
    }
}

struct OptimizeContext<'a> {
    color_space: &'a ColorSpace,
    illuminant_white: &'a Spectrum<'a>,
    k: f64,
    xyz_white: glam::DVec3,
}

impl<'a> OptimizeContext<'a> {
    fn new(color_space: &'a ColorSpace, illuminant_white: &'a Spectrum<'a>) -> Self {
        let mut k = 0.0;
        for lambda in LAMBDA_MIN..=LAMBDA_MAX {
            k += illuminant_white.eval(lambda as f32) as f64 * Y_CMF.eval(lambda as f32) as f64;
        }
        k = 1.0 / k;
        let xyz_white = Xyz::from_chromaticity(color_space.white(), 1.0).as_dvec3();

        Self {
            color_space,
            illuminant_white,
            k,
            xyz_white,
        }
    }
}

#[derive(Clone, Copy)]
struct OptimizeOptions {
    init_mu: f64,
    max_mu: f64,
    max_iterations: usize,
    residual_squared_epsilon: f64,
}

impl Default for OptimizeOptions {
    fn default() -> Self {
        Self {
            init_mu: 1e-7,
            max_mu: 1e12,
            max_iterations: 1024,
            residual_squared_epsilon: 1e-8,
        }
    }
}

#[derive(Clone, Copy)]
struct Evaluation {
    theta: glam::DVec3,

    cielab: glam::DVec3,
    jacobian: glam::DMat3,
}

impl Evaluation {
    fn eval(ctx: &OptimizeContext, theta: glam::DVec3) -> Self {
        let mut xyz = glam::DVec3::ZERO;
        let mut dxyz_dtheta = glam::DMat3::ZERO;
        let mut dlab_dxyz = glam::DMat3::ZERO;

        for lambda in LAMBDA_MIN..=LAMBDA_MAX {
            let u = (lambda - LAMBDA_MIN) as f64 / (LAMBDA_MAX - LAMBDA_MIN) as f64;
            let q = ((theta[2] * u) + theta[1]) * u + theta[0];
            let (s, ds) = sigmoid_and_derivate(q);
            let ki = ctx.k * ctx.illuminant_white.eval(lambda as f32) as f64;
            let xyz_cmfs = glam::dvec3(
                X_CMF.eval(lambda as f32) as f64,
                Y_CMF.eval(lambda as f32) as f64,
                Z_CMF.eval(lambda as f32) as f64,
            );
            xyz += ki * s * xyz_cmfs;
            dxyz_dtheta.x_axis += ki * ds * xyz_cmfs;
            dxyz_dtheta.y_axis += ki * ds * xyz_cmfs * u;
            dxyz_dtheta.z_axis += ki * ds * xyz_cmfs * u * u;
        }

        let xyz_normalized = xyz / ctx.xyz_white;
        let (fx, mut dfx) = cielab_f_and_derivate(xyz_normalized.x);
        let (fy, mut dfy) = cielab_f_and_derivate(xyz_normalized.y);
        let (fz, mut dfz) = cielab_f_and_derivate(xyz_normalized.z);
        dfx /= ctx.xyz_white.x;
        dfy /= ctx.xyz_white.y;
        dfz /= ctx.xyz_white.z;
        let cielab = glam::dvec3(116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz));
        dlab_dxyz.x_axis = glam::dvec3(0.0, 500.0 * dfx, 0.0);
        dlab_dxyz.y_axis = glam::dvec3(116.0 * dfy, -500.0 * dfy, 200.0 * dfy);
        dlab_dxyz.z_axis = glam::dvec3(0.0, 0.0, -200.0 * dfz);

        Self {
            theta,
            cielab,
            jacobian: -dlab_dxyz * dxyz_dtheta,
        }
    }
}

fn cholesky(jtj: glam::DMat3, mu_d: glam::DVec3, b: glam::DVec3) -> glam::DVec3 {
    let l00 = (jtj.x_axis.x + mu_d.x).sqrt();
    let l10 = jtj.y_axis.x / l00;
    let l20 = jtj.z_axis.x / l00;
    let l11 = (jtj.y_axis.y + mu_d.y - l10 * l10).sqrt();
    let l21 = (jtj.z_axis.y - l10 * l20) / l11;
    let l22 = (jtj.z_axis.z + mu_d.z - l20 * l20 - l21 * l21).sqrt();

    let y0 = b.x / l00;
    let y1 = (b.y - l10 * y0) / l11;
    let y2 = (b.z - l20 * y0 - l21 * y1) / l22;

    let dtheta2 = y2 / l22;
    let dtheta1 = (y1 - l21 * dtheta2) / l11;
    let dtheta0 = (y0 - l10 * dtheta1 - l20 * dtheta2) / l00;

    glam::dvec3(dtheta0, dtheta1, dtheta2)
}

fn optimize(
    ctx: &OptimizeContext,
    rgb_target: LinearRgb,
    initial: Evaluation,
    options: OptimizeOptions,
) -> Evaluation {
    let xyz_target = ctx.color_space.xyz_from_rgb(rgb_target).as_dvec3() / ctx.xyz_white;
    let cief_target = glam::dvec3(
        cielab_f(xyz_target.x),
        cielab_f(xyz_target.y),
        cielab_f(xyz_target.z),
    );
    let cielab_target = glam::dvec3(
        116.0 * cief_target.y - 16.0,
        500.0 * (cief_target.x - cief_target.y),
        200.0 * (cief_target.y - cief_target.z),
    );

    let mut current = initial;
    let mut mu = options.init_mu;
    for _ in 0..options.max_iterations {
        let r = cielab_target - current.cielab;
        let rtr = r.length_squared();
        if rtr < options.residual_squared_epsilon {
            return current;
        }
        let jt = current.jacobian.transpose();
        let jtj = jt * current.jacobian;
        let d = glam::dvec3(
            jtj.x_axis.x.max(1e-3),
            jtj.y_axis.y.max(1e-3),
            jtj.z_axis.z.max(1e-3),
        );
        let b = -jt * r;

        let mut v = 2.0;
        loop {
            if mu > options.max_mu {
                return current;
            }
            let dtheta = cholesky(jtj, mu * d, b);

            let r_pred = r + current.jacobian * dtheta;
            let df_pred = rtr - r_pred.length_squared();

            let candidate = Evaluation::eval(ctx, current.theta + dtheta);

            let r_actual = cielab_target - candidate.cielab;
            let df_actual = rtr - r_actual.length_squared();

            let rho = df_actual / df_pred;
            if rho > 0.0 {
                current = candidate;
                let t = 2.0 * rho - 1.0;
                mu *= (1.0 - t * t * t).max(1.0 / 3.0);
                break;
            } else {
                mu *= v;
                v *= 2.0;
            }
        }
    }

    current
}

const LUT_RESOLUTION: usize = 64;
const LUT_MAGIC: u32 = 0x87d4_b1a6;

type LutCell = [glam::Vec3; LUT_RESOLUTION];

pub struct ColorLut<'a> {
    color_space: &'a ColorSpace,
    illuminant: &'a Spectrum<'a>,

    alpha_nodes: [f64; LUT_RESOLUTION],

    // data [i][y][x] [alpha] -> Vec3(theta0, theta1, theta2)
    data: Vec<LutCell>,
}

impl<'a> ColorLut<'a> {
    pub fn new(
        filename: impl AsRef<std::path::Path>,
        color_space: &'a ColorSpace,
        illuminant: &'a Spectrum<'a>,
    ) -> Self {
        let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
        let alpha_nodes = std::array::from_fn(|i| {
            let t = i as f64 / (LUT_RESOLUTION - 1) as f64;
            smooth(smooth(t))
        });

        let mut lut = Self {
            color_space,
            illuminant,
            alpha_nodes,
            data: vec![[glam::Vec3::ZERO; LUT_RESOLUTION]; 3 * LUT_RESOLUTION * LUT_RESOLUTION],
        };

        let loaded = lut.load(filename.as_ref()).unwrap_or_else(|err| {
            eprintln!("Failed to load color LUT: {}", err);
            false
        });

        if !loaded {
            lut.generate();
            if let Err(err) = lut.save(filename.as_ref()) {
                println!("Failed to save color LUT: {}", err)
            }
        }

        lut
    }

    pub fn color_space(&self) -> &ColorSpace {
        self.color_space
    }

    pub fn illuminant(&self) -> &Spectrum<'a> {
        self.illuminant
    }

    pub fn lookup_rgb8(&self, r: u8, g: u8, b: u8) -> Spectrum<'_> {
        self.lookup_encoded(EncodedRgb::from_quantized(r as u32, g as u32, b as u32, 8))
    }

    pub fn lookup_encoded(&self, encoded_rgb: EncodedRgb) -> Spectrum<'_> {
        self.lookup_linear(self.color_space.decode(encoded_rgb))
    }

    pub fn lookup_linear(&self, linear_rgb: LinearRgb) -> Spectrum<'_> {
        let color = glam::vec3(linear_rgb.r(), linear_rgb.g(), linear_rgb.b())
            .clamp(glam::Vec3::ZERO, glam::Vec3::ONE);

        let i = color.max_position();
        let alpha = color[i];
        if alpha == 0.0 {
            return Spectrum::sigmoid(f32::NEG_INFINITY, 0.0, 0.0);
        }

        let x = color[(i + 1) % 3] / alpha;
        let y = color[(i + 2) % 3] / alpha;

        let alpha_i = self.find_alpha_node_index(alpha);
        let x_pos = x * (LUT_RESOLUTION - 1) as f32;
        let y_pos = y * (LUT_RESOLUTION - 1) as f32;
        let xi = (x_pos.floor() as usize).min(LUT_RESOLUTION - 2);
        let yi = (y_pos.floor() as usize).min(LUT_RESOLUTION - 2);

        let x_t = x_pos - xi as f32;
        let y_t = y_pos - yi as f32;

        let alpha0 = self.alpha_nodes[alpha_i] as f32;
        let alpha1 = self.alpha_nodes[alpha_i + 1] as f32;
        let alpha_t = (alpha - alpha0) / (alpha1 - alpha0);

        let lower = self
            .at(i, xi, yi, alpha_i)
            .lerp(self.at(i, xi + 1, yi, alpha_i), x_t)
            .lerp(
                self.at(i, xi, yi + 1, alpha_i)
                    .lerp(self.at(i, xi + 1, yi + 1, alpha_i), x_t),
                y_t,
            );
        let upper = self
            .at(i, xi, yi, alpha_i + 1)
            .lerp(self.at(i, xi + 1, yi, alpha_i + 1), x_t)
            .lerp(
                self.at(i, xi, yi + 1, alpha_i + 1)
                    .lerp(self.at(i, xi + 1, yi + 1, alpha_i + 1), x_t),
                y_t,
            );
        let theta = lower.lerp(upper, alpha_t);

        Spectrum::sigmoid(theta.x, theta.y, theta.z)
    }

    fn find_alpha_node_index(&self, alpha: f32) -> usize {
        let alpha = alpha as f64;
        if alpha <= self.alpha_nodes[0] {
            return 0;
        }
        if alpha >= self.alpha_nodes[LUT_RESOLUTION - 1] {
            return LUT_RESOLUTION - 2;
        }
        self.alpha_nodes.partition_point(|&node| node <= alpha) - 1
    }

    fn cell_index(i: usize, x: usize, y: usize) -> usize {
        (i * LUT_RESOLUTION + y) * LUT_RESOLUTION + x
    }

    fn at(&self, i: usize, x: usize, y: usize, alpha: usize) -> glam::Vec3 {
        self.data[Self::cell_index(i, x, y)][alpha]
    }

    fn load(&mut self, filename: &std::path::Path) -> std::io::Result<bool> {
        use bytemuck::cast_slice_mut;
        use std::{
            fs::File,
            io::{BufReader, Read},
        };

        let file = match File::open(filename) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(false);
            }
            Err(error) => return Err(error),
        };

        let mut file = BufReader::new(file);

        let mut header = [0u8; 8];
        if file.read_exact(&mut header).is_err() {
            return Ok(false);
        }

        let magic = u32::from_le_bytes(header[0..4].try_into().unwrap());
        let resolution = u32::from_le_bytes(header[4..8].try_into().unwrap());
        if magic != LUT_MAGIC || resolution != LUT_RESOLUTION as u32 {
            return Ok(false);
        }

        if file.read_exact(cast_slice_mut(&mut self.data)).is_err() {
            return Ok(false);
        }

        Ok(true)
    }

    fn save(&self, filename: &std::path::Path) -> std::io::Result<()> {
        use bytemuck::cast_slice;
        use std::{
            fs::File,
            io::{BufWriter, Write},
        };

        let mut file = BufWriter::new(File::create(filename)?);

        file.write_all(&LUT_MAGIC.to_le_bytes())?;
        file.write_all(&(LUT_RESOLUTION as u32).to_le_bytes())?;
        file.write_all(cast_slice(&self.data))?;

        Ok(())
    }

    fn make_rgb_target(
        alpha_nodes: &[f64; LUT_RESOLUTION],
        i: usize,
        x: f32,
        y: f32,
        alpha_i: usize,
    ) -> LinearRgb {
        let alpha = alpha_nodes[alpha_i] as f32;

        let mut rgb = [0.0; 3];

        rgb[i] = alpha;
        rgb[(i + 1) % 3] = alpha * x;
        rgb[(i + 2) % 3] = alpha * y;

        LinearRgb::new(rgb[0], rgb[1], rgb[2])
    }

    fn generate(&mut self) {
        profile!("Generate Color LUT, resolution = {}", LUT_RESOLUTION);

        const K_START: usize = LUT_RESOLUTION / 5;

        let context = OptimizeContext::new(self.color_space, self.illuminant);
        let options = OptimizeOptions::default();

        let progress = Progress::new(3 * LUT_RESOLUTION * LUT_RESOLUTION * LUT_RESOLUTION, 5);
        for i in 0..3 {
            let begin = i * LUT_RESOLUTION * LUT_RESOLUTION;
            let end = begin + LUT_RESOLUTION * LUT_RESOLUTION;
            let data = &mut self.data[begin..end];

            THREAD_POOL.parallel_for_2d(LUT_RESOLUTION, LUT_RESOLUTION, data, |xi, yi, cell| {
                let x = xi as f32 / (LUT_RESOLUTION - 1) as f32;
                let y = yi as f32 / (LUT_RESOLUTION - 1) as f32;

                let mut initial = Evaluation::eval(&context, glam::DVec3::ZERO);

                initial = optimize(
                    &context,
                    Self::make_rgb_target(&self.alpha_nodes, i, x, y, K_START),
                    initial,
                    options,
                );
                cell[K_START] = initial.theta.as_vec3();

                let mut up = initial;
                for k in K_START + 1..LUT_RESOLUTION {
                    up = optimize(
                        &context,
                        Self::make_rgb_target(&self.alpha_nodes, i, x, y, k),
                        up,
                        options,
                    );
                    cell[k] = up.theta.as_vec3();
                }

                let mut down = initial;
                for k in (0..K_START).rev() {
                    down = optimize(
                        &context,
                        Self::make_rgb_target(&self.alpha_nodes, i, x, y, k),
                        down,
                        options,
                    );
                    cell[k] = down.theta.as_vec3();
                }

                progress.update(LUT_RESOLUTION);
            });
        }
    }
}

pub static LUT_SRGB: LazyLock<ColorLut> =
    LazyLock::new(|| ColorLut::new("spectrums/ColorLUT_sRGB.lut", &SRGB, &CIE_STD_ILLUMNT_D65));
