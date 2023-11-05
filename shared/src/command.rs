use std::io::{Error, ErrorKind, Result};
use std::process::{Command, Output, Child, Stdio};
use std::borrow::Cow;
use std::ffi::OsStr;

#[inline]
pub fn command_with_args<C, A>(cmd: C, args: A) -> Command where C: AsRef<OsStr>, A: IntoIterator<Item=C> {
    let mut command = Command::new(cmd);
    command.args(args);
    command
}

#[inline]
pub fn piped(cmd_ref: &mut Command) -> &mut Command {
    cmd_ref.stdout(Stdio::piped())
}

pub fn pipe_run(commands: &mut [Command]) -> Result<Output>{
    let mut previous: Option<Child> = None;
    for c in commands {
        c.stdout(Stdio::piped());
        if let Some(up) = previous {
            if let Some(output) = up.stdout {
                c.stdin(Stdio::from(output));
            } else {
                return Err(Error::new(ErrorKind::InvalidData, "Unable to get output from upstream"));
            }
        } 
        let proc = c.spawn()?;
        previous = Some(proc)
    }
    match previous {
        Some(process) => process.wait_with_output(),
        None => Err(Error::new(ErrorKind::InvalidData, "No process found"))
    }
}

pub fn stringfy(binary: &Vec<u8>) -> Cow<'_, str>{
    String::from_utf8_lossy(binary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piped_with_args() {
        let echo = command_with_args("echo", ["{\"name\":{\"first\":\"John\", \"last\":\"Berkman\"},\"age\":32}"]);
        let jq = command_with_args("jq", ["-r", ".name"]);
        let jq_first = command_with_args("jq", ["-r", ".first"]);
        let output = pipe_run(&mut [echo, jq, jq_first]).unwrap();
        let result = stringfy(&output.stdout);
        assert_eq!(result, "John\n");

    }
}
