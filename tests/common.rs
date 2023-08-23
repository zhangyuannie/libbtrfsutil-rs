use std::{error::Error, fmt, fs, io, path::PathBuf, process::Command};

pub struct LoopDevice {
    path: PathBuf,
    name: String,
    mountpoint: Option<PathBuf>,
}

impl LoopDevice {
    pub fn new(path: PathBuf) -> Self {
        let status = Command::new("truncate")
            .arg("--size=512M")
            .arg(&path)
            .status()
            .unwrap();

        assert!(status.success());

        let output = Command::new("losetup")
            .arg("--show")
            .arg("--find")
            .arg(&path)
            .output()
            .unwrap();

        let name = String::from_utf8(output.stdout).unwrap().trim().to_string();

        Self {
            path,
            name,
            mountpoint: None,
        }
    }

    pub fn mount(&mut self, path: PathBuf) -> io::Result<()> {
        assert!(self.mountpoint.is_none());
        fs::create_dir_all(&path)?;
        let status = Command::new("mount").arg(&self.name).arg(&path).status()?;
        assert!(status.success());
        self.mountpoint.replace(path);
        Ok(())
    }

    pub fn umount(&mut self) -> io::Result<()> {
        let mount_path = self.mountpoint.as_ref().unwrap();
        let status = Command::new("umount").arg(mount_path).status()?;
        assert!(status.success());
        fs::remove_dir_all(mount_path)?;
        self.mountpoint.take();
        Ok(())
    }

    pub fn mountpoint(&self) -> Option<&PathBuf> {
        self.mountpoint.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl Drop for LoopDevice {
    fn drop(&mut self) {
        if self.mountpoint.is_some() {
            self.umount().unwrap();
        }
        let status = Command::new("losetup")
            .arg("--detach")
            .arg(&self.name)
            .status()
            .unwrap();

        assert!(status.success());

        fs::remove_file(&self.path).unwrap();
    }
}

pub trait CommandExt {
    fn call(&mut self) -> Result<String, Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct CommandCallError {
    code: Option<i32>,
    stderr: String,
}
impl fmt::Display for CommandCallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "exitcode: {:?}, stderr: {}", self.code, self.stderr)
    }
}
impl Error for CommandCallError {}

impl CommandExt for Command {
    fn call(&mut self) -> Result<String, Box<dyn Error>> {
        let output = self.output()?;
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err((CommandCallError {
                code: output.status.code(),
                stderr: String::from_utf8(output.stderr)?,
            })
            .into())
        }
    }
}

pub fn setup(device_path: PathBuf, mnt_dir: PathBuf) -> LoopDevice {
    let mut ret = LoopDevice::new(device_path);
    Command::new("mkfs.btrfs").arg(ret.name()).call().unwrap();
    ret.mount(mnt_dir).unwrap();
    ret
}
