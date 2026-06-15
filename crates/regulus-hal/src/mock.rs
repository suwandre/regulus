//! In-memory backend for tests.

use crate::traits::{HalResult, PowerControl, Telemetry};
use crate::types::{CpuPowerLimits, GpuPowerLimit, TelemetrySample, Watts};

#[derive(Default)]
pub struct MockBackend {
    pub last_cpu: Option<CpuPowerLimits>,
    pub last_gpu: Option<GpuPowerLimit>,
    pub sample: TelemetrySample,
}

impl PowerControl for MockBackend {
    fn set_cpu_limits(&mut self, limits: CpuPowerLimits) -> HalResult<()> {
        self.last_cpu = Some(limits);
        Ok(())
    }
    fn set_gpu_limit(&mut self, limit: GpuPowerLimit) -> HalResult<()> {
        self.last_gpu = Some(limit);
        Ok(())
    }
    fn gpu_limit_range(&self) -> HalResult<(Watts, Watts)> {
        Ok((Watts::new(5), Watts::new(175)))
    }
}

impl Telemetry for MockBackend {
    fn read(&mut self) -> HalResult<TelemetrySample> {
        Ok(self.sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{PowerControl, Telemetry};
    use crate::types::{CpuPowerLimits, GpuPowerLimit, Watts};

    #[test]
    fn mock_records_applied_limits() {
        let mut be = MockBackend::default();
        let cpu = CpuPowerLimits {
            stapm: Watts::new(50),
            slow: Watts::new(60),
            fast: Watts::new(70),
        };
        be.set_cpu_limits(cpu).unwrap();
        be.set_gpu_limit(GpuPowerLimit(Watts::new(120))).unwrap();
        assert_eq!(be.last_cpu, Some(cpu));
        assert_eq!(be.last_gpu, Some(GpuPowerLimit(Watts::new(120))));
    }

    #[test]
    fn mock_returns_seeded_sample() {
        let mut be = MockBackend::default();
        be.sample.cpu_temp_c = Some(84.0);
        assert_eq!(be.read().unwrap().cpu_temp_c, Some(84.0));
    }
}
