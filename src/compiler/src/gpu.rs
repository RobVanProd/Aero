use crate::accelerator::AcceleratorBackend;
use std::process::Command;

pub trait DeviceProfile {
    fn backend(&self) -> AcceleratorBackend;
    fn target_triple(&self) -> &'static str;
    fn mcpu(&self) -> &str;
    fn mattr(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDevice {
    backend: AcceleratorBackend,
    device_id: i32,
    gpu_arch: String,
}

impl GpuDevice {
    pub fn new(backend: AcceleratorBackend, device_id: i32, gpu_arch: Option<String>) -> Self {
        let fallback_arch = default_gpu_arch(backend).unwrap_or("");
        let resolved_arch = gpu_arch.unwrap_or_else(|| fallback_arch.to_string());
        Self {
            backend,
            device_id,
            gpu_arch: resolved_arch,
        }
    }

    pub fn auto_detect() -> Self {
        if let Some(backend) = AcceleratorBackend::from_env("AERO_ACCELERATOR") {
            return Self::new(backend, 0, None);
        }
        if rocm_runtime_available() {
            return Self::new(AcceleratorBackend::Rocm, 0, None);
        }
        Self::new(AcceleratorBackend::Cpu, 0, None)
    }

    pub fn device_id(&self) -> i32 {
        self.device_id
    }

    pub fn gpu_arch(&self) -> &str {
        &self.gpu_arch
    }

    pub fn llc_target_flags(&self) -> Option<Vec<String>> {
        match self.backend {
            AcceleratorBackend::Rocm => Some(vec![
                "-march=amdgcn".to_string(),
                format!("-mcpu={}", self.mcpu()),
                format!("-mattr={}", self.mattr()),
            ]),
            AcceleratorBackend::Cuda | AcceleratorBackend::Cpu => None,
        }
    }
}

impl DeviceProfile for GpuDevice {
    fn backend(&self) -> AcceleratorBackend {
        self.backend
    }

    fn target_triple(&self) -> &'static str {
        match self.backend {
            AcceleratorBackend::Rocm => "amdgcn-amd-amdhsa",
            AcceleratorBackend::Cuda => "nvptx64-nvidia-cuda",
            AcceleratorBackend::Cpu => host_target_triple(),
        }
    }

    fn mcpu(&self) -> &str {
        if self.gpu_arch.is_empty() {
            default_gpu_arch(self.backend).unwrap_or("")
        } else {
            &self.gpu_arch
        }
    }

    fn mattr(&self) -> &'static str {
        match self.backend {
            AcceleratorBackend::Rocm => "+wavefrontsize64,+gfx11-insts",
            AcceleratorBackend::Cuda | AcceleratorBackend::Cpu => "",
        }
    }
}

pub fn default_gpu_arch(backend: AcceleratorBackend) -> Option<&'static str> {
    match backend {
        AcceleratorBackend::Cpu => None,
        AcceleratorBackend::Rocm => Some("gfx1101"),
        AcceleratorBackend::Cuda => Some("sm_89"),
    }
}

fn host_target_triple() -> &'static str {
    if cfg!(target_os = "windows") {
        "x86_64-pc-windows-msvc"
    } else if cfg!(target_os = "macos") {
        "x86_64-apple-darwin"
    } else {
        "x86_64-pc-linux-gnu"
    }
}

fn rocm_runtime_available() -> bool {
    command_exists("hipconfig")
        || command_exists("rocminfo")
        || std::env::var_os("HIP_PATH").is_some()
        || std::env::var_os("ROCM_PATH").is_some()
}

fn command_exists(command: &str) -> bool {
    Command::new(command).arg("--version").output().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rocm_device_defaults_to_gfx1101() {
        let device = GpuDevice::new(AcceleratorBackend::Rocm, 0, None);
        assert_eq!(device.backend(), AcceleratorBackend::Rocm);
        assert_eq!(device.target_triple(), "amdgcn-amd-amdhsa");
        assert_eq!(device.mcpu(), "gfx1101");
        assert_eq!(device.mattr(), "+wavefrontsize64,+gfx11-insts");
    }

    #[test]
    fn rocm_llc_flags_include_expected_codegen_switches() {
        let device = GpuDevice::new(AcceleratorBackend::Rocm, 0, Some("gfx1101".to_string()));
        let flags = device
            .llc_target_flags()
            .expect("rocm should emit llc flags");
        assert_eq!(flags[0], "-march=amdgcn");
        assert_eq!(flags[1], "-mcpu=gfx1101");
        assert_eq!(flags[2], "-mattr=+wavefrontsize64,+gfx11-insts");
    }
}
