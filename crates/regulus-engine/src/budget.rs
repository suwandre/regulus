//! Software combined-budget policy: split a total CPU+GPU budget into per-domain
//! caps. These are caps, not guaranteed draw. Minimums are hard floors (a domain
//! is never starved below its minimum even if that pushes the sum past the
//! total target); otherwise the invariant `cpu + gpu <= total` holds.

use regulus_hal::types::Watts;

#[derive(Debug, Clone, Copy)]
pub struct DomainLimits {
    pub cpu_min: Watts,
    pub cpu_max: Watts,
    pub gpu_min: Watts,
    pub gpu_max: Watts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    pub cpu: Watts,
    pub gpu: Watts,
}

/// `bias`: 0.0 = all CPU, 1.0 = all GPU. Clamped to [0,1].
pub fn split_budget(total: Watts, bias: f32, limits: &DomainLimits) -> Split {
    let total = total.get() as i64;
    let bias = bias.clamp(0.0, 1.0) as f64;

    let (cpu_min, cpu_max) = (limits.cpu_min.get() as i64, limits.cpu_max.get() as i64);
    let (gpu_min, gpu_max) = (limits.gpu_min.get() as i64, limits.gpu_max.get() as i64);

    // Raw desired split, then clamp each domain to its range.
    let mut gpu = (total as f64 * bias).round() as i64;
    let mut cpu = total - gpu;
    gpu = gpu.clamp(gpu_min, gpu_max);
    cpu = cpu.clamp(cpu_min, cpu_max);

    let diff = total - (cpu + gpu);
    let gpu_priority = bias >= 0.5;

    if diff > 0 {
        // Surplus (a domain clamped low): fill the priority domain first, up to max.
        let mut left = diff;
        for is_gpu in [gpu_priority, !gpu_priority] {
            if is_gpu {
                let give = left.min(gpu_max - gpu);
                gpu += give;
                left -= give;
            } else {
                let give = left.min(cpu_max - cpu);
                cpu += give;
                left -= give;
            }
        }
    } else if diff < 0 {
        // Overshoot (a domain clamped up to its min): cut the non-priority domain
        // first, down to its min, to honor the total cap.
        let mut excess = -diff;
        for is_gpu in [!gpu_priority, gpu_priority] {
            if is_gpu {
                let cut = excess.min(gpu - gpu_min);
                gpu -= cut;
                excess -= cut;
            } else {
                let cut = excess.min(cpu - cpu_min);
                cpu -= cut;
                excess -= cut;
            }
        }
    }

    Split {
        cpu: Watts::new(cpu.max(0) as u32),
        gpu: Watts::new(gpu.max(0) as u32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> DomainLimits {
        DomainLimits {
            cpu_min: Watts::new(10),
            cpu_max: Watts::new(120),
            gpu_min: Watts::new(5),
            gpu_max: Watts::new(175),
        }
    }

    #[test]
    fn even_split_at_half_bias() {
        let s = split_budget(Watts::new(100), 0.5, &limits());
        assert_eq!(s.cpu.get() + s.gpu.get(), 100);
        assert_eq!(s.gpu, Watts::new(50));
        assert_eq!(s.cpu, Watts::new(50));
    }

    #[test]
    fn full_gpu_bias_caps_at_gpu_max_and_reallocates_surplus() {
        // bias 1.0 wants all 300W to GPU, but gpu_max=175; surplus → cpu up to its max.
        let s = split_budget(Watts::new(300), 1.0, &limits());
        assert!(s.cpu.get() + s.gpu.get() <= 300);
        assert_eq!(s.gpu, Watts::new(175));
        assert_eq!(s.cpu, Watts::new(120));
    }

    #[test]
    fn high_bias_within_range_splits_directly() {
        let s = split_budget(Watts::new(100), 0.9, &limits());
        assert_eq!(s.cpu.get() + s.gpu.get(), 100);
        assert_eq!(s.gpu, Watts::new(90));
        assert_eq!(s.cpu, Watts::new(10));
    }

    #[test]
    fn min_clamp_never_exceeds_total() {
        // bias clamps to 1.0; cpu forced to its min(10), so gpu must drop to 90 to fit 100.
        let s = split_budget(Watts::new(100), 5.0, &limits());
        assert!(s.cpu.get() + s.gpu.get() <= 100);
        assert_eq!(s.gpu, Watts::new(90));
        assert_eq!(s.cpu, Watts::new(10));
    }

    #[test]
    fn tiny_budget_respects_minimums_even_if_it_exceeds_total() {
        // total 8 < cpu_min(10)+gpu_min(5): minimums win, total is a soft target.
        let s = split_budget(Watts::new(8), 0.5, &limits());
        assert_eq!(s.cpu, Watts::new(10));
        assert_eq!(s.gpu, Watts::new(5));
    }
}
