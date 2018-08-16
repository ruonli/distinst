use std::collections::HashSet;
use std::io::{self, BufRead};
use std::process::Command;
use disk::{Bootloader, FileSystemSupport};
use os_release::OsRelease;

pub fn check_language_support(locale: &str) -> io::Result<Option<Vec<u8>>> {
    // Attempt to run the check-language-support external command.
    let check_language_support = Command::new("check-language-support")
        .args(&["-l", locale, "--show-installed"])
        .output();

    // If the command executed, get the standard output.
    let output = match check_language_support {
        Ok(output) => Some(output.stdout),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(why) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("failed to spawn check-language-support: {}", why)
            ));
        }
    };

    Ok(output)
}

// This is a hack to work around issues with Ubuntu's manifest-remove file.
// This will get the immediate dependencies of the given packages.
pub fn get_dependencies_from_list<P: AsRef<str>>(deps: &[P]) -> Option<Vec<String>> {
    if deps.is_empty() {
        return None;
    }

    let mut outer = HashSet::new();

    {
        let outer = &mut outer;
        for dep in deps {
            get_dependencies_from_package(dep, |dep| {
                let dep = dep.to_owned();
                if !outer.contains(&dep) {
                    outer.insert(dep);
                }
            });
        }

        for dep in deps {
            outer.insert(dep.as_ref().to_owned());
        }
    }

    Some(outer.into_iter().collect())
}

fn get_dependencies_from_package<A: FnMut(&str), P: AsRef<str>>(dep: P, mut action: A) {
    let output = Command::new("apt-cache")
        .args(&["show", dep.as_ref()])
        .output()
        .ok();

    if let Some(output) = output {
        for line in io::Cursor::new(output.stdout).lines() {
            if let Ok(line) = line {
                if ! line.starts_with("Depends:") {
                    continue
                }

                parse_dependency_line(
                    line[8..].trim(),
                    |dep| action(dep)
                );
            }
        }
    }
}

fn parse_dependency_line<F: FnMut(&str)>(line: &str, mut func: F) {
    if line.is_empty() {
        return;
    }

    for dep in line.split(',').filter_map(|dep| dep.split_whitespace().next()) {
        func(dep);
    }
}

pub fn get_bootloader_packages(os_release: &OsRelease) -> &'static [&'static str] {
    match Bootloader::detect() {
        Bootloader::Bios => &["grub", "grub-pc"],
        Bootloader::Efi if os_release.name == "Pop!_OS" => &["kernelstub"],
        Bootloader::Efi => {
            &["grub-efi", "grub-efi-amd64", "grub-efi-amd64-signed",
              "shim-signed", "mokutil", "fwupdate-signed", "linux-signed-generic"]
        }
    }
}

pub fn get_required_packages(flags: FileSystemSupport) -> Vec<&'static str> {
    let mut retain = Vec::new();

    if flags.contains(FileSystemSupport::BTRFS) {
        retain.extend_from_slice(&["btrfs-progs", "btrfs-tools"]);
    }

    if flags.contains(FileSystemSupport::EXT4) {
        retain.push("e2fsprogs");
    }

    if flags.contains(FileSystemSupport::F2FS) {
        retain.push("f2fs-tools");
    }

    if flags.contains(FileSystemSupport::FAT) {
        retain.push("dosfstools");
    }

    if flags.contains(FileSystemSupport::NTFS) {
        retain.push("ntfs-3g");
    }

    if flags.contains(FileSystemSupport::XFS) {
        retain.push("xfsprogs");
    }

    if flags.contains(FileSystemSupport::LUKS) {
        retain.extend_from_slice(&["cryptsetup", "cryptsetup-bin"]);
    }

    if flags.intersects(FileSystemSupport::LVM | FileSystemSupport::LUKS) {
        retain.extend_from_slice(&["lvm2", "dmeventd", "dmraid"]);
    }

    retain
}
