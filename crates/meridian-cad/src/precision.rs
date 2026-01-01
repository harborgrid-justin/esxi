//! High-precision arithmetic for engineering accuracy
//!
//! This module provides high-precision numeric types for CAD calculations,
//! ensuring accuracy for engineering and architectural applications.

use num_traits::{Float, FromPrimitive, Num, NumCast, ToPrimitive, Zero};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::{CadError, CadResult};

/// 128-bit decimal type for high-precision calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Decimal128(Decimal);

impl Decimal128 {
    /// Create a new decimal from integer
    pub fn new(value: i64) -> Self {
        Self(Decimal::from(value))
    }

    /// Create from f64 (may lose precision)
    pub fn from_f64(value: f64) -> CadResult<Self> {
        Decimal::from_f64_retain(value)
            .map(Self)
            .ok_or_else(|| CadError::PrecisionError(format!("Cannot convert {} to Decimal", value)))
    }

    /// Create from string
    pub fn from_str(s: &str) -> CadResult<Self> {
        s.parse::<Decimal>()
            .map(Self)
            .map_err(|e| CadError::PrecisionError(format!("Parse error: {}", e)))
    }

    /// Convert to f64 (may lose precision)
    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().unwrap_or(0.0)
    }

    /// Get zero value
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    /// Get one value
    pub fn one() -> Self {
        Self(Decimal::ONE)
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Absolute value
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    /// Round to decimal places
    pub fn round(&self, decimal_places: u32) -> Self {
        Self(self.0.round_dp(decimal_places))
    }

    /// Floor
    pub fn floor(&self) -> Self {
        Self(self.0.floor())
    }

    /// Ceiling
    pub fn ceil(&self) -> Self {
        Self(self.0.ceil())
    }

    /// Power (limited to integer exponents)
    pub fn powi(&self, exp: i64) -> Self {
        if exp < 0 {
            let pos_result = self.powi(-exp);
            return Self(Decimal::ONE / pos_result.0);
        }

        let mut result = Decimal::ONE;
        let mut base = self.0;
        let mut exponent = exp;

        while exponent > 0 {
            if exponent % 2 == 1 {
                result = result * base;
            }
            base = base * base;
            exponent /= 2;
        }

        Self(result)
    }

    /// Square root (Newton-Raphson approximation)
    pub fn sqrt(&self) -> CadResult<Self> {
        if self.0.is_sign_negative() {
            return Err(CadError::PrecisionError("Cannot take square root of negative number".into()));
        }

        // Newton-Raphson: x_{n+1} = (x_n + a/x_n) / 2
        let two = Decimal::from(2);
        let mut x = self.0;
        let mut last_x;

        for _ in 0..20 {
            // Max iterations
            last_x = x;
            x = (x + self.0 / x) / two;

            // Check convergence
            let diff = (x - last_x).abs();
            if diff < Decimal::new(1, 10) {
                // 1e-10
                break;
            }
        }

        Ok(Self(x))
    }

    /// Minimum of two values
    pub fn min(self, other: Self) -> Self {
        if self.0 < other.0 {
            self
        } else {
            other
        }
    }

    /// Maximum of two values
    pub fn max(self, other: Self) -> Self {
        if self.0 > other.0 {
            self
        } else {
            other
        }
    }
}

impl fmt::Display for Decimal128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for Decimal128 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Decimal128 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for Decimal128 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for Decimal128 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Neg for Decimal128 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl From<i64> for Decimal128 {
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl From<Decimal> for Decimal128 {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

/// Engineering precision manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringPrecision {
    /// Number of decimal places for display
    pub display_precision: u32,

    /// Tolerance for equality comparisons
    pub tolerance: f64,

    /// Rounding mode
    pub rounding_mode: RoundingMode,

    /// Units for display
    pub units: PrecisionUnits,
}

impl Default for EngineeringPrecision {
    fn default() -> Self {
        Self {
            display_precision: 3,
            tolerance: 1e-6,
            rounding_mode: RoundingMode::HalfUp,
            units: PrecisionUnits::Millimeters,
        }
    }
}

impl EngineeringPrecision {
    /// Create precision settings for architectural drawings
    pub fn architectural() -> Self {
        Self {
            display_precision: 2,
            tolerance: 1e-3,
            rounding_mode: RoundingMode::HalfUp,
            units: PrecisionUnits::Millimeters,
        }
    }

    /// Create precision settings for mechanical engineering
    pub fn mechanical() -> Self {
        Self {
            display_precision: 4,
            tolerance: 1e-5,
            rounding_mode: RoundingMode::HalfEven,
            units: PrecisionUnits::Millimeters,
        }
    }

    /// Create precision settings for scientific calculations
    pub fn scientific() -> Self {
        Self {
            display_precision: 6,
            tolerance: 1e-9,
            rounding_mode: RoundingMode::HalfEven,
            units: PrecisionUnits::Meters,
        }
    }

    /// Check if two values are equal within tolerance
    pub fn equals(&self, a: f64, b: f64) -> bool {
        (a - b).abs() < self.tolerance
    }

