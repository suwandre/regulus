//! Reapply controller: limits reset on reboot/wake, so re-push the active
//! profile when the OS signals boot or resume. The app feeds OS power events
//! in here.

use crate::budget::DomainLimits;
use crate::engine::apply_profile;
use crate::profile::Profile;
use regulus_hal::traits::PowerControl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEvent {
    Boot,
    Resume,
    Suspend,
}

pub struct ReapplyController {
    active: Profile,
    limits: DomainLimits,
    reapply_count: u64,
}

impl ReapplyController {
    pub fn new(active: Profile, limits: DomainLimits) -> Self {
        Self {
            active,
            limits,
            reapply_count: 0,
        }
    }
    pub fn set_active(&mut self, p: Profile) {
        self.active = p;
    }
    pub fn reapply_count(&self) -> u64 {
        self.reapply_count
    }

    pub fn on_event(&mut self, power: &mut dyn PowerControl, ev: PowerEvent) {
        if matches!(ev, PowerEvent::Boot | PowerEvent::Resume)
            && apply_profile(power, &self.active, &self.limits).is_ok()
        {
            self.reapply_count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regulus_hal::mock::MockBackend;
    use regulus_hal::types::Watts;

    fn limits() -> DomainLimits {
        DomainLimits {
            cpu_min: Watts::new(10),
            cpu_max: Watts::new(120),
            gpu_min: Watts::new(5),
            gpu_max: Watts::new(175),
        }
    }

    #[test]
    fn resume_event_reapplies_active_profile() {
        let mut be = MockBackend::default();
        let prof = Profile {
            name: "X".into(),
            budget_watts: 100,
            bias: 0.5,
        };
        let mut ctrl = ReapplyController::new(prof, limits());
        ctrl.on_event(&mut be, PowerEvent::Resume);
        assert_eq!(be.last_gpu.unwrap().0, Watts::new(50));
        assert_eq!(ctrl.reapply_count(), 1);
    }

    #[test]
    fn suspend_event_does_not_reapply() {
        let mut be = MockBackend::default();
        let prof = Profile {
            name: "X".into(),
            budget_watts: 100,
            bias: 0.5,
        };
        let mut ctrl = ReapplyController::new(prof, limits());
        ctrl.on_event(&mut be, PowerEvent::Suspend);
        assert_eq!(ctrl.reapply_count(), 0);
        assert!(be.last_gpu.is_none());
    }
}
