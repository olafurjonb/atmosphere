use meval::Expr;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum FormulaError {
    ParseError { formula: String, error_msg: String },
    BindError { formula: String, error_msg: String },
}

impl fmt::Display for FormulaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError { formula, error_msg } => {
                write!(f, "Failed to parse formula '{}': {}", formula, error_msg)
            }
            Self::BindError { formula, error_msg } => {
                write!(f, "Failed to bind 't' in formula '{}': {}", formula, error_msg)
            }
        }
    }
}

impl Error for FormulaError {}

pub struct TrajectoryEvaluator {
    x_func: Box<dyn Fn(f64) -> f64>,
    y_func: Box<dyn Fn(f64) -> f64>,
    z_func: Box<dyn Fn(f64) -> f64>,
    vol_func: Box<dyn Fn(f64) -> f64>,
    start_x: f64,
    start_y: f64,
    start_z: f64,
}

impl TrajectoryEvaluator {
    /// Compiles spatial trajectory formulas and volume formulas.
    /// Default values are used if formulas are omitted.
    pub fn new(
        start_x: f64,
        start_y: f64,
        start_z: f64,
        xformula: Option<&str>,
        yformula: Option<&str>,
        zformula: Option<&str>,
        volume: Option<&str>,
    ) -> Result<Self, FormulaError> {
        let x_str = xformula.unwrap_or("0.0");
        let y_str = yformula.unwrap_or("0.0");
        let z_str = zformula.unwrap_or("0.0");
        let vol_str = volume.unwrap_or("1.0");

        let x_expr: Expr = x_str.parse().map_err(|e| FormulaError::ParseError {
            formula: x_str.to_string(),
            error_msg: format!("{:?}", e),
        })?;
        let x_func = x_expr.bind("t").map_err(|e| FormulaError::BindError {
            formula: x_str.to_string(),
            error_msg: format!("{:?}", e),
        })?;

        let y_expr: Expr = y_str.parse().map_err(|e| FormulaError::ParseError {
            formula: y_str.to_string(),
            error_msg: format!("{:?}", e),
        })?;
        let y_func = y_expr.bind("t").map_err(|e| FormulaError::BindError {
            formula: y_str.to_string(),
            error_msg: format!("{:?}", e),
        })?;

        let z_expr: Expr = z_str.parse().map_err(|e| FormulaError::ParseError {
            formula: z_str.to_string(),
            error_msg: format!("{:?}", e),
        })?;
        let z_func = z_expr.bind("t").map_err(|e| FormulaError::BindError {
            formula: z_str.to_string(),
            error_msg: format!("{:?}", e),
        })?;

        let vol_expr: Expr = vol_str.parse().map_err(|e| FormulaError::ParseError {
            formula: vol_str.to_string(),
            error_msg: format!("{:?}", e),
        })?;
        let vol_func = vol_expr.bind("t").map_err(|e| FormulaError::BindError {
            formula: vol_str.to_string(),
            error_msg: format!("{:?}", e),
        })?;

        Ok(Self {
            x_func: Box::new(x_func),
            y_func: Box::new(y_func),
            z_func: Box::new(z_func),
            vol_func: Box::new(vol_func),
            start_x,
            start_y,
            start_z,
        })
    }

    /// Evaluates the position and basic volume envelope at time `t`.
    /// Returns a tuple of `(x, y, z, volume)`.
    /// Trajectory positions are relative to the starting coordinates:
    /// `x(t) = start_x + xformula(t)`, etc.
    pub fn evaluate(&self, t: f64) -> (f64, f64, f64, f64) {
        let x = self.start_x + (self.x_func)(t);
        let y = self.start_y + (self.y_func)(t);
        let z = self.start_z + (self.z_func)(t);
        let vol = (self.vol_func)(t);
        (x, y, z, vol)
    }
}

/// Calculates the spatial culling attenuation factor for a given point `(x, y, z)`.
/// Valid boundaries:
/// - X in [-1.0, 1.0]
/// - Y in [-1.0, 1.0]
/// - Z in [0.0, 1.0]
///
/// If inside bounds, attenuation is 1.0 (no change).
/// If outside, attenuation fades out linearly to 0 over 1.0 unit of distance.
pub fn calculate_culling_attenuation(x: f64, y: f64, z: f64) -> f64 {
    let xc = x.clamp(-1.0, 1.0);
    let yc = y.clamp(-1.0, 1.0);
    let zc = z.clamp(0.0, 1.0);

    let dx = x - xc;
    let dy = y - yc;
    let dz = z - zc;

    let dist_out = (dx * dx + dy * dy + dz * dz).sqrt();
    if dist_out <= 0.0 {
        1.0
    } else {
        (1.0 - dist_out).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_defaults() {
        let eval = TrajectoryEvaluator::new(0.5, -0.5, 0.2, None, None, None, None).unwrap();
        let (x, y, z, vol) = eval.evaluate(10.0);
        assert_eq!(x, 0.5);
        assert_eq!(y, -0.5);
        assert_eq!(z, 0.2);
        assert_eq!(vol, 1.0);
    }

    #[test]
    fn test_evaluator_formulas() {
        let eval = TrajectoryEvaluator::new(
            0.0,
            0.0,
            0.0,
            Some("sin(t * pi)"),
            Some("cos(t * pi)"),
            Some("t * 0.1"),
            Some("1.0 - t * 0.1"),
        )
        .unwrap();

        // At t = 0.5:
        // sin(0.5 * pi) = 1.0
        // cos(0.5 * pi) = 0.0 (approx)
        // z = 0.05
        // vol = 0.95
        let (x, y, z, vol) = eval.evaluate(0.5);
        assert!((x - 1.0).abs() < 1e-9);
        assert!(y.abs() < 1e-9);
        assert!((z - 0.05).abs() < 1e-9);
        assert!((vol - 0.95).abs() < 1e-9);
    }

    #[test]
    fn test_culling_attenuation() {
        // Inside bounds
        assert_eq!(calculate_culling_attenuation(0.0, 0.0, 0.5), 1.0);
        assert_eq!(calculate_culling_attenuation(1.0, -1.0, 0.0), 1.0);

        // Outside X boundary (x = 1.5, yc = 0.0, zc = 0.5) => distance = 0.5
        let att1 = calculate_culling_attenuation(1.5, 0.0, 0.5);
        assert_eq!(att1, 0.5);

        // Far outside => completely silent
        let att2 = calculate_culling_attenuation(3.0, 0.0, 0.5);
        assert_eq!(att2, 0.0);

        // Outside Z boundary (x = 0.0, y = 0.0, z = -0.5) => distance = 0.5
        let att3 = calculate_culling_attenuation(0.0, 0.0, -0.5);
        assert_eq!(att3, 0.5);
    }
}
