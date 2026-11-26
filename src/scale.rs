//! Audio-native parameter scaling, display formatting, and text-to-value parsing.

/// Defines how a continuous parameter maps between its plain domain value and
/// normalized UI range (`0.0..=1.0`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParamScale {
    /// Uniform linear mapping between `min` and `max`.
    Linear { min: f32, max: f32 },
    /// Logarithmic mapping (ideal for frequency cutoffs such as `20.0..=20000.0` Hz).
    Log { min: f32, max: f32 },
    /// Decibel gain mapping where values at or below `min_db` format as `-inf dB`.
    Decibel { min_db: f32, max_db: f32 },
    /// Center-zero symmetric range (`-max_abs..=+max_abs`) for pan, detune, or bipolar depth.
    Bipolar { max_abs: f32 },
}

impl ParamScale {
    /// Maps a plain parameter value into normalized `0.0..=1.0` space.
    pub fn normalize(&self, value: f32) -> f32 {
        match *self {
            Self::Linear { min, max } => ((value - min) / (max - min)).clamp(0.0, 1.0),
            Self::Log { min, max } => {
                let v = value.clamp(min.max(1e-6), max);
                ((v / min.max(1e-6)).ln() / (max / min.max(1e-6)).ln()).clamp(0.0, 1.0)
            }
            Self::Decibel { min_db, max_db } => {
                if value <= min_db {
                    0.0
                } else {
                    ((value - min_db) / (max_db - min_db)).clamp(0.0, 1.0)
                }
            }
            Self::Bipolar { max_abs } => ((value / max_abs) * 0.5 + 0.5).clamp(0.0, 1.0),
        }
    }

    /// Maps a normalized `0.0..=1.0` value back into the plain parameter domain.
    pub fn denormalize(&self, norm: f32) -> f32 {
        let t = norm.clamp(0.0, 1.0);
        match *self {
            Self::Linear { min, max } => min + t * (max - min),
            Self::Log { min, max } => {
                let lo = min.max(1e-6);
                lo * (max / lo).powf(t)
            }
            Self::Decibel { min_db, max_db } => min_db + t * (max_db - min_db),
            Self::Bipolar { max_abs } => (t * 2.0 - 1.0) * max_abs,
        }
    }

    /// Returns `true` if this scale is center-zero (`ParamScale::Bipolar`).
    pub fn is_bipolar(&self) -> bool {
        matches!(self, Self::Bipolar { .. })
    }

    /// Formats a plain parameter value with optional unit suffix (auto-scaling `Hz` to `kHz`).
    pub fn format(&self, value: f32, unit: &str) -> String {
        match *self {
            Self::Log { .. } if unit == "Hz" && value >= 1000.0 => {
                format!("{:.2} kHz", value / 1000.0)
            }
            Self::Decibel { min_db, .. } if value <= min_db + 0.05 => "-inf dB".to_string(),
            Self::Bipolar { .. } => format!("{:+.2}{}", value, unit),
            _ => {
                if unit.is_empty() {
                    format!("{:.2}", value)
                } else {
                    format!("{:.1} {}", value, unit)
                }
            }
        }
    }

    /// Parses strings like "1.2k", "-6dB", "440 Hz", "50%".
    pub fn parse(&self, input: &str) -> Option<f32> {
        let s = input.trim().to_lowercase();
        if s.contains("-inf") {
            if let Self::Decibel { min_db, .. } = *self {
                return Some(min_db);
            }
        }
        let mult = if s.contains("khz") || s.ends_with('k') { 1000.0 } else { 1.0 };
        let cleaned: String = s
            .chars()
            .filter(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+'))
            .collect();
        cleaned.parse::<f32>().ok().map(|v| {
            let scaled = v * mult;
            self.denormalize(self.normalize(scaled))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_linear_log_db_bipolar() {
        let scales = [
            ParamScale::Linear { min: 0.0, max: 100.0 },
            ParamScale::Log { min: 20.0, max: 20000.0 },
            ParamScale::Decibel { min_db: -60.0, max_db: 6.0 },
            ParamScale::Bipolar { max_abs: 1.0 },
        ];
        for scale in scales {
            for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let v = scale.denormalize(t);
                let back = scale.normalize(v);
                assert!((back - t).abs() < 1e-4, "scale {:?} failed at t={}", scale, t);
            }
        }
    }

    #[test]
    fn parse_units_and_edge_cases() {
        let freq = ParamScale::Log { min: 20.0, max: 20000.0 };
        assert!((freq.parse("1.2k").unwrap() - 1200.0).abs() < 1.0);
        assert!((freq.parse("440 Hz").unwrap() - 440.0).abs() < 1.0);
        assert_eq!(freq.parse("not-a-number"), None);

        let db = ParamScale::Decibel { min_db: -60.0, max_db: 6.0 };
        assert!((db.parse("-6dB").unwrap() - (-6.0)).abs() < 1e-3);
        assert_eq!(db.parse("-inf dB"), Some(-60.0));
        assert_eq!(db.format(-60.0, "dB"), "-inf dB");
    }
}
