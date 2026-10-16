fn main() {
    // Expose the build time so the startup banner can name the exact binary.
    // Emitting an env var makes cargo rebuild this crate whenever the value
    // changes, which is why `src` is watched explicitly (tauri-build otherwise
    // narrows rerun-triggering to tauri.conf.json and the icons).
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=WIND_BUILD_EPOCH={epoch}");
    println!("cargo:rerun-if-changed=src");

    tauri_build::build()
}
