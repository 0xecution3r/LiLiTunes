use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub struct LegacyEnv {
    pub prefix: PathBuf,
    pub bin: PathBuf,
    pub lib: PathBuf,
    pub lib64: PathBuf,
    pub pkgconfig: Vec<PathBuf>,
}

impl LegacyEnv {
    /// Build legacy env rooted at "$HOME/ios-legacy" by default.
    pub fn from_home_default() -> Result<Self, String> {
        let home = env::var_os("HOME").ok_or("HOME is not set")?;
        let home = PathBuf::from(home);
        Self::from_prefix(home.join("ios-legacy"))
    }

    /// Build legacy env from an explicit prefix.
    pub fn from_prefix(prefix: PathBuf) -> Result<Self, String> {
        if !prefix.exists() {
            return Err(format!("Legacy prefix does not exist: {}", prefix.display()));
        }

        let bin = prefix.join("bin");
        let lib = prefix.join("lib");
        let lib64 = prefix.join("lib64");

        Ok(Self {
            prefix,
            bin,
            lib,
            lib64,
            pkgconfig: vec![
                // runtime doesn't really need this, but harmless and useful if you spawn build helpers
                PathBuf::from("lib/pkgconfig"),
                PathBuf::from("share/pkgconfig"),
            ]
            .into_iter()
            .map(|p| prefix.join(p))
            .collect(),
        })
    }

    /// Apply legacy variables to the *current process* so all spawned children inherit them.
    pub fn apply_to_process(&self) {
        env::set_var("PREFIX", &self.prefix);

        // Prefer legacy binaries first
        prepend_env_path("PATH", &self.bin);

        // Force runtime linker to use legacy OpenSSL/libs
        // Order matters: lib then lib64, both before existing.
        prepend_env_path("LD_LIBRARY_PATH", &self.lib64);
        prepend_env_path("LD_LIBRARY_PATH", &self.lib);

        // Usually not needed at runtime, but you asked for parity with your env script
        for pc in self.pkgconfig.iter().rev() {
            prepend_env_path("PKG_CONFIG_PATH", pc);
        }
    }

    /// Create a Command pre-wired to run under this legacy env (even if you *don't* apply globally).
    pub fn command<S: AsRef<std::ffi::OsStr>>(&self, program: S) -> Command {
        let mut cmd = Command::new(program);

        // Mirror apply_to_process(), but only for this child process:
        cmd.env("PREFIX", &self.prefix);

        cmd.env("PATH", build_prepended_env("PATH", &self.bin));
        cmd.env(
            "LD_LIBRARY_PATH",
            build_prepended_env_multi("LD_LIBRARY_PATH", [&self.lib, &self.lib64]),
        );

        // Optional but included
        cmd.env(
            "PKG_CONFIG_PATH",
            build_prepended_env_multi("PKG_CONFIG_PATH", self.pkgconfig.iter().map(|p| p).collect::<Vec<_>>()),
        );

        cmd
    }
}

/// Prepend a single path to an environment variable in the current process.
fn prepend_env_path(var: &str, path: &Path) {
    let new_val = build_prepended_env(var, path);
    env::set_var(var, new_val);
}

/// Build "path:old" for var, using existing value if present.
fn build_prepended_env(var: &str, path: &Path) -> OsString {
    let mut out = OsString::new();
    out.push(path.as_os_str());

    if let Some(old) = env::var_os(var) {
        if !old.is_empty() {
            out.push(OsString::from(":"));
            out.push(old);
        }
    }
    out
}

/// Build "p1:p2:old" for var, in the order provided.
fn build_prepended_env_multi<'a, I>(var: &str, paths: I) -> OsString
where
    I: IntoIterator<Item = &'a PathBuf>,
{
    let mut out = OsString::new();
    let mut first = true;

    for p in paths {
        if first {
            out.push(p.as_os_str());
            first = false;
        } else {
            out.push(OsString::from(":"));
            out.push(p.as_os_str());
        }
    }

    if let Some(old) = env::var_os(var) {
        if !old.is_empty() {
            out.push(OsString::from(":"));
            out.push(old);
        }
    }

    out
}
