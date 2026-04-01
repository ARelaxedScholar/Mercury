use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// A single field in a semantic signature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub description: String,
}

impl Field {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

/// The semantic contract for a node, defining inputs and outputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Signature {
    pub inputs: Vec<Field>,
    pub outputs: Vec<Field>,
}

impl Signature {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    /// Add an input field to the signature.
    pub fn input(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.inputs.push(Field::new(name, description));
        self
    }

    /// Add an output field to the signature.
    pub fn output(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.outputs.push(Field::new(name, description));
        self
    }

    /// Returns a stable, structural hash of the signature.
    /// Descriptions are EXCLUDED from this hash as per the spec,
    /// to ensure prompt refinement doesn't break structural identity.
    pub fn structural_hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        for field in &self.inputs {
            field.name.hash(&mut hasher);
        }
        "input_separator".hash(&mut hasher);
        for field in &self.outputs {
            field.name.hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }
}

impl FromStr for Signature {
    type Err = String;

    /// Parses shorthand syntax: "input1, input2 -> output1, output2"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("->").collect();
        if parts.len() != 2 {
            return Err("Signature must contain exactly one '->'".to_string());
        }

        let parse_fields = |part: &str| {
            part.split(',')
                .map(|f| f.trim())
                .filter(|f| !f.is_empty())
                .map(|f| Field::new(f, ""))
                .collect::<Vec<Field>>()
        };

        Ok(Signature {
            inputs: parse_fields(parts[0]),
            outputs: parse_fields(parts[1]),
        })
    }
}

/// Macro for rapid signature creation: signature!("doc -> summary")
#[macro_export]
macro_rules! signature {
    ($s:expr) => {
        $s.parse::<$crate::core::semantic::signature::Signature>().expect("Invalid signature shorthand")
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_from_str_basic() {
        let sig = Signature::from_str("in1, in2 -> out1").unwrap();
        assert_eq!(sig.inputs.len(), 2);
        assert_eq!(sig.inputs[0].name, "in1");
        assert_eq!(sig.inputs[1].name, "in2");
        assert_eq!(sig.outputs.len(), 1);
        assert_eq!(sig.outputs[0].name, "out1");
    }

    #[test]
    fn test_from_str_single() {
        let sig = Signature::from_str("in -> out").unwrap();
        assert_eq!(sig.inputs.len(), 1);
        assert_eq!(sig.inputs[0].name, "in");
        assert_eq!(sig.outputs.len(), 1);
        assert_eq!(sig.outputs[0].name, "out");
    }

    #[test]
    fn test_from_str_no_inputs() {
        let sig = Signature::from_str(" -> out").unwrap();
        assert_eq!(sig.inputs.len(), 0);
        assert_eq!(sig.outputs.len(), 1);
        assert_eq!(sig.outputs[0].name, "out");
    }

    #[test]
    fn test_from_str_no_outputs() {
        let sig = Signature::from_str("in -> ").unwrap();
        assert_eq!(sig.inputs.len(), 1);
        assert_eq!(sig.inputs[0].name, "in");
        assert_eq!(sig.outputs.len(), 0);
    }

    #[test]
    fn test_from_str_empty() {
        let sig = Signature::from_str(" -> ").unwrap();
        assert_eq!(sig.inputs.len(), 0);
        assert_eq!(sig.outputs.len(), 0);
    }

    #[test]
    fn test_from_str_whitespace() {
        let sig = Signature::from_str("  in1  ,  in2  ->  out1  ").unwrap();
        assert_eq!(sig.inputs.len(), 2);
        assert_eq!(sig.inputs[0].name, "in1");
        assert_eq!(sig.inputs[1].name, "in2");
        assert_eq!(sig.outputs.len(), 1);
        assert_eq!(sig.outputs[0].name, "out1");
    }

    #[test]
    fn test_from_str_redundant_commas() {
        let sig = Signature::from_str("in1, , in2 -> out1").unwrap();
        assert_eq!(sig.inputs.len(), 2);
        assert_eq!(sig.inputs[0].name, "in1");
        assert_eq!(sig.inputs[1].name, "in2");
    }

    #[test]
    fn test_from_str_error_no_arrow() {
        let result = Signature::from_str("in out");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Signature must contain exactly one '->'");
    }

    #[test]
    fn test_from_str_error_multiple_arrows() {
        let result = Signature::from_str("in -> mid -> out");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Signature must contain exactly one '->'");
    }
}