    /// Round value according to precision settings
    pub fn round(&self, value: f64) -> f64 {
        let multiplier = 10_f64.powi(self.display_precision as i32);
        match self.rounding_mode {
            RoundingMode::HalfUp => (value * multiplier + 0.5).floor() / multiplier,
            RoundingMode::HalfDown => (value * multiplier - 0.5).ceil() / multiplier,
            RoundingMode::HalfEven => {
                let scaled = value * multiplier;
                let floored = scaled.floor();
                let fraction = scaled - floored;

                if fraction < 0.5 {
                    floored / multiplier
                } else if fraction > 0.5 {
                    (floored + 1.0) / multiplier
                } else {
                    // Round to nearest even
                    if floored as i64 % 2 == 0 {
                        floored / multiplier
                    } else {
                        (floored + 1.0) / multiplier
                    }
                }
            }
            RoundingMode::Up => (value * multiplier).ceil() / multiplier,
            RoundingMode::Down => (value * multiplier).floor() / multiplier,
        }
    }

    /// Format value for display
    pub fn format(&self, value: f64) -> String {
        let rounded = self.round(value);
        format!("{:.prec$}", rounded, prec = self.display_precision as usize)
    }

    /// Format value with units
    pub fn format_with_units(&self, value: f64) -> String {
        format!("{} {}", self.format(value), self.units.symbol())
    }

    /// Convert value from one unit to another
    pub fn convert(&self, value: f64, from: PrecisionUnits, to: PrecisionUnits) -> f64 {
        // Convert to millimeters first
        let mm = value * from.to_mm();
        // Then convert to target unit
        mm / to.to_mm()
    }

    /// Validate precision settings
    pub fn validate(&self) -> CadResult<()> {
        if self.display_precision > 15 {
            return Err(CadError::PrecisionError(
                "Display precision cannot exceed 15 decimal places".into(),
            ));
        }

        if self.tolerance <= 0.0 {
            return Err(CadError::PrecisionError(
                "Tolerance must be positive".into(),
            ));
        }

        Ok(())
    }
}

/// Rounding modes for engineering calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundingMode {
    /// Round half up (standard rounding)
    HalfUp,

    /// Round half down
    HalfDown,

    /// Round half to even (banker's rounding)
    HalfEven,

    /// Always round up
    Up,

    /// Always round down
    Down,
}

/// Precision units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionUnits {
    Millimeters,
    Centimeters,
    Meters,
    Inches,
    Feet,
    Micrometers,
    Nanometers,
}

impl PrecisionUnits {
    /// Get conversion factor to millimeters
    pub fn to_mm(&self) -> f64 {
        match self {
            PrecisionUnits::Millimeters => 1.0,
            PrecisionUnits::Centimeters => 10.0,
            PrecisionUnits::Meters => 1000.0,
            PrecisionUnits::Inches => 25.4,
            PrecisionUnits::Feet => 304.8,
            PrecisionUnits::Micrometers => 0.001,
            PrecisionUnits::Nanometers => 0.000001,
        }
    }

    /// Get unit symbol
    pub fn symbol(&self) -> &str {
        match self {
            PrecisionUnits::Millimeters => "mm",
            PrecisionUnits::Centimeters => "cm",
            PrecisionUnits::Meters => "m",
            PrecisionUnits::Inches => "in",
            PrecisionUnits::Feet => "ft",
            PrecisionUnits::Micrometers => "μm",
            PrecisionUnits::Nanometers => "nm",
        }
    }

    /// Get full name
    pub fn name(&self) -> &str {
        match self {
            PrecisionUnits::Millimeters => "millimeters",
            PrecisionUnits::Centimeters => "centimeters",
            PrecisionUnits::Meters => "meters",
            PrecisionUnits::Inches => "inches",
            PrecisionUnits::Feet => "feet",
            PrecisionUnits::Micrometers => "micrometers",
            PrecisionUnits::Nanometers => "nanometers",
        }
    }
}

/// High-precision 2D point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrecisionPoint {
    pub x: Decimal128,
    pub y: Decimal128,
}

impl PrecisionPoint {
    /// Create a new precision point
    pub fn new(x: Decimal128, y: Decimal128) -> Self {
        Self { x, y }
    }

    /// Create from f64 coordinates
    pub fn from_f64(x: f64, y: f64) -> CadResult<Self> {
        Ok(Self {
            x: Decimal128::from_f64(x)?,
            y: Decimal128::from_f64(y)?,
        })
    }

    /// Convert to f64 coordinates
    pub fn to_f64(&self) -> (f64, f64) {
        (self.x.to_f64(), self.y.to_f64())
    }

    /// Calculate distance to another point
    pub fn distance(&self, other: &PrecisionPoint) -> CadResult<Decimal128> {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let sum = dx * dx + dy * dy;
        sum.sqrt()
    }

    /// Calculate midpoint
    pub fn midpoint(&self, other: &PrecisionPoint) -> Self {
        let two = Decimal128::from(2);
        Self {
            x: (self.x + other.x) / two,
            y: (self.y + other.y) / two,
        }
    }

    /// Origin point (0, 0)
    pub fn origin() -> Self {
        Self {
            x: Decimal128::zero(),
            y: Decimal128::zero(),
        }
    }
}

