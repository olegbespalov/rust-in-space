use super::*;
use std::{ffi::CString, os::unix::ffi::OsStrExt};

#[cfg(target_arch = "aarch64")]
pub(super) const PACKAGE: &str = "aarch64-apple-darwin.zip";
#[cfg(target_arch = "x86_64")]
pub(super) const PACKAGE: &str = "x86_64-apple-darwin.zip";
const PUBLIC_KEY: Option<&str> = option_env!("RUST_IN_SPACE_UPDATE_PUBLIC_KEY");
const BUNDLE_ID: &str = "io.github.olegbespalov.RustInSpace";

pub(super) fn installation() -> Option<Installation> {
    let executable = env::current_exe().ok()?.canonicalize().ok()?;
    let bundle = crate::resources::macos_bundle_resources(&executable)?
        .parent()?
        .parent()?
        .to_owned();
    if bundle
        .components()
        .any(|part| part.as_os_str() == "AppTranslocation")
    {
        return None;
    }
    Installation::new(bundle, executable).ok()
}

pub(super) fn can_install(release: &Release) -> bool {
    PUBLIC_KEY.is_some_and(|key| !key.is_empty())
        && release
            .signature
            .as_ref()
            .is_some_and(|asset| asset.valid() && asset.size <= 1024)
}

pub(super) fn prepare(release: &Release, payload: &Path, staging: &Path) -> io::Result<PathBuf> {
    let signature = staging.join("signature");
    let public_key = staging.join("public.pem");
    release
        .signature
        .as_ref()
        .ok_or_else(invalid_update)?
        .download(&release.tag, &signature)?;
    fs::write(&public_key, PUBLIC_KEY.ok_or_else(invalid_update)?)?;
    checked(
        Command::new("/usr/bin/openssl")
            .args(["dgst", "-sha256", "-verify"])
            .arg(public_key)
            .arg("-signature")
            .arg(signature)
            .arg(payload),
    )?;

    let extracted = staging.join("extracted");
    checked(
        Command::new("/usr/bin/ditto")
            .args(["-x", "-k"])
            .arg(payload)
            .arg(&extracted),
    )?;
    let app = extracted.join("Rust in Space.app");
    if !app.symlink_metadata()?.is_dir() {
        return Err(invalid_update());
    }
    let plist = app.join("Contents/Info.plist");
    let expected_version = release
        .tag
        .trim_start_matches('v')
        .split('+')
        .next()
        .unwrap_or_default();
    if plist_field(&plist, "CFBundleIdentifier")? != BUNDLE_ID
        || plist_field(&plist, "CFBundleShortVersionString")? != expected_version
    {
        return Err(invalid_update());
    }
    checked(
        Command::new("/usr/bin/codesign")
            .args(["--verify", "--strict"])
            .arg(&app),
    )?;
    Ok(app)
}

fn plist_field(plist: &Path, field: &str) -> io::Result<String> {
    let output = checked(
        Command::new("/usr/bin/plutil")
            .args(["-extract", field, "raw", "-o", "-"])
            .arg(plist),
    )?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub(super) fn replace(source: &Path, destination: &Path) -> io::Result<()> {
    let source = CString::new(source.as_os_str().as_bytes())?;
    let destination = CString::new(destination.as_os_str().as_bytes())?;
    // RENAME_SWAP keeps a complete bundle at the original path even if the update fails.
    let result =
        unsafe { libc::renamex_np(source.as_ptr(), destination.as_ptr(), libc::RENAME_SWAP) };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub(super) fn sha256() -> Command {
    let mut command = Command::new("/usr/bin/shasum");
    command.args(["-a", "256"]);
    command
}

pub(super) fn open_page(url: &str) -> io::Result<()> {
    checked(Command::new("/usr/bin/open").arg(url)).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomically_swaps_app_bundles() -> io::Result<()> {
        let staging = Staging::new(&env::temp_dir())?;
        let installed = staging.0.join("Installed.app");
        let replacement = staging.0.join("Replacement.app");
        fs::create_dir(&installed)?;
        fs::create_dir(&replacement)?;
        fs::write(installed.join("version"), b"old")?;
        fs::write(replacement.join("version"), b"new")?;

        replace(&replacement, &installed)?;

        assert_eq!(fs::read(installed.join("version"))?, b"new");
        assert_eq!(fs::read(replacement.join("version"))?, b"old");
        Ok(())
    }
}
