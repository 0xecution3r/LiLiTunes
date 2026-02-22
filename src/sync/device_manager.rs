use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread;
use std::time::Duration;

use crate::legacy_env::LegacyEnv;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Udid(pub String);

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub udid: Udid,
    pub kv: HashMap<String, String>,
}

impl DeviceInfo {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.kv.get(key).map(|s| s.as_str())
    }

    pub fn device_class(&self) -> Option<&str> {
        self.get("DeviceClass")
    }

    pub fn hardware_model(&self) -> Option<&str> {
        self.get("HardwareModel")
    }

    pub fn product_version(&self) -> Option<&str> {
        self.get("ProductVersion")
    }

    pub fn device_name(&self) -> Option<&str> {
        self.get("DeviceName")
    }

    pub fn matches_profile(&self, profile: &DeviceProfile) -> bool {
        profile.matches(self)
    }
}

#[derive(Clone, Debug)]
pub struct DeviceProfile {
    pub device_class: Option<String>,
    pub hardware_model: Option<String>,
    pub product_version: Option<String>,
}

impl DeviceProfile {
    /// Default target: iPod touch 1st gen on iOS 3.1.3 (HardwareModel N45AP).
    pub fn ipod_touch_1g_313() -> Self {
        Self {
            device_class: Some("iPod".to_string()),
            hardware_model: Some("N45AP".to_string()),
            product_version: Some("3.1.3".to_string()),
        }
    }

    pub fn matches(&self, info: &DeviceInfo) -> bool {
        let ok_class = self
            .device_class
            .as_deref()
            .map(|want| info.device_class() == Some(want))
            .unwrap_or(true);

        let ok_model = self
            .hardware_model
            .as_deref()
            .map(|want| info.hardware_model() == Some(want))
            .unwrap_or(true);

        let ok_ver = self
            .product_version
            .as_deref()
            .map(|want| info.product_version() == Some(want))
            .unwrap_or(true);

        ok_class && ok_model && ok_ver
    }
}

#[derive(Clone, Debug)]
pub enum DeviceEvent {
    Connected { udid: Udid },
    Disconnected { udid: Udid },
    /// We successfully read/parsed ideviceinfo -s
    InfoUpdated { info: DeviceInfo },
    /// The device matches your target profile (e.g., iPod touch 1G 3.1.3)
    TargetConfirmed { info: DeviceInfo },
    /// Something went wrong (command failed, parse error, etc.)
    Error { message: String },
}

pub struct DeviceManager {
    stop_flag: Arc<AtomicBool>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl DeviceManager {
    /// Starts a polling device manager in a background thread.
    ///
    /// - `legacy`: your legacy env manager (so we use ~/ios-legacy tools/libs)
    /// - `profile`: what "correct device" means (defaults provided)
    /// - `poll_interval`: e.g. 800ms–1500ms feels good in a UI
    pub fn start(
        legacy: LegacyEnv,
        profile: DeviceProfile,
        poll_interval: Duration,
    ) -> (Self, Receiver<DeviceEvent>) {
        let (tx, rx) = mpsc::channel::<DeviceEvent>();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_thread = stop_flag.clone();

        let handle = thread::spawn(move || {
            // Keep an in-thread cache of last-seen UDIDs so we can emit connect/disconnect.
            let mut last_udids: HashSet<Udid> = HashSet::new();
            // Cache last info so we don't spam UI with the same info constantly.
            let mut last_info: HashMap<Udid, DeviceInfo> = HashMap::new();

            while !stop_flag_thread.load(Ordering::Relaxed) {
                match list_udids(&legacy) {
                    Ok(current_udids) => {
                        let current_set: HashSet<Udid> = current_udids.into_iter().collect();

                        // Newly connected
                        for udid in current_set.difference(&last_udids) {
                            let _ = tx.send(DeviceEvent::Connected { udid: udid.clone() });
                        }

                        // Disconnected
                        for udid in last_udids.difference(&current_set) {
                            let _ = tx.send(DeviceEvent::Disconnected { udid: udid.clone() });
                            last_info.remove(udid);
                        }

                        // Update info for connected devices
                        for udid in current_set.iter() {
                            match read_device_info(&legacy, udid) {
                                Ok(info) => {
                                    let should_emit = match last_info.get(udid) {
                                        Some(prev) => prev.kv != info.kv,
                                        None => true,
                                    };

                                    if should_emit {
                                        last_info.insert(udid.clone(), info.clone());
                                        let _ = tx.send(DeviceEvent::InfoUpdated { info: info.clone() });
                                    }

                                    if info.matches_profile(&profile) {
                                        let _ = tx.send(DeviceEvent::TargetConfirmed { info });
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(DeviceEvent::Error { message: e });
                                }
                            }
                        }

                        last_udids = current_set;
                    }
                    Err(e) => {
                        let _ = tx.send(DeviceEvent::Error { message: e });
                    }
                }

                thread::sleep(poll_interval);
            }
        });

        (
            Self {
                stop_flag,
                thread_handle: Some(handle),
            },
            rx,
        )
    }

    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(h) = self.thread_handle.take() {
            let _ = h.join();
        }
    }
}

/// Uses legacy idevice_id -l to list connected device UDIDs.
fn list_udids(legacy: &LegacyEnv) -> Result<Vec<Udid>, String> {
    let out = legacy
        .command("idevice_id")
        .arg("-l")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("idevice_id failed to run: {e}"))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("idevice_id returned non-zero: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let udids = stdout
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| Udid(s.to_string()))
        .collect::<Vec<_>>();

    Ok(udids)
}

/// Reads ideviceinfo -s for a specific UDID and parses Key: Value lines.
fn read_device_info(legacy: &LegacyEnv, udid: &Udid) -> Result<DeviceInfo, String> {
    let out = legacy
        .command("ideviceinfo")
        .arg("-s")
        .arg("-u")
        .arg(&udid.0)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("ideviceinfo failed to run: {e}"))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("ideviceinfo (-s) non-zero for {}: {stderr}", udid.0));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let kv = parse_colon_kv(&stdout);

    if kv.is_empty() {
        return Err(format!("ideviceinfo (-s) returned no parseable data for {}", udid.0));
    }

    Ok(DeviceInfo {
        udid: udid.clone(),
        kv,
    })
}

/// Parses output like:
///   Key: Value
/// into HashMap<Key, Value>
/// Ignores lines without ':'.
fn parse_colon_kv(s: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let key = k.trim();
        let val = v.trim();
        if !key.is_empty() {
            map.insert(key.to_string(), val.to_string());
        }
    }

    map
}