/// Angle with high precision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrecisionAngle {
    radians: Decimal128,
}

impl PrecisionAngle {
    /// Create angle from radians
    pub fn from_radians(radians: Decimal128) -> Self {
        Self { radians }
    }

    /// Create angle from degrees
    pub fn from_degrees(degrees: Decimal128) -> Self {
        let pi = Decimal128::from_f64(std::f64::consts::PI).unwrap();
        let radians = degrees * pi / Decimal128::from(180);
        Self { radians }
    }

    /// Get angle in radians
    pub fn radians(&self) -> Decimal128 {
        self.radians
    }

    /// Get angle in degrees
    pub fn degrees(&self) -> Decimal128 {
        let pi = Decimal128::from_f64(std::f64::consts::PI).unwrap();
        self.radians * Decimal128::from(180) / pi
    }

    /// Normalize angle to [0, 2π)
    pub fn normalize(&self) -> Self {
        let two_pi = Decimal128::from_f64(2.0 * std::f64::consts::PI).unwrap();
        let mut normalized = self.radians;

        while normalized.to_f64() < 0.0 {
            normalized = normalized + two_pi;
        }
        while normalized.to_f64() >= 2.0 * std::f64::consts::PI {
            normalized = normalized - two_pi;
        }

        Self { radians: normalized }
    }
}

/// Tolerance-based comparison utilities
#[derive(Debug, Clone)]
pub struct ToleranceComparator {
    tolerance: f64,
}

impl ToleranceComparator {
    /// Create a new comparator with tolerance
    pub fn new(tolerance: f64) -> Self {
        Self { tolerance }
    }

    /// Check if two values are approximately equal
    pub fn approx_eq(&self, a: f64, b: f64) -> bool {
        (a - b).abs() < self.tolerance
    }

    /// Check if value is approximately zero
    pub fn approx_zero(&self, value: f64) -> bool {
        value.abs() < self.tolerance
    }

    /// Compare two values with tolerance
    pub fn compare(&self, a: f64, b: f64) -> std::cmp::Ordering {
        if self.approx_eq(a, b) {
            std::cmp::Ordering::Equal
        } else if a < b {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }

    /// Clamp value to tolerance if near zero
    pub fn clamp_zero(&self, value: f64) -> f64 {
        if self.approx_zero(value) {
            0.0
        } else {
            value
        }
    }
}

impl Default for ToleranceComparator {
    fn default() -> Self {
        Self::new(1e-6)
    }
}

/// Precision context for CAD operations
#[derive(Debug, Clone)]
pub struct PrecisionContext {
    pub engineering: EngineeringPrecision,
    pub comparator: ToleranceComparator,
}

impl Default for PrecisionContext {
    fn default() -> Self {
        let engineering = EngineeringPrecision::default();
        let comparator = ToleranceComparator::new(engineering.tolerance);
        Self {
            engineering,
            comparator,
        }
    }
}

impl PrecisionContext {
    /// Create context for specific engineering discipline
    pub fn for_discipline(discipline: EngineeringDiscipline) -> Self {
        let engineering = match discipline {
            EngineeringDiscipline::Architectural => EngineeringPrecision::architectural(),
            EngineeringDiscipline::Mechanical => EngineeringPrecision::mechanical(),
            EngineeringDiscipline::Scientific => EngineeringPrecision::scientific(),
        };

        let comparator = ToleranceComparator::new(engineering.tolerance);

        Self {
            engineering,
            comparator,
        }
    }
}

/// Engineering disciplines with different precision requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineeringDiscipline {
    Architectural,
    Mechanical,
    Scientific,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal128_arithmetic() {
        let a = Decimal128::new(10);
        let b = Decimal128::new(5);

        assert_eq!(a + b, Decimal128::new(15));
        assert_eq!(a - b, Decimal128::new(5));
        assert_eq!(a * b, Decimal128::new(50));
        assert_eq!(a / b, Decimal128::new(2));
    }

    #[test]
    fn test_precision_point() {
        let p1 = PrecisionPoint::new(Decimal128::new(0), Decimal128::new(0));
        let p2 = PrecisionPoint::new(Decimal128::new(3), Decimal128::new(4));

        let distance = p1.distance(&p2).unwrap();
        assert_eq!(distance.to_f64(), 5.0);
    }

    #[test]
    fn test_engineering_precision() {
        let precision = EngineeringPrecision::mechanical();
        assert_eq!(precision.display_precision, 4);

        let formatted = precision.format(3.14159265);
        assert_eq!(formatted, "3.1416");
    }

    #[test]
    fn test_tolerance_comparator() {
        let comp = ToleranceComparator::new(1e-6);
        assert!(comp.approx_eq(1.0, 1.0000001));
        assert!(!comp.approx_eq(1.0, 1.001));
    }

    #[test]
    fn test_unit_conversion() {
        let precision = EngineeringPrecision::default();
        let inches = precision.convert(10.0, PrecisionUnits::Millimeters, PrecisionUnits::Inches);
        assert!((inches - 0.3937).abs() < 0.001);
    }
}
