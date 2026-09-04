use std::array;

pub const LAMBDA_MIN: u32 = 360;
pub const LAMBDA_MAX: u32 = 830;

pub const WAVELENGTH_SAMPLE_COUNT: usize = 4;

pub struct WavelengthSample {
    lambdas: [f32; WAVELENGTH_SAMPLE_COUNT],
    pdfs: [f32; WAVELENGTH_SAMPLE_COUNT],
}

impl WavelengthSample {
    pub fn uniform(u: f32) -> Self {
        let delta = (LAMBDA_MAX - LAMBDA_MIN) as f32 / WAVELENGTH_SAMPLE_COUNT as f32;
        let lambda_start = LAMBDA_MIN as f32 * (1.0 - u) + LAMBDA_MAX as f32 * u;
        let lambdas = array::from_fn(|i| {
            let lambda = lambda_start + i as f32 * delta;
            if lambda > LAMBDA_MAX as f32 {
                LAMBDA_MIN as f32 + lambda - LAMBDA_MAX as f32
            } else {
                lambda
            }
        });
        let pdfs = [1.0 / (LAMBDA_MAX - LAMBDA_MIN) as f32; WAVELENGTH_SAMPLE_COUNT];
        Self { lambdas, pdfs }
    }

    #[allow(clippy::excessive_precision)]
    pub fn importance(u: f32) -> Self {
        let lambdas = array::from_fn(|i| {
            let mut ui = u + i as f32 / WAVELENGTH_SAMPLE_COUNT as f32;
            if ui >= 1.0 {
                ui -= 1.0;
            }
            538.0 + 138.88889 * (1.827502 * ui - 0.85691065).atanh()
        });

        let pdfs = lambdas.map(|lambda| {
            let cosh = (0.0072 * (lambda - 538.0)).cosh();
            0.0039398042 / cosh.powi(2)
        });

        Self { lambdas, pdfs }
    }

    pub fn lambda(&self, index: usize) -> f32 {
        self.lambdas[index]
    }

    pub fn pdf(&self, index: usize) -> f32 {
        self.pdfs[index]
    }

    pub fn lambdas(&self) -> &[f32; WAVELENGTH_SAMPLE_COUNT] {
        &self.lambdas
    }

    pub fn pdfs(&self) -> &[f32; WAVELENGTH_SAMPLE_COUNT] {
        &self.pdfs
    }

    pub fn terminate_secondary(&mut self) {
        self.pdfs[0] /= WAVELENGTH_SAMPLE_COUNT as f32;
        self.pdfs[1..].fill(0.0);
    }
}
