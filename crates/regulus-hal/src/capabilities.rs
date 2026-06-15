//! Runtime capability model: what the current machine's loaded backends support.

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Capability {
    CpuPower,
    GpuPower,
    Telemetry,
    Cooler,
    Display,
    Overclock,
}

#[derive(Debug, Default, Clone)]
pub struct CapabilitySet(BTreeSet<Capability>);

impl CapabilitySet {
    pub fn empty() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, c: Capability) {
        self.0.insert(c);
    }
    pub fn has(&self, c: Capability) -> bool {
        self.0.contains(&c)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_set_reports_membership() {
        let mut caps = CapabilitySet::empty();
        caps.insert(Capability::CpuPower);
        caps.insert(Capability::Telemetry);
        assert!(caps.has(Capability::CpuPower));
        assert!(!caps.has(Capability::Cooler));
        assert_eq!(caps.len(), 2);
    }
}
