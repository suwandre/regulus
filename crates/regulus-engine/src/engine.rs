//! Engine orchestration: translate a Profile into hardware writes via the HAL.

use crate::budget::{split_budget, DomainLimits};
use crate::profile::Profile;
use regulus_hal::traits::{HalResult, PowerControl};
use regulus_hal::types::{CpuPowerLimits, GpuPowerLimit, Watts};

/// Map a CPU sustained budget into STAPM/slow/fast. M1 policy: slow = +15%,
/// fast = +30%, each clamped to `cpu_max`.
fn cpu_limits_from_sustained(stapm: Watts, cpu_max: Watts) -> CpuPowerLimits {
    let s = stapm.get();
    CpuPowerLimits {
        stapm,
        slow: Watts::new(((s as f32 * 1.15) as u32).min(cpu_max.get())),
        fast: Watts::new(((s as f32 * 1.30) as u32).min(cpu_max.get())),
    }
}

pub fn apply_profile(
    power: &mut dyn PowerControl,
    profile: &Profile,
    limits: &DomainLimits,
) -> HalResult<()> {
    let split = split_budget(Watts::new(profile.budget_watts), profile.bias, limits);
    power.set_cpu_limits(cpu_limits_from_sustained(split.cpu, limits.cpu_max))?;
    power.set_gpu_limit(GpuPowerLimit(split.gpu))?;
    Ok(())
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
    fn apply_profile_sets_cpu_and_gpu_from_budget_split() {
        let mut be = MockBackend::default();
        let prof = Profile {
            name: "X".into(),
            budget_watts: 100,
            bias: 0.5,
        };
        apply_profile(&mut be, &prof, &limits()).unwrap();
        assert_eq!(be.last_gpu.unwrap().0, Watts::new(50));
        let cpu = be.last_cpu.unwrap();
        assert_eq!(cpu.stapm, Watts::new(50));
        assert!(cpu.fast >= cpu.slow && cpu.slow >= cpu.stapm);
    }
}
