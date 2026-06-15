//! Capability traits. A backend implements the subset it supports; the engine
//! composes trait objects and the UI renders only supported capabilities.

use crate::types::{CpuPowerLimits, GpuPowerLimit, TelemetrySample, Watts};

#[derive(Debug, thiserror::Error)]
pub enum HalError {
    #[error("capability not supported on this hardware")]
    Unsupported,
    #[error("backend error: {0}")]
    Backend(String),
}

pub type HalResult<T> = Result<T, HalError>;

/// Setting CPU and GPU power limits.
pub trait PowerControl: Send {
    fn set_cpu_limits(&mut self, limits: CpuPowerLimits) -> HalResult<()>;
    fn set_gpu_limit(&mut self, limit: GpuPowerLimit) -> HalResult<()>;
    /// Backend-reported allowable GPU range (min, max) in watts.
    fn gpu_limit_range(&self) -> HalResult<(Watts, Watts)>;
}

/// Reading a telemetry sample.
pub trait Telemetry: Send {
    fn read(&mut self) -> HalResult<TelemetrySample>;
}
