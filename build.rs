fn main() {
    // For Windows, libgit2 needs these system libraries for cryptography and registry functions
    #[cfg(target_os = "windows")]
    {
        // Try to find Windows SDK directory
        if let Ok(sdk_version) = std::env::var("WindowsSDKVersion") {
            let sdk_root = std::env::var("WindowsSDKLibVersion")
                .unwrap_or_else(|_| sdk_version.trim_end_matches('\\').to_string());

            println!("cargo:rustc-link-search=native=C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\{}\\um\\x64", sdk_root);
        }

        // Standard Windows SDK libs
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=kernel32");
        println!("cargo:rustc-link-lib=crypt32");
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=ws2_32");
    }
}
