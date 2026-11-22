use super::*;
use std::{io::Read, os::unix::fs::PermissionsExt, process::Stdio};

pub(super) const PACKAGE: &str = "x86_64-unknown-linux-gnu.AppImage";

pub(super) fn installation() -> Option<Installation> {
    let mount = PathBuf::from(env::var_os("APPDIR")?).canonicalize().ok()?;
    if env::current_exe().ok()? != mount.join("usr/bin/rust-in-space") {
        return None;
    }
    let path = PathBuf::from(env::var_os("APPIMAGE")?)
        .canonicalize()
        .ok()?;
    verify_appimage(&path).ok()?;
    Installation::new(path.clone(), path).ok()
}

pub(super) fn can_install(_release: &Release) -> bool {
    true
}

pub(super) fn prepare(_release: &Release, payload: &Path, _staging: &Path) -> io::Result<PathBuf> {
    verify_appimage(payload)?;
    Ok(payload.to_owned())
}

fn verify_appimage(path: &Path) -> io::Result<()> {
    let mut header = [0; 11];
    fs::File::open(path)?.read_exact(&mut header)?;
    if &header[..4] == b"\x7fELF" && &header[8..] == b"AI\x02" {
        Ok(())
    } else {
        Err(invalid_update())
    }
}

pub(super) fn replace(source: &Path, destination: &Path) -> io::Result<()> {
    let mode = fs::metadata(destination)?.permissions().mode() & 0o777;
    fs::set_permissions(source, fs::Permissions::from_mode(mode))?;
    fs::rename(source, destination)
}

pub(super) fn sha256() -> Command {
    Command::new("/usr/bin/sha256sum")
}

pub(super) fn open_page(url: &str) -> io::Result<()> {
    // The desktop launcher may wait until the browser closes.
    let mut child = Command::new("xdg-open")
        .arg(url)
        .stdin(Stdio::null())
        .spawn()?;
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_only_a_prepared_appimage_and_detects_stale_installations() -> io::Result<()> {
        let staging = Staging::new(&env::temp_dir())?;
        let installed = staging.0.join("Game.AppImage");
        let download = staging.0.join("download");
        let previous = b"\x7fELF\0\0\0\0AI\x02old";
        let next = b"\x7fELF\0\0\0\0AI\x02new";
        fs::write(&installed, previous)?;
        fs::set_permissions(&installed, fs::Permissions::from_mode(0o700))?;
        fs::write(&download, b"invalid data")?;

        let target = Installation::new(installed.clone(), installed.clone())?;
        let release = Release {
            tag: "v0.6.0".to_owned(),
            package: Asset {
                name: String::new(),
                size: 0,
                digest: None,
            },
            installation: None,
        };
        assert!(prepare(&release, &download, &staging.0).is_err());
        assert_eq!(fs::read(&installed)?, previous);

        fs::write(&download, next)?;
        let replacement = prepare(&release, &download, &staging.0)?;
        replace(&replacement, &installed)?;
        assert_eq!(fs::read(&installed)?, next);
        assert_eq!(
            fs::metadata(&installed)?.permissions().mode() & 0o777,
            0o700
        );
        assert!(target.verify_identity(&fs::metadata(&installed)?).is_err());
        Ok(())
    }
}
