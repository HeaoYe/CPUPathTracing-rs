use super::{Chromaticity, Xyz, encoded_rgb::TransferFunction};
use crate::{
    color::{EncodedRgb, LinearRgb},
    spectrum::{CIE_STD_ILLUMNT_D65, Spectrum},
};
use std::sync::LazyLock;

pub struct ColorSpace {
    name: &'static str,

    red: Chromaticity,
    green: Chromaticity,
    blue: Chromaticity,
    white: Chromaticity,

    rgb_from_xyz: glam::Mat3A,
    xyz_from_rgb: glam::Mat3A,

    transfer_function: TransferFunction,
}

impl ColorSpace {
    pub fn new(
        name: &'static str,
        red: Chromaticity,
        green: Chromaticity,
        blue: Chromaticity,
        white: Chromaticity,
        transfer_function: TransferFunction,
    ) -> Self {
        let r = Xyz::from_chromaticity(red, 1.0).as_vec3();
        let g = Xyz::from_chromaticity(green, 1.0).as_vec3();
        let b = Xyz::from_chromaticity(blue, 1.0).as_vec3();
        let w = Xyz::from_chromaticity(white, 1.0).as_vec3();
        let k: glam::prelude::Vec3 = glam::Mat3::from_cols(r, g, b).inverse() * w;
        let xyz_from_rgb = glam::Mat3::from_cols(k.x * r, k.y * g, k.z * b).into();
        Self {
            name,
            red,
            green,
            blue,
            white,
            xyz_from_rgb,
            rgb_from_xyz: xyz_from_rgb.inverse(),
            transfer_function,
        }
    }

    pub fn from_illuminant<'a>(
        name: &'static str,
        red: Chromaticity,
        green: Chromaticity,
        blue: Chromaticity,
        illuminant_white: &'a Spectrum<'a>,
        transfer_function: TransferFunction,
    ) -> Self {
        let white = Xyz::from_spectrum(illuminant_white);
        let white = white / white.y();
        Self::new(name, red, green, blue, white.into(), transfer_function)
    }

    pub fn rgb_from_xyz(&self, xyz: Xyz) -> LinearRgb {
        (self.rgb_from_xyz * xyz.as_vec3()).into()
    }

    pub fn xyz_from_rgb(&self, rgb: LinearRgb) -> Xyz {
        (self.xyz_from_rgb * rgb.as_vec3()).into()
    }

    pub fn decode(&self, encoded_rgb: EncodedRgb) -> LinearRgb {
        LinearRgb::new(
            self.transfer_function.decode(encoded_rgb.r()),
            self.transfer_function.decode(encoded_rgb.g()),
            self.transfer_function.decode(encoded_rgb.b()),
        )
    }

    pub fn encode(&self, linear_rgb: LinearRgb) -> EncodedRgb {
        EncodedRgb::new(
            self.transfer_function.encode(linear_rgb.r()),
            self.transfer_function.encode(linear_rgb.g()),
            self.transfer_function.encode(linear_rgb.b()),
        )
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn red(&self) -> Chromaticity {
        self.red
    }

    pub fn green(&self) -> Chromaticity {
        self.green
    }

    pub fn blue(&self) -> Chromaticity {
        self.blue
    }

    pub fn white(&self) -> Chromaticity {
        self.white
    }
}

pub static SRGB: LazyLock<ColorSpace> = LazyLock::new(|| {
    ColorSpace::from_illuminant(
        "sRGB",
        Chromaticity::new(0.64, 0.33),
        Chromaticity::new(0.30, 0.60),
        Chromaticity::new(0.15, 0.06),
        &CIE_STD_ILLUMNT_D65,
        TransferFunction::new(
            |l| {
                if l <= 0.0031308 {
                    12.92 * l
                } else {
                    1.055 * l.powf(1.0 / 2.4) - 0.055
                }
            },
            |v| {
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            },
        ),
    )
});

pub static DCI_P3: LazyLock<ColorSpace> = LazyLock::new(|| {
    ColorSpace::new(
        "DCI-P3",
        Chromaticity::new(0.680, 0.320),
        Chromaticity::new(0.265, 0.690),
        Chromaticity::new(0.150, 0.060),
        Chromaticity::new(0.314, 0.351),
        TransferFunction::new(|l| l.powf(1.0 / 2.6), |v| v.powf(2.6)),
    )
});
