use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcceleratorBackend {
    Cpu,
    Cuda,
    Rocm,
}

impl AcceleratorBackend {
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "cpu" => Some(Self::Cpu),
            "cuda" | "nvidia" => Some(Self::Cuda),
            "rocm" | "amd" => Some(Self::Rocm),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Cuda => "cuda",
            Self::Rocm => "rocm",
        }
    }

    pub fn from_env(var: &str) -> Option<Self> {
        let value = std::env::var(var).ok()?;
        Self::parse(&value)
    }
}

impl Default for AcceleratorBackend {
    fn default() -> Self {
        Self::Cpu
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_parse_supports_cpu_cuda_rocm() {
        assert_eq!(
            AcceleratorBackend::parse("cpu"),
            Some(AcceleratorBackend::Cpu)
        );
        assert_eq!(
            AcceleratorBackend::parse("cuda"),
            Some(AcceleratorBackend::Cuda)
        );
        assert_eq!(
            AcceleratorBackend::parse("rocm"),
            Some(AcceleratorBackend::Rocm)
        );
        assert_eq!(AcceleratorBackend::parse("unknown"), None);
    }
}
