//! Host-side parser for forge.cfg. No configuration parser runs in the kernel.

#[derive(Debug, PartialEq, Eq)]
pub struct BuildConfig {
    pub boot_frame_size: usize,
}

impl BuildConfig {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut boot_frame_size = None;
        for (index, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap().trim();
            if line.is_empty() {
                continue;
            }
            let error = |message: &str| format!("line {}: {message}", index + 1);
            let (name, value) = line
                .split_once('=')
                .ok_or_else(|| error("expected name = value"))?;
            if name.trim() != "boot_frame_size" {
                return Err(error("unknown setting; expected boot_frame_size"));
            }
            if boot_frame_size.is_some() {
                return Err(error("duplicate boot_frame_size"));
            }
            let value = value.trim();
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(error("boot_frame_size must be a decimal integer"));
            }
            let size: usize = value
                .parse()
                .map_err(|_| error("boot_frame_size is too large"))?;
            if !size.is_power_of_two() {
                return Err(error("boot_frame_size must be a nonzero power of two"));
            }
            // linker.ld reserves 64 KiB. Allow at least two whole frames after
            // inward alignment, including the worst case for the arena address.
            if size > 16384 {
                return Err(error(
                    "boot_frame_size must be <= 16384 for the QEMU boot arena",
                ));
            }
            boot_frame_size = Some(size);
        }
        Ok(Self {
            boot_frame_size: boot_frame_size.ok_or("missing boot_frame_size")?,
        })
    }
}

#[cfg(test)]
#[path = "build_config_test.rs"]
mod tests;
