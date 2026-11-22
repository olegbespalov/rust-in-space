#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform;
#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod platform;

use serde::Deserialize;
use std::{
    env, fs, io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

const REPOSITORY: &str = "https://github.com/olegbespalov/rust-in-space";
const LATEST_RELEASE: &str =
    "https://api.github.com/repos/olegbespalov/rust-in-space/releases/latest";
const MAX_PACKAGE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    size: u64,
    digest: Option<String>,
}

pub struct Release {
    pub tag: String,
    package: Asset,
    installation: Option<Installation>,
    #[cfg(target_os = "macos")]
    signature: Option<Asset>,
}

impl Release {
    pub fn can_install(&self) -> bool {
        self.installation.is_some() && self.package.valid() && platform::can_install(self)
    }

    fn open_page(&self) -> io::Result<()> {
        platform::open_page(&format!("{REPOSITORY}/releases/tag/{}", self.tag))
    }

    fn install(&self) -> io::Result<()> {
        let target = self.installation.as_ref().ok_or_else(invalid_update)?;
        let guard = fs::File::open(&target.identity_path)?;
        guard.try_lock().map_err(io::Error::other)?;
        target.verify_identity(&guard.metadata()?)?;
        let parent = target.path.parent().ok_or_else(invalid_update)?;
        let staging = Staging::new(parent)?;
        let payload = staging.0.join("download");
        self.package.download(&self.tag, &payload)?;
        let replacement = platform::prepare(self, &payload, &staging.0)?;
        target.verify_identity(&fs::metadata(&target.identity_path)?)?;
        platform::replace(&replacement, &target.path)
    }
}

impl Asset {
    fn digest(&self) -> Option<&str> {
        self.digest
            .as_deref()
            .and_then(|digest| digest.strip_prefix("sha256:"))
            .filter(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
    }

    fn valid(&self) -> bool {
        (1..=MAX_PACKAGE_BYTES).contains(&self.size) && self.digest().is_some()
    }

    fn download(&self, tag: &str, destination: &Path) -> io::Result<()> {
        if !self.valid() {
            return Err(invalid_update());
        }
        let url = format!("{REPOSITORY}/releases/download/{tag}/{}", self.name);
        checked(
            curl(self.size, 300)
                .arg("--output")
                .arg(destination)
                .arg(url),
        )?;
        self.verify(destination)?;
        fs::OpenOptions::new()
            .write(true)
            .open(destination)?
            .sync_all()
    }

    fn verify(&self, path: &Path) -> io::Result<()> {
        let expected = self.digest().ok_or_else(invalid_update)?;
        let file = fs::File::open(path)?;
        if file.metadata()?.len() != self.size {
            return Err(invalid_update());
        }
        let output = checked(platform::sha256().stdin(file))?;
        let output = String::from_utf8_lossy(&output.stdout);
        let hash = output
            .split_whitespace()
            .next()
            .ok_or_else(invalid_update)?;
        if !hash.eq_ignore_ascii_case(expected) {
            return Err(invalid_update());
        }
        Ok(())
    }
}

pub enum Update {
    Hidden,
    Ready,
    Checking(Receiver<io::Result<Option<Release>>>),
    Checked { failed: bool, retry_at: Instant },
    Available(Release),
    Installing(Receiver<(Release, io::Result<()>)>),
    Failed(Release),
    Installed,
}

impl Update {
    pub fn new(dev_mode: bool) -> Self {
        if dev_mode || option_env!("RUST_IN_SPACE_RELEASE_TAG").is_none() {
            Self::Hidden
        } else {
            Self::Ready
        }
    }

    pub fn poll(&mut self) {
        match self {
            Self::Checking(receive) => match receive.try_recv() {
                Ok(Ok(Some(release))) => *self = Self::Available(release),
                Ok(Err(error)) => {
                    eprintln!("Update check failed: {error}");
                    *self = Self::checked(true);
                }
                Ok(Ok(None)) => *self = Self::checked(false),
                Err(TryRecvError::Disconnected) => *self = Self::checked(true),
                Err(TryRecvError::Empty) => {}
            },
            Self::Checked { retry_at, .. } if Instant::now() >= *retry_at => *self = Self::Ready,
            Self::Installing(receive) => match receive.try_recv() {
                Ok((_, Ok(()))) => *self = Self::Installed,
                Ok((release, Err(error))) => {
                    eprintln!("Update installation failed: {error}");
                    *self = Self::Failed(release);
                }
                Err(TryRecvError::Disconnected) => *self = Self::Hidden,
                Err(TryRecvError::Empty) => {}
            },
            _ => {}
        }
    }

    fn checked(failed: bool) -> Self {
        Self::Checked {
            failed,
            retry_at: Instant::now() + Duration::from_secs(60),
        }
    }

    pub fn blocks_menu(&self) -> bool {
        matches!(self, Self::Installing(_) | Self::Installed)
    }

