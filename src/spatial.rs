use std::f64::consts::FRAC_PI_2;
use crate::formula::calculate_culling_attenuation;

/// Total channels in 7.1.4 layout.
pub const NUM_CHANNELS: usize = 12;

/// Channel indices:
/// 0: L (Left)
/// 1: R (Right)
/// 2: C (Center)
/// 3: LFE (Low Frequency Effects)
/// 4: Ls (Left Surround)
/// 5: Rs (Right Surround)
/// 6: Lb (Left Back)
/// 7: Rb (Right Back)
/// 8: Ltf (Left Top Front)
/// 9: Rtf (Right Top Front)
/// 10: Ltr (Left Top Rear)
/// 11: Rtr (Right Top Rear)
pub const CHANNEL_NAMES: [&str; NUM_CHANNELS] = [
    "L", "R", "C", "LFE", "Ls", "Rs", "Lb", "Rb", "Ltf", "Rtf", "Ltr", "Rtr"
];

/// Computes the 12-channel gains for an object at position `(x, y, z)`.
/// Coordinates:
/// - `x`: Left/Right axis, range [-1.0, 1.0] (negative is Left, positive is Right)
/// - `y`: Back/Front axis, range [-1.0, 1.0] (negative is Back, positive is Front)
/// - `z`: Height axis, range [0.0, 1.0] (0.0 is ear-level, 1.0 is ceiling)
///
/// Handles spatial culling by applying attenuation if the coordinates drift out of bounds.
pub fn pan_3d(x: f64, y: f64, z: f64) -> [f64; NUM_CHANNELS] {
    let mut gains = [0.0; NUM_CHANNELS];

    // Calculate spatial culling attenuation
    let attenuation = calculate_culling_attenuation(x, y, z);
    if attenuation <= 0.0 {
        return gains;
    }

    // Clamp coordinates to the physical boundaries for panning geometry
    let xc = x.clamp(-1.0, 1.0);
    let yc = y.clamp(-1.0, 1.0);
    let zc = z.clamp(0.0, 1.0);

    // 1. Split power between the Bed layer (z=0) and Height layer (z=1) using Constant Power
    let g_height = (zc * FRAC_PI_2).sin();
    let g_bed = (zc * FRAC_PI_2).cos();

    // 2. Pan on the Height layer (4 speakers: Ltf, Rtf, Ltr, Rtr)
    // Left/Right split for heights
    let w_x_left = (((xc + 1.0) * std::f64::consts::FRAC_PI_4).cos()).abs();
    let w_x_right = (((xc + 1.0) * std::f64::consts::FRAC_PI_4).sin()).abs();
    // Back/Front split for heights
    let w_y_back = (((yc + 1.0) * std::f64::consts::FRAC_PI_4).cos()).abs();
    let w_y_front = (((yc + 1.0) * std::f64::consts::FRAC_PI_4).sin()).abs();

    gains[8] = g_height * w_x_left * w_y_front;   // Ltf
    gains[9] = g_height * w_x_right * w_y_front;  // Rtf
    gains[10] = g_height * w_x_left * w_y_back;   // Ltr
    gains[11] = g_height * w_x_right * w_y_back;  // Rtr

    // 3. Pan on the Bed layer (7 active speakers, LFE is silent/0.0)
    let r = (xc * xc + yc * yc).sqrt().min(1.0);
    let theta = if xc == 0.0 && yc == 0.0 {
        0.0
    } else {
        xc.atan2(yc).to_degrees()
    };

    // Find localized pairwise gains on the speaker ring
    let (ch1, ch2, gp1, gp2) = bed_pairwise_panning(theta);

    let mut bed_gains = [0.0; 8];
    bed_gains[ch1] = gp1;
    bed_gains[ch2] = gp2;

    // Blend in the diffuse component based on radius (center bleed)
    // At r=0, sound is equally split. At r=1, sound is highly localized.
    let diffuse_g = (1.0 - r) * (1.0 / 7.0_f64.sqrt());
    let mut blended_bed = [0.0; 8];
    for i in 0..8 {
        if i == 3 {
            continue; // Skip LFE
        }
        blended_bed[i] = r * bed_gains[i] + diffuse_g;
    }

    // Normalize panned bed gains to preserve constant power (sum of squares = 1.0)
    let mut sum_sq = 0.0;
    for i in 0..8 {
        if i == 3 {
            continue;
        }
        sum_sq += blended_bed[i] * blended_bed[i];
    }

    let norm_factor = if sum_sq > 0.0 { 1.0 / sum_sq.sqrt() } else { 1.0 };

    for i in 0..8 {
        if i == 3 {
            gains[i] = 0.0; // Keep LFE silent during panning
        } else {
            gains[i] = g_bed * blended_bed[i] * norm_factor;
        }
    }

    // 4. Scale all channels by the spatial culling attenuation
    for i in 0..NUM_CHANNELS {
        gains[i] *= attenuation;
    }

    gains
}

