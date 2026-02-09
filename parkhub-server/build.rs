//! Build script for ParkHub Server
//!
//! Compiles Slint UI files for the setup wizard (when gui feature is enabled).
//! Sets BUILD_DATE environment variable for version endpoint.

fn main() {
    // Set build date
    let output = std::process::Command::new("date").arg("+%Y-%m-%d").output();
    if let Ok(o) = output {
        let date = String::from_utf8_lossy(&o.stdout).trim().to_string();
        println!("cargo:rustc-env=BUILD_DATE={}", date);
    }

    // Read display version from VERSION file (includes build number)
    if let Ok(ver) = std::fs::read_to_string("../VERSION") {
        let ver = ver.trim();
        if !ver.is_empty() {
            println!("cargo:rustc-env=PARKHUB_VERSION={}", ver);
        }
    }
    println!("cargo:rerun-if-changed=../VERSION");

    // Only compile Slint UI when GUI feature is enabled
    #[cfg(feature = "gui")]
    {
        let config = slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);

        slint_build::compile_with_config("ui/main.slint", config)
            .expect("Slint compilation failed");
    }

    // Windows-specific: embed icon and manifest
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();

        // Set application icon
        if std::path::Path::new("../assets/app.ico").exists() {
            res.set_icon("../assets/app.ico");
        }

        res.set_manifest(
            r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    version="0.1.0.0"
    processorArchitecture="*"
    name="ParkHub.Server"
    type="win32"
  />
  <description>ParkHub Server - Parking Management Backend</description>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>
"#,
        );

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