    pub fn activate(&mut self) {
        *self = match std::mem::replace(self, Self::Hidden) {
            Self::Ready => {
                let (send, receive) = mpsc::channel();
                thread::spawn(move || {
                    let _ = send.send(check_for_update(platform::installation()));
                });
                Self::Checking(receive)
            }
            Self::Available(release) if release.can_install() => {
                let (send, receive) = mpsc::channel();
                thread::spawn(move || {
                    let result = release.install();
                    let _ = send.send((release, result));
                });
                Self::Installing(receive)
            }
            state @ (Self::Available(_) | Self::Failed(_)) => {
                if let Self::Available(release) | Self::Failed(release) = &state {
                    if let Err(error) = release.open_page() {
                        eprintln!("Could not open the release page: {error}");
                    }
                }
                state
            }
            state => state,
        };
    }
}

fn check_for_update(installation: Option<Installation>) -> io::Result<Option<Release>> {
    let output = checked(
        curl(1024 * 1024, 15)
            .args(["--header", "Accept: application/vnd.github+json"])
            .arg(LATEST_RELEASE),
    )?;
    let release = serde_json::from_slice::<GitHubRelease>(&output.stdout)?;
    Ok(select_release(release, crate::VERSION, installation))
}

fn select_release(
    release: GitHubRelease,
    current: &str,
    installation: Option<Installation>,
) -> Option<Release> {
    let latest = version(&release.tag_name)?;
    if release.draft || release.prerelease || !latest.1 || latest <= version(current)? {
        return None;
    }
    let package_name = format!("rust-in-space-{}-{}", release.tag_name, platform::PACKAGE);
    #[cfg(target_os = "macos")]
    let signature_name = format!("{package_name}.sig");
    #[cfg(target_os = "macos")]
    let mut signature = None;
    let mut package = None;
    for asset in release.assets {
        if asset.name == package_name {
            package = Some(asset);
            continue;
        }
        #[cfg(target_os = "macos")]
        if asset.name == signature_name {
            signature = Some(asset);
        }
    }
    Some(Release {
        tag: release.tag_name,
        package: package?,
        installation,
        #[cfg(target_os = "macos")]
        signature,
    })
}

fn version(tag: &str) -> Option<([u32; 3], bool)> {
    let tag = tag.strip_prefix('v')?;
    if !tag
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b".-+".contains(&byte))
    {
        return None;
    }
    let version = tag.split('+').next()?;
    let stable = !version.contains('-');
    let mut parts = version.split('-').next()?.split('.');
    let numbers = [
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ];
    parts.next().is_none().then_some((numbers, stable))
}

fn curl(max_bytes: u64, timeout: u64) -> Command {
    let mut command = Command::new("/usr/bin/curl");
    command
        .args([
            "--disable",
            "--silent",
            "--show-error",
            "--fail",
            "--location",
            "--max-redirs",
            "5",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--connect-timeout",
            "8",
            "--user-agent",
            "Rust-in-Space-Updater",
            "--max-time",
        ])
        .arg(timeout.to_string())
        .arg("--max-filesize")
        .arg(max_bytes.to_string());
    command
}

fn checked(command: &mut Command) -> io::Result<Output> {
    let output = command.output()?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(io::Error::other(format!(
            "{:?} failed ({}) {}",
            command.get_program(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn invalid_update() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Unexpected update data or installation path",
    )
}

struct Installation {
    path: PathBuf,
    identity_path: PathBuf,
    device: u64,
    inode: u64,
}

impl Installation {
    fn new(path: PathBuf, identity_path: PathBuf) -> io::Result<Self> {
        let metadata = fs::metadata(&identity_path)?;
        if !path.is_absolute() || !metadata.is_file() {
            return Err(invalid_update());
        }
        Ok(Self {
            path,
            identity_path,
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }

    fn verify_identity(&self, metadata: &fs::Metadata) -> io::Result<()> {
        if metadata.dev() == self.device && metadata.ino() == self.inode {
            Ok(())
        } else {
            Err(io::Error::other("The installed game has changed"))
        }
    }
}

struct Staging(PathBuf);

impl Staging {
    fn new(parent: &Path) -> io::Result<Self> {
        use std::os::unix::ffi::OsStringExt;

        let output = checked(
            Command::new("/usr/bin/mktemp")
                .arg("-d")
                .arg(parent.join(".rust-in-space-update.XXXXXXXX")),
        )?;
        let path = output
            .stdout
            .strip_suffix(b"\n")
            .ok_or_else(invalid_update)?;
        Ok(Self(PathBuf::from(std::ffi::OsString::from_vec(
            path.to_vec(),
        ))))
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn github_release(tag: &str, prerelease: bool, package: &str) -> GitHubRelease {
        GitHubRelease {
            tag_name: tag.to_owned(),
            draft: false,
            prerelease,
            assets: vec![Asset {
                name: format!("rust-in-space-{tag}-{package}"),
                size: 3,
                digest: None,
            }],
        }
    }

    #[test]
    fn selects_only_newer_stable_releases_with_a_matching_package() {
        let cases = [
            ("v0.5.9", "v0.5.10", false, platform::PACKAGE, false),
            ("v0.5.10", "v0.5.9", false, platform::PACKAGE, true),
            ("v0.5.10", "v0.5.10", false, platform::PACKAGE, false),
            ("v0.5.10", "v0.5.10-rc.1", false, platform::PACKAGE, true),
            ("v0.5.10-rc.1", "v0.5.9", true, platform::PACKAGE, false),
            ("v0.5.10", "v0.5.9", true, platform::PACKAGE, false),
            ("v0.5.10", "v0.5.9", false, "wrong-target.zip", false),
            (
                "v0.5.10+bad/path",
                "v0.5.9",
                false,
                platform::PACKAGE,
                false,
            ),
        ];
        for (latest, current, prerelease, package, found) in cases {
            let release = github_release(latest, prerelease, package);
            assert_eq!(select_release(release, current, None).is_some(), found);
        }
    }

    #[test]
    fn verifies_both_asset_size_and_digest() -> io::Result<()> {
        let staging = Staging::new(&env::temp_dir())?;
        let payload = staging.0.join("payload");
        let asset = Asset {
            name: "payload".to_owned(),
            size: 3,
            digest: Some(
                "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
                    .to_owned(),
            ),
        };

        fs::write(&payload, b"abc")?;
        asset.verify(&payload)?;
        fs::write(&payload, b"xyz")?;
        assert!(asset.verify(&payload).is_err());
        fs::write(&payload, b"abcd")?;
        assert!(asset.verify(&payload).is_err());
        Ok(())
    }
}
