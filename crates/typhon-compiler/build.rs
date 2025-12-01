fn main() {
    // If LLVM_SYS_181_PREFIX isn't set, try to auto-detect common installation locations
    if std::env::var_os("LLVM_SYS_181_PREFIX").is_none() {
        // Common LLVM installation paths
        let common_paths = vec![
            "/usr/local/opt/llvm@18",
            "/usr/local/opt/llvm",
            "/opt/homebrew/opt/llvm@18",
            "/opt/homebrew/opt/llvm",
            "/usr/lib/llvm-18",
            "/usr/local/lib/llvm-18",
        ];

        for path in common_paths {
            let path = std::path::Path::new(path);
            if path.exists() {
                println!("cargo:rustc-env=LLVM_SYS_181_PREFIX={}", path.display());
                break;
            }
        }
    }

    // Always print the config used
    println!(
        "cargo:warning=Using LLVM from: {}",
        std::env::var("LLVM_SYS_181_PREFIX").unwrap_or_else(|_| "system path".to_string())
    );
}
