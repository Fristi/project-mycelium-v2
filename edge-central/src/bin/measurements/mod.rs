pub mod random;
pub mod types;

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub mod trouble;
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub mod btleplug;