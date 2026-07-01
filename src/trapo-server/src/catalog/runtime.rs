pub fn runtime_variants(app_root: &Path) -> Vec<RuntimeVariant> {
    let probe = detect_hardware();
    runtime_specs()
        .into_iter()
        .map(|spec| {
            let ffi = app_root
                .join("thirdparty")
                .join("uocr-runtime")
                .join(spec.platform)
                .join("bin")
                .join(spec.library_name);
            let installed = ffi.is_file();
            let hardware_supported = match spec.accelerator {
                "cuda" => probe.cuda,
                "rocm" => probe.rocm,
                "metal" => probe.metal,
                _ => true,
            };
            RuntimeVariant {
                runtime_id: spec.platform.to_string(),
                platform: spec.platform.to_string(),
                label: spec.label.to_string(),
                accelerator: spec.accelerator.to_string(),
                backend: spec.backend.to_string(),
                ffi_library: ffi.to_string_lossy().to_string(),
                priority: spec.priority,
                n_gpu_layers: spec.n_gpu_layers,
                installed,
                hardware_supported,
                selectable: installed && hardware_supported,
                support_detail: support_detail(&probe, spec.accelerator),
            }
        })
        .collect()
}

pub fn choose_runtime_id(variants: &[RuntimeVariant], preferred: &str) -> String {
    if variants
        .iter()
        .any(|item| item.runtime_id == preferred && item.selectable)
    {
        return preferred.to_string();
    }
    variants
        .iter()
        .filter(|item| item.selectable)
        .max_by_key(|item| item.priority)
        .or_else(|| variants.first())
        .map(|item| item.runtime_id.clone())
        .unwrap_or_default()
}

pub fn runtime_record(variant: &RuntimeVariant, selected_runtime_id: &str) -> RuntimeVariantRecord {
    RuntimeVariantRecord {
        runtime_id: variant.runtime_id.clone(),
        label: variant.label.clone(),
        platform: variant.platform.clone(),
        accelerator: variant.accelerator.clone(),
        backend: variant.backend.clone(),
        ffi_library: variant.ffi_library.clone(),
        installed: variant.installed,
        hardware_supported: variant.hardware_supported,
        selectable: variant.selectable,
        selected: variant.runtime_id == selected_runtime_id,
        support_detail: variant.support_detail.clone(),
    }
}

fn detect_hardware() -> HardwareProbe {
    HardwareProbe {
        cuda: env_flag("UOCR_FORCE_CUDA")
            || command_exists("nvidia-smi")
            || Path::new("C:/Windows/System32/nvidia-smi.exe").is_file(),
        rocm: env_flag("UOCR_FORCE_ROCM")
            || command_exists("rocminfo")
            || command_exists("hipinfo")
            || Path::new("C:/Program Files/AMD/ROCm/bin/rocminfo.exe").is_file()
            || Path::new("C:/Program Files/AMD/ROCm/bin/hipinfo.exe").is_file(),
        metal: cfg!(target_os = "macos"),
    }
}

fn env_flag(name: &str) -> bool {
    env::var(name).is_ok_and(|value| !value.is_empty() && value != "0")
}

fn command_exists(command: &str) -> bool {
    Command::new(command).arg("--help").output().is_ok()
}

fn support_detail(probe: &HardwareProbe, accelerator: &str) -> String {
    match accelerator {
        "cuda" if probe.cuda => "NVIDIA CUDA probe found".to_string(),
        "cuda" => "nvidia-smi was not found".to_string(),
        "rocm" if probe.rocm => "AMD ROCm/HIP probe found".to_string(),
        "rocm" => "rocminfo/hipinfo was not found".to_string(),
        "metal" if probe.metal => "Apple Metal runtime is available".to_string(),
        "metal" => "Apple Metal runtime is unavailable".to_string(),
        _ => "CPU runtime is always supported".to_string(),
    }
}

