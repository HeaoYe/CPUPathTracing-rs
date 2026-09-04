#[derive(Debug, Default, Clone, Copy)]
pub struct EncodedRgb {
    data: glam::Vec3,
}

impl EncodedRgb {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self {
            data: glam::vec3(r, g, b),
        }
    }

    pub fn from_vec3(data: glam::Vec3) -> Self {
        Self { data }
    }

    pub fn from_quantized(r: u32, g: u32, b: u32, bit_depth: u32) -> Self {
        let maximum = quantized_maximum(bit_depth) as f32;
        Self::new(r as f32 / maximum, g as f32 / maximum, b as f32 / maximum)
    }

    pub fn to_quantized(&self, bit_depth: u32) -> glam::UVec3 {
        let maximum = quantized_maximum(bit_depth) as f32;
        let value = self.data * maximum;

        glam::UVec3::new(
            value.x.round().clamp(0.0, maximum) as u32,
            value.y.round().clamp(0.0, maximum) as u32,
            value.z.round().clamp(0.0, maximum) as u32,
        )
    }

    pub fn generate_heatmap(t: f32) -> Self {
        const PALETTE: [[u8; 3]; 25] = [
            [68, 1, 84],
            [71, 17, 100],
            [72, 31, 112],
            [71, 45, 123],
            [68, 58, 131],
            [64, 70, 136],
            [59, 82, 139],
            [54, 93, 141],
            [49, 104, 142],
            [44, 114, 142],
            [40, 124, 142],
            [36, 134, 142],
            [33, 144, 140],
            [31, 154, 138],
            [32, 164, 134],
            [39, 173, 129],
            [53, 183, 121],
            [71, 193, 110],
            [93, 200, 99],
            [117, 208, 84],
            [143, 215, 68],
            [170, 220, 50],
            [199, 224, 32],
            [227, 228, 24],
            [253, 231, 37],
        ];

        if t < 0.0 || t >= 1.0 {
            return Self::from_quantized(255, 0, 0, 8);
        }
        let idx_float = t * (PALETTE.len() - 1) as f32;
        let idx = idx_float.floor() as usize;
        let s = idx_float.fract();
        EncodedRgb::from_quantized(
            (PALETTE[idx][0] as f32 * (1.0 - s) + PALETTE[idx + 1][0] as f32 * s) as u32,
            (PALETTE[idx][1] as f32 * (1.0 - s) + PALETTE[idx + 1][1] as f32 * s) as u32,
            (PALETTE[idx][2] as f32 * (1.0 - s) + PALETTE[idx + 1][2] as f32 * s) as u32,
            8,
        )
    }

    pub fn r(&self) -> f32 {
        self.data.x
    }

    pub fn g(&self) -> f32 {
        self.data.y
    }

    pub fn b(&self) -> f32 {
        self.data.z
    }

    pub fn as_vec3(&self) -> glam::Vec3 {
        self.data
    }

    pub fn as_dvec3(&self) -> glam::DVec3 {
        self.data.as_dvec3()
    }
}

fn quantized_maximum(bit_depth: u32) -> u32 {
    assert!((1..=32).contains(&bit_depth));

    if bit_depth == 32 {
        u32::MAX
    } else {
        (1u32 << bit_depth) - 1
    }
}

#[derive(Clone, Copy)]
pub struct TransferFunction {
    encode: fn(f32) -> f32,
    decode: fn(f32) -> f32,
}

impl TransferFunction {
    pub fn new(encode: fn(f32) -> f32, decode: fn(f32) -> f32) -> Self {
        Self { encode, decode }
    }

    pub fn encode(&self, value: f32) -> f32 {
        (self.encode)(value)
    }

    pub fn decode(&self, value: f32) -> f32 {
        (self.decode)(value)
    }
}

impl Default for TransferFunction {
    fn default() -> Self {
        fn identity(value: f32) -> f32 {
            value
        }

        Self {
            encode: identity,
            decode: identity,
        }
    }
}
