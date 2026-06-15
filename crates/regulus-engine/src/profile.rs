//! A profile is the user-facing unit: a combined budget + bias, by name.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub budget_watts: u32,
    /// CPU↔GPU bias, 0.0 = all CPU, 1.0 = all GPU.
    pub bias: f32,
}

impl Profile {
    pub fn presets() -> Vec<Profile> {
        vec![
            Profile {
                name: "Quiet".into(),
                budget_watts: 60,
                bias: 0.45,
            },
            Profile {
                name: "Balanced".into(),
                budget_watts: 130,
                bias: 0.55,
            },
            Profile {
                name: "Beast".into(),
                budget_watts: 250,
                bias: 0.6,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_exist_and_differ() {
        let ps = Profile::presets();
        let names: Vec<_> = ps.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Quiet"));
        assert!(names.contains(&"Balanced"));
        assert!(names.contains(&"Beast"));
        assert!(
            ps.iter().find(|p| p.name == "Quiet").unwrap().budget_watts
                < ps.iter().find(|p| p.name == "Beast").unwrap().budget_watts
        );
    }

    #[test]
    fn profile_toml_roundtrip() {
        let p = Profile {
            name: "Custom".into(),
            budget_watts: 130,
            bias: 0.6,
        };
        let s = toml::to_string(&p).unwrap();
        let back: Profile = toml::from_str(&s).unwrap();
        assert_eq!(p, back);
    }
}