fn runtime_specs() -> Vec<RuntimeSpec> {
    if cfg!(windows) {
        if cfg!(target_arch = "aarch64") {
            return vec![RuntimeSpec {
                platform: "windows-arm64-cpu",
                label: "Windows arm64 CPU",
                accelerator: "cpu",
                backend: "cpu",
                library_name: "uocr-ffi.dll",
                priority: 100,
                n_gpu_layers: 0,
            }];
        }
        vec![
            RuntimeSpec {
                platform: "windows-x86_64-cuda13",
                label: "Windows x64 CUDA 13",
                accelerator: "cuda",
                backend: "cuda",
                library_name: "uocr-ffi.dll",
                priority: 300,
                n_gpu_layers: -2,
            },
            RuntimeSpec {
                platform: "windows-x86_64-rocm6",
                label: "Windows x64 AMD ROCm/HIP",
                accelerator: "rocm",
                backend: "rocm",
                library_name: "uocr-ffi.dll",
                priority: 200,
                n_gpu_layers: -2,
            },
            RuntimeSpec {
                platform: "windows-x86_64-cpu",
                label: "Windows x64 CPU",
                accelerator: "cpu",
                backend: "cpu",
                library_name: "uocr-ffi.dll",
                priority: 100,
                n_gpu_layers: 0,
            },
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            RuntimeSpec {
                platform: "macos-arm64-metal",
                label: "macOS Apple Silicon Metal",
                accelerator: "metal",
                backend: "metal",
                library_name: "libuocr-ffi.dylib",
                priority: 300,
                n_gpu_layers: -2,
            },
            RuntimeSpec {
                platform: "macos-arm64-cpu",
                label: "macOS Apple Silicon CPU",
                accelerator: "cpu",
                backend: "cpu",
                library_name: "libuocr-ffi.dylib",
                priority: 100,
                n_gpu_layers: 0,
            },
        ]
    } else if cfg!(any(target_arch = "aarch64", target_arch = "arm64ec")) {
        vec![RuntimeSpec {
            platform: "linux-arm64-cpu",
            label: "Linux arm64 CPU",
            accelerator: "cpu",
            backend: "cpu",
            library_name: "libuocr-ffi.so",
            priority: 100,
            n_gpu_layers: 0,
        }]
    } else {
        vec![
            RuntimeSpec {
                platform: "linux-x86_64-cuda13",
                label: "Linux x64 CUDA 13",
                accelerator: "cuda",
                backend: "cuda",
                library_name: "libuocr-ffi.so",
                priority: 300,
                n_gpu_layers: -2,
            },
            RuntimeSpec {
                platform: "linux-x86_64-rocm6",
                label: "Linux x64 AMD ROCm/HIP",
                accelerator: "rocm",
                backend: "rocm",
                library_name: "libuocr-ffi.so",
                priority: 200,
                n_gpu_layers: -2,
            },
            RuntimeSpec {
                platform: "linux-x86_64-cpu",
                label: "Linux x64 CPU",
                accelerator: "cpu",
                backend: "cpu",
                library_name: "libuocr-ffi.so",
                priority: 100,
                n_gpu_layers: 0,
            },
        ]
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    #[cfg(all(windows, target_arch = "aarch64"))]
    fn windows_arm64_uses_native_cpu_runtime() {
        let specs = runtime_specs();

        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].platform, "windows-arm64-cpu");
        assert_eq!(specs[0].library_name, "uocr-ffi.dll");
    }

    #[test]
    #[cfg(all(windows, not(target_arch = "aarch64")))]
    fn windows_x64_keeps_cuda_and_cpu_runtimes() {
        let platforms: Vec<_> = runtime_specs()
            .into_iter()
            .map(|spec| spec.platform)
            .collect();

        assert!(platforms.contains(&"windows-x86_64-cuda13"));
        assert!(platforms.contains(&"windows-x86_64-cpu"));
        assert!(!platforms.contains(&"windows-arm64-cpu"));
    }
}
