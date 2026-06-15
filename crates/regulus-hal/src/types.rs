//! Core domain types shared across the HAL.

use serde::{Deserialize, Serialize};

/// Whole watts. The single power unit used across the UI and engine; converted
/// to milliwatts only at FFI boundaries (libryzenadj, NVML).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Watts(u32);

impl Watts {
    pub const fn new(w: u32) -> Self {
        Watts(w)
    }
    pub const fn get(self) -> u32 {
        self.0
    }
    pub const fn as_milliwatts(self) -> u32 {
        self.0 * 1000
    }
    pub fn clamp(self, lo: Watts, hi: Watts) -> Watts {
        Watts(self.0.clamp(lo.0, hi.0))
    }
}

/// CPU power limits in the AMD SMU model (sustained / slow PPT / fast PPT).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuPowerLimits {
    pub stapm: Watts,
    pub slow: Watts,
    pub fast: Watts,
}

/// Discrete GPU power-limit target (TGP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuPowerLimit(pub Watts);

/// One telemetry sample. Fields are `Option` because not every backend can read
/// every metric (e.g. CPU power readout is unavailable on Fire Range without MSR
/// access).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TelemetrySample {
    pub cpu_power: Option<Watts>,
    pub cpu_temp_c: Option<f32>,
    pub cpu_clock_mhz: Option<u32>,
    pub gpu_power: Option<Watts>,
    pub gpu_temp_c: Option<f32>,
    pub gpu_clock_mhz: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watts_clamp_into_range() {
        assert_eq!(
            Watts::new(200).clamp(Watts::new(5), Watts::new(120)),
            Watts::new(120)
        );
        assert_eq!(
            Watts::new(1).clamp(Watts::new(5), Watts::new(120)),
            Watts::new(5)
        );
        assert_eq!(
            Watts::new(50).clamp(Watts::new(5), Watts::new(120)),
            Watts::new(50)
        );
    }

    #[test]
    fn watts_to_milliwatts_roundtrip() {
        assert_eq!(Watts::new(80).as_milliwatts(), 80_000);
    }
}
