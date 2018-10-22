use std::process::{self, Stdio};
use std::io::{self, BufRead, BufReader, Error, ErrorKind};
use std::ffi::OsStr;
use std::thread;

pub struct Command(process::Command);

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Command {
        Command(process::Command::new(program))
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Command {
        self.0.arg(arg);
        self
    }

    pub fn args<S: AsRef<OsStr>, I: IntoIterator<Item = S>>(&mut self, args: I) -> &mut Command {
        self.0.args(args);
        self
    }

    pub fn env(&mut self, key: &str, value: &str) {
        self.0.env(key, value);
    }

    pub fn env_clear(&mut self) {
        self.0.env_clear();
    }

    pub fn stdin(&mut self, stdio: Stdio) { self.0.stdin(stdio); }
    pub fn stderr(&mut self, stdio: Stdio) { self.0.stderr(stdio); }
    pub fn stdout(&mut self, stdio: Stdio) { self.0.stdout(stdio); }

    fn redirect<R: io::Read + Send + 'static, F: FnMut(&str) + Send + 'static>(
        reader: Option<R>,
        mut writer: F
    ) {
        if let Some(reader) = reader {
            let mut reader = BufReader::new(reader);
            thread::spawn(move || {
                let buffer = &mut String::with_capacity(8 * 1024);
                loop {
                    buffer.clear();
                    match reader.read_line(buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => writer(buffer.trim_right())
                    }
                }
            });
        }
    }

    pub fn run_with_stdout(&mut self) -> io::Result<String> {
        let cmd = format!("{:?}", self.0);
        info!("running {}", cmd);

        self.0.stdout(Stdio::piped());

        let child = self.0.spawn()
            .map_err(|why| Error::new(
                why.kind(),
                format!("failed to wait for process {}: {}", cmd, why)
            ))?;

        child.wait_with_output()
            .map_err(|why| Error::new(
                why.kind(),
                format!("failed to get output of {}: {}", cmd, why)
            ))
            .and_then(|output| {
                String::from_utf8(output.stdout)
                    .map_err(|why| Error::new(
                        ErrorKind::Other,
                        format!("command output has invalid UTF-8: {}", why)
                    ))
            })
    }

    pub fn run(&mut self) -> io::Result<()> {
        let cmd = format!("{:?}", self.0);
        info!("running {}", cmd);

        let mut child = self.0.spawn()
            .map_err(|why| Error::new(
                why.kind(),
                format!("failed to spawn process {}: {}", cmd, why)
            ))?;

        Self::redirect(child.stdout.take(), |msg| info!("{}", msg));
        Self::redirect(child.stderr.take(), |msg| warn!("{}", msg));

        let status = child.wait()
            .map_err(|why| Error::new(
                why.kind(),
                format!("failed to wait for process {}: {}", cmd, why)
            ))?;

        if status.success() {
            Ok(())
        } else if let Some(127) = status.code() {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("command {} was not found", cmd)
            ))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("command failed with exit status: {}", status)
            ))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_not_found() {
        assert!(Command::new("asdfasdf").run().unwrap_err().kind() == io::ErrorKind::NotFound);
    }

    #[test]
    fn command_with_output() {
        assert_eq!(
            Command::new("echo")
                .arg("Hello, Command!")
                .run_with_stdout()
                .unwrap(),
            "Hello, Command!\n".to_owned()
        );
    }
}