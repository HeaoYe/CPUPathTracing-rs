use crate::{
    color::{ColorSpace, LinearRgb, SRGB, Xyz},
    spectrum::{CIE_STD_ILLUMNT_D65, LAMBDA_MAX, LAMBDA_MIN, Spectrum, X_CMF, Y_CMF, Z_CMF},
};

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

pub fn test_optimize() {
    let context = OptimizeContext::new(&SRGB, &CIE_STD_ILLUMNT_D65);

    let rgb_target = LinearRgb::new(0.2, 0.6, 0.3);

    let initial = Evaluation::eval(&context, glam::DVec3::ZERO);
    let result = optimize(&context, rgb_target, initial, Default::default());
    let r = Spectrum::sigmoid(
        result.theta.x as f32,
        result.theta.y as f32,
        result.theta.z as f32,
    );
    let s = Spectrum::analytic(
        move |lambda: f32| {
            context.k as f32 * context.illuminant_white.eval(lambda) * r.eval(lambda)
        },
        LAMBDA_MIN as f32,
        LAMBDA_MAX as f32,
    );
    let xyz_pred = Xyz::from_spectrum(&s);
    let rgb_pred = SRGB.rgb_from_xyz(xyz_pred);

    println!("Target: {:?}", rgb_target);
    println!(" Pred : {:?}", rgb_pred);
}
