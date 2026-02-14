use std::path::PathBuf;
use std::process::Command;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let path = find_or_download_slangc()?;

    println!("cargo:rustc-env=SLANG_DIR={}", path.display());
    // println!("cargo:warning=Found slangc at: {}", path.display());
    Ok(())
}

fn find_or_download_slangc() -> anyhow::Result<PathBuf> {
    if let Ok(path) = std::env::var("SLANG_DIR") {
        let path = PathBuf::from(path);
        let exe_path = path.join("bin").join(if cfg!(target_os = "windows") {
            "slangc.exe"
        } else {
            "slangc"
        });
        if is_valid_slangc(&exe_path)? {
            return Ok(path);
        } else {
            anyhow::bail!("Not valid slangc installation at: {}", path.display());
        }
    }

    if let Some(exe_path) = find_in_path("slangc") {
        if is_valid_slangc(&exe_path)? {
            if let Some(root) = exe_path.parent().and_then(|p| p.parent()) {
                return Ok(root.to_path_buf());
            }
        }
    }

    if let Some(exe_path) = check_cached_download() {
        if is_valid_slangc(&exe_path)? {
            if let Some(root) = exe_path.parent().and_then(|p| p.parent()) {
                return Ok(root.to_path_buf());
            }
        }
    }

    #[cfg(feature = "download-slang")]
    {
        return download_slang();
    }

    #[cfg(not(feature = "download-slang"))]
    {
        Err(anyhow::anyhow!(
            "slangc not found. Either:\n\
             1. Install slangc and add it to PATH\n\
             2. Set SLANG_DIR environment variable\n\
             3. Enable the 'download-slang' feature"
        ))
    }
}

fn is_valid_slangc(path: &PathBuf) -> anyhow::Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    Ok(Command::new(path)
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false))
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(name);
                #[cfg(target_os = "windows")]
                {
                    if full_path.with_extension("exe").exists() {
                        return Some(full_path.with_extension("exe"));
                    }
                }
                if full_path.exists() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

fn check_cached_download() -> Option<PathBuf> {
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let cached = PathBuf::from(out_dir)
            .join("slangc")
            .join("bin")
            .join(if cfg!(target_os = "windows") {
                "slangc.exe"
            } else {
                "slangc"
            });

        if cached.exists() {
            return Some(cached);
        }
    }

    if let Some(cache_dir) = dirs::cache_dir() {
        let cached = cache_dir
            .join("slank")
            .join("slangc")
            .join("bin")
            .join(if cfg!(target_os = "windows") {
                "slangc.exe"
            } else {
                "slangc"
            });

        if cached.exists() {
            return Some(cached);
        }
    }

    None
}

#[cfg(feature = "download-slang")]
fn download_slang() -> anyhow::Result<PathBuf> {

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let version = get_latest_slang_version()?;

    let (download_url, archive_name) = get_download_url(&version, &target_os, &target_arch)?;

    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
        .join("slank")
        .join("slangc");

    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create cache directory: {}", e))?;

    let archive_path = cache_dir.join(&archive_name);

    if !archive_path.exists() {
        download_file(&download_url, &archive_path)?;
    }

    extract_archive(&archive_path, &cache_dir)?;

    let slangc_exe = cache_dir
        .join("bin")
        .join(if cfg!(target_os = "windows") {
            "slangc.exe"
        } else {
            "slangc"
        });

    if !slangc_exe.exists() {
        return Err(anyhow::anyhow!("slangc not found after extraction at: {}", slangc_exe.display()));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&slangc_exe)
            .map_err(|e| anyhow::anyhow!("Failed to get permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&slangc_exe, perms)
            .map_err(|e| anyhow::anyhow!("Failed to set permissions: {}", e))?;
    }

    Ok(cache_dir)
}

#[cfg(feature = "download-slang")]
fn get_latest_slang_version() -> anyhow::Result<String> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
    }

    let client = reqwest::blocking::Client::new();
    let mut request = client
        .get("https://api.github.com/repos/shader-slang/slang/releases/latest")
        .header(reqwest::header::USER_AGENT, "dropbear-slank-build");

    // Add GitHub token if available for authenticated requests (avoids rate limiting)
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token));
    }

    let response = request
        .send()
        .map_err(|e| anyhow::anyhow!("Failed to fetch latest version from GitHub: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "GitHub API failed with status: {}. If rate limited, set GITHUB_TOKEN environment variable.",
            response.status()
        ));
    }

    let release: Release = response.json()
        .map_err(|e| anyhow::anyhow!("Failed to parse GitHub response: {}", e))?;

    Ok(release.tag_name.trim_start_matches('v').to_string())
}

#[cfg(feature = "download-slang")]
fn get_download_url(version: &str, os: &str, arch: &str) -> anyhow::Result<(String, String)> {
    let (platform, ext) = match (os, arch) {
        ("windows", "x86_64") => ("windows-x86_64", "zip"),
        ("windows", "aarch64") => ("windows-aarch64", "zip"),
        ("linux", "x86_64") => ("linux-x86_64", "tar.gz"),
        ("linux", "aarch64") => ("linux-aarch64", "tar.gz"),
        ("macos", "x86_64") => ("macos-x86_64", "zip"),
        ("macos", "aarch64") => ("macos-aarch64", "zip"),
        _ => return Err(anyhow::anyhow!("Unsupported platform: {}-{}", os, arch)),
    };

    let archive_name = format!("slang-{}-{}.{}", version, platform, ext);
    let url = format!(
        "https://github.com/shader-slang/slang/releases/download/v{}/{}",
        version, archive_name
    );

    Ok((url, archive_name))
}

#[cfg(feature = "download-slang")]
fn download_file(url: &str, dest: &PathBuf) -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

    let mut response = client.get(url)
        .header(reqwest::header::USER_AGENT, "dropbear-slank-build")
        .send()
        .map_err(|e| anyhow::anyhow!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Download failed with status: {}", response.status()));
    }

    let mut file = std::fs::File::create(dest)
        .map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;

    response.copy_to(&mut file)
        .map_err(|e| anyhow::anyhow!("Failed to save file content at {}: {}", dest.display(), e))?;

    Ok(())
}

#[cfg(feature = "download-slang")]
fn extract_archive(archive: &PathBuf, dest: &PathBuf) -> anyhow::Result<()> {
    let file = std::fs::File::open(archive)
        .map_err(|e| anyhow::anyhow!("Failed to open archive: {}", e))?;

    if archive.extension().and_then(|s| s.to_str()) == Some("zip") {
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| anyhow::anyhow!("Failed to read zip: {}", e))?;

        archive.extract(dest)
            .map_err(|e| anyhow::anyhow!("Failed to extract zip: {}", e))?;
    } else {
        let tar = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar);

        archive.unpack(dest)
            .map_err(|e| anyhow::anyhow!("Failed to extract tar.gz: {}", e))?;
    }

    Ok(())
}