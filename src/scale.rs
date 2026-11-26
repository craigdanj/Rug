#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParamScale {
    Linear { min: f32, max: f32 },
    Log { min: f32, max: f32 },
    Decibel { min_db: f32, max_db: f32 },
    Bipolar { max_abs: f32 },
}

impl ParamScale {
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

    pub fn is_bipolar(&self) -> bool {
        matches!(self, Self::Bipolar { .. })
    }

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
