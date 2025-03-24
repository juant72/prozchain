fn main() {
    // Inform cargo to rerun this script if any of these files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    
    // Set feature flags if needed based on environment
    #[cfg(feature = "metrics")]
    println!("cargo:rustc-cfg=metrics");
    
    // Inform the build about target platforms
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-cfg=linux_platform");
    
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-cfg=windows_platform");
    
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-cfg=macos_platform");
}