/// Computes pairwise panning gains for 7.1 bed speakers.
/// Input `theta` is in degrees [-180.0, 180.0].
/// Returns `(channel_index_1, channel_index_2, gain_1, gain_2)`
fn bed_pairwise_panning(theta: f64) -> (usize, usize, f64, f64) {
    if theta >= -150.0 && theta < -110.0 {
        // Lb (Left Back: -150) to Ls (Left Surround: -110)
        let u = (theta - (-150.0)) / 40.0;
        (6, 4, (u * FRAC_PI_2).cos(), (u * FRAC_PI_2).sin())
    } else if theta >= -110.0 && theta < -30.0 {
        // Ls (Left Surround: -110) to L (Left: -30)
        let u = (theta - (-110.0)) / 80.0;
        (4, 0, (u * FRAC_PI_2).cos(), (u * FRAC_PI_2).sin())
    } else if theta >= -30.0 && theta < 0.0 {
        // L (Left: -30) to C (Center: 0)
        let u = (theta - (-30.0)) / 30.0;
        (0, 2, (u * FRAC_PI_2).cos(), (u * FRAC_PI_2).sin())
    } else if theta >= 0.0 && theta < 30.0 {
        // C (Center: 0) to R (Right: 30)
        let u = (theta - 0.0) / 30.0;
        (2, 1, (u * FRAC_PI_2).cos(), (u * FRAC_PI_2).sin())
    } else if theta >= 30.0 && theta < 110.0 {
        // R (Right: 30) to Rs (Right Surround: 110)
        let u = (theta - 30.0) / 80.0;
        (1, 5, (u * FRAC_PI_2).cos(), (u * FRAC_PI_2).sin())
    } else if theta >= 110.0 && theta < 150.0 {
        // Rs (Right Surround: 110) to Rb (Right Back: 150)
        let u = (theta - 110.0) / 40.0;
        (5, 7, (u * FRAC_PI_2).cos(), (u * FRAC_PI_2).sin())
    } else {
        // Rb (Right Back: 150) to Lb (Left Back: -150)
        // Angle wraps around 180 degrees.
        let norm_theta = if theta < 0.0 { theta + 360.0 } else { theta };
        // norm_theta is in [150.0, 210.0]
        let u = (norm_theta - 150.0) / 60.0;
        (7, 6, (u * FRAC_PI_2).cos(), (u * FRAC_PI_2).sin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sum_squares_excluding_lfe(gains: &[f64; 12]) -> f64 {
        gains.iter().enumerate().filter(|&(i, _)| i != 3).map(|(_, &g)| g * g).sum()
    }

    #[test]
    fn test_constant_power_normalization() {
        // Panning at various positions should always have a sum of squares equal to 1.0 (before attenuation)
        let positions = [
            (0.0, 0.0, 0.0),     // Center
            (1.0, 0.0, 0.0),     // Hard Right
            (-0.5, 0.5, 0.5),    // Arbitrary 3D point inside
            (0.0, -1.0, 1.0),    // Back height
            (0.7, -0.7, 0.25),   // Mixed
        ];

        for (x, y, z) in positions {
            let gains = pan_3d(x, y, z);
            let pwr = sum_squares_excluding_lfe(&gains);
            assert!((pwr - 1.0).abs() < 1e-9, "Power at ({}, {}, {}) is {}, expected 1.0", x, y, z, pwr);
        }
    }

    #[test]
    fn test_hard_panning_points() {
        // At Center (0.0, 1.0, 0.0), all sound should be routed to channel 2 (C)
        let gains_center = pan_3d(0.0, 1.0, 0.0);
        assert!((gains_center[2] - 1.0).abs() < 1e-9);
        assert!(gains_center[0] < 1e-9);
        assert!(gains_center[1] < 1e-9);

        // At Left (sin(-30), cos(-30), 0) or simply hard azimuth -30 deg
        // Azimuth -30 is X = sin(-30) = -0.5, Y = cos(-30) = 0.8660254
        let gains_l = pan_3d(-0.5, 0.8660254, 0.0);
        assert!((gains_l[0] - 1.0).abs() < 1e-5); // Should be 100% Left

        // Height layer hard corners
        // At Front Left Top (x = -1.0, y = 1.0, z = 1.0) => 100% Ltf (channel 8)
        let gains_ltf = pan_3d(-1.0, 1.0, 1.0);
        assert!((gains_ltf[8] - 1.0).abs() < 1e-9);
        assert!(gains_ltf[9] < 1e-9);
        assert!(gains_ltf[10] < 1e-9);
        assert!(gains_ltf[11] < 1e-9);
    }

    #[test]
    fn test_diffuse_field_panning() {
        // At coordinate (0.0, 0.0, 0.0), the sound should be perfectly diffuse
        // which means all bed speakers (except LFE) have equal gains.
        let gains_diffuse = pan_3d(0.0, 0.0, 0.0);
        let expected_gain = 1.0 / 7.0_f64.sqrt();
        for idx in [0, 1, 2, 4, 5, 6, 7] {
            assert!((gains_diffuse[idx] - expected_gain).abs() < 1e-9);
        }
        assert_eq!(gains_diffuse[3], 0.0); // LFE is silent
    }

    #[test]
    fn test_panning_with_attenuation() {
        // An object far out of bounds (x = 3.0) should be completely silent due to culling
        let gains_silent = pan_3d(3.0, 0.0, 0.0);
        for g in gains_silent {
            assert_eq!(g, 0.0);
        }

        // An object partially out of bounds (x = 1.5, distance = 0.5) should have its power scaled by attenuation (0.5)
        // which means its sum of squares should be (0.5)^2 = 0.25
        let gains_attenuated = pan_3d(1.5, 0.0, 0.0);
        let pwr = sum_squares_excluding_lfe(&gains_attenuated);
        assert!((pwr - 0.25).abs() < 1e-9);
    }
}
