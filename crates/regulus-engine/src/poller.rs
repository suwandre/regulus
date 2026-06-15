//! Telemetry poller. The engine calls `tick` on a 1–2 Hz timer (the real timer
//! lives in the app; tests drive `tick` directly). The UI reads `latest` — never
//! the hardware.

use regulus_hal::traits::Telemetry;
use regulus_hal::types::TelemetrySample;

#[derive(Default)]
pub struct Poller {
    latest: TelemetrySample,
}

impl Poller {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn latest(&self) -> TelemetrySample {
        self.latest
    }
    pub fn tick(&mut self, source: &mut dyn Telemetry) {
        if let Ok(s) = source.read() {
            self.latest = s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regulus_hal::mock::MockBackend;

    #[test]
    fn tick_updates_latest_sample() {
        let mut be = MockBackend::default();
        be.sample.cpu_temp_c = Some(70.0);
        let mut poller = Poller::new();
        assert!(poller.latest().cpu_temp_c.is_none());
        poller.tick(&mut be);
        assert_eq!(poller.latest().cpu_temp_c, Some(70.0));
    }
}
