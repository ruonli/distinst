use std::ffi::OsStr;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use mount::{Mount, MountKind};

pub struct Chroot {
    path: PathBuf,
    dev_mount: Mount,
    proc_mount: Mount,
    sys_mount: Mount,
}

impl Chroot {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Chroot> {
        let path = path.as_ref().canonicalize()?;
        let dev_mount = Mount::new("/dev", path.join("dev"), MountKind::Bind)?;
        let proc_mount = Mount::new("/proc", path.join("proc"), MountKind::Bind)?;
        let sys_mount = Mount::new("/sys", path.join("sys"), MountKind::Bind)?;
        Ok(Chroot {
            path: path,
            dev_mount: dev_mount,
            proc_mount: proc_mount,
            sys_mount: sys_mount,
        })
    }

    pub fn command<S: AsRef<OsStr>, T: AsRef<OsStr>, I: IntoIterator<Item=T>>(&mut self, cmd: S, args: I) -> Result<ExitStatus> {
        let mut command = Command::new("chroot");
        command.arg(&self.path);
        command.arg(cmd.as_ref());
        for arg in args.into_iter() {
            command.arg(arg);
        }
        command.status()
    }

    /// Return true if the filesystem was unmounted, false if it was already unmounted
    pub fn unmount(&mut self, lazy: bool) -> Result<()> {
        self.dev_mount.unmount(lazy)?;
        self.proc_mount.unmount(lazy)?;
        self.sys_mount.unmount(lazy)?;
        Ok(())
    }
}

impl Drop for Chroot {
    fn drop(&mut self) {
        println!("Chroot drop");

        // Ensure unmounting
        let _ = self.dev_mount.unmount(true);
        let _ = self.proc_mount.unmount(true);
        let _ = self.sys_mount.unmount(true);
    }
}