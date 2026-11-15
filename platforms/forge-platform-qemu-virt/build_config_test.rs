use super::BuildConfig;

#[test]
fn accepts_comments_whitespace_and_supported_sizes() {
    for size in [1, 256, 4096, 16384] {
        let text = format!("# settings\n\n boot_frame_size = {size} # bytes\r\n");
        assert_eq!(BuildConfig::parse(&text).unwrap().boot_frame_size, size);
    }
}

#[test]
fn requires_the_setting() {
    assert_eq!(
        BuildConfig::parse("# no settings\n"),
        Err("missing boot_frame_size".into())
    );
}

#[test]
fn rejects_unknown_duplicate_and_malformed_settings() {
    for (text, expected) in [
        ("boot_frame_szie = 4096", "unknown setting"),
        (
            "boot_frame_size = 4096\nboot_frame_size = 8192",
            "duplicate",
        ),
        ("boot_frame_size: 4096", "expected name = value"),
        ("boot_frame_size = 4096\nshell = true", "unknown setting"),
    ] {
        assert!(BuildConfig::parse(text).unwrap_err().contains(expected));
    }
}

#[test]
fn rejects_invalid_values() {
    for value in ["", "-1", "+4096", "4k", "4096=1", "0x1000", "\"4096\""] {
        let text = format!("boot_frame_size = {value}");
        assert!(
            BuildConfig::parse(&text)
                .unwrap_err()
                .contains("decimal integer")
        );
    }
    for size in [0, 3, 6144] {
        let text = format!("boot_frame_size = {size}");
        assert!(
            BuildConfig::parse(&text)
                .unwrap_err()
                .contains("power of two")
        );
    }
}

#[test]
fn rejects_sizes_that_do_not_fit_the_platform_arena() {
    assert!(
        BuildConfig::parse("boot_frame_size = 32768")
            .unwrap_err()
            .contains("QEMU boot arena")
    );
    assert!(
        BuildConfig::parse("boot_frame_size = 99999999999999999999999999999999999999")
            .unwrap_err()
            .contains("too large")
    );
}

#[test]
fn identifies_the_error_line() {
    assert!(
        BuildConfig::parse("# header\n\ninvalid")
            .unwrap_err()
            .starts_with("line 3:")
    );
}
