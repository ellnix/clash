mod test_run;

use std::io::Write;
use std::process::Command;
use std::time::Duration;

pub use test_run::{TestResult, TestRun};
use wait_timeout::ChildExt;

use crate::clash::TestCase;

pub fn lazy_run<'a>(
    testcases: impl IntoIterator<Item = &'a TestCase>,
    run_command: &'a mut Command,
    timeout: &'a Duration,
) -> impl IntoIterator<Item = TestRun<'a>> {
    testcases.into_iter().map(|test| run_testcase(test, run_command, timeout))
}

fn run_testcase<'a>(test: &'a TestCase, run_command: &mut Command, timeout: &Duration) -> TestRun<'a> {
    let mut run = match run_command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(run) => run,
        Err(error) => return TestRun::new(test, unable_to_run(error, run_command)),
    };

    run.stdin
        .as_mut()
        .unwrap()
        .write(test.test_in.as_bytes())
        .expect("Fatal error: could not write to stdin.");

    TestRun::new(test, get_result(run, &test.test_out, timeout))
}

fn unable_to_run(error: std::io::Error, cmd: &mut Command) -> TestResult {
    TestResult::UnableToRun {
        error_msg: format!("{}: {}", cmd.get_program().to_str().unwrap_or("Unable to run command"), error),
    }
}

fn get_result(mut run: std::process::Child, expected: &str, timeout: &Duration) -> TestResult {
    let timed_out = run.wait_timeout(*timeout).expect("Could not wait for program execution.").is_none();

    if timed_out {
        run.kill().expect("Failed to kill test run process");
    }

    let output = run.wait_with_output().expect("Could not wait for program execution.");

    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_default()
        .replace("\r\n", "\n")
        .trim_end()
        .to_string();
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();

    if stdout == expected.trim_end() {
        TestResult::Success
    } else if timed_out {
        TestResult::Timeout { stdout, stderr }
    } else if output.status.success() {
        TestResult::WrongOutput { stdout, stderr }
    } else {
        TestResult::RuntimeError { stdout, stderr }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::time::Duration;

    use anyhow::Context;
    use directories::ProjectDirs;

    use super::*;
    use crate::clash::{Clash, PublicHandle};

    fn sample_puzzle() -> anyhow::Result<Clash> {
        // Boggus clash, all tests are "123"
        let handle = PublicHandle::from_str("90435e82d1d5e3fe5f9d3dd813770f0d5a7d2")?;
        let clashes_dir = ProjectDirs::from("", "CoCtus", "coctus")
            .expect("Unable to find project directory")
            .data_dir()
            .join("clashes");
        let clash_file = clashes_dir.join(format!("{}.json", handle));
        let contents = std::fs::read_to_string(&clash_file)
            .with_context(|| format!("Unable to find clash with handle {}", handle))?;
        let clash: Clash = serde_json::from_str(&contents)
            .with_context(|| format!("Unable to deserialize clash from {:?}", &clash_file))?;

        Ok(clash)
    }

    #[test]
    fn test_passing_solution() {
        let clash = sample_puzzle().unwrap();
        let mut run_cmd = Command::new("sh");
        run_cmd.arg("-c");
        run_cmd.arg("read input; echo 123");
        let timeout = Duration::from_secs(1);
        assert!(lazy_run(clash.testcases(), &mut run_cmd, &timeout)
            .into_iter()
            .all(|test_run| test_run.is_successful()))
    }

    #[test]
    fn test_failing_solution() {
        let clash = sample_puzzle().unwrap();
        let mut run_cmd = Command::new("sh");
        run_cmd.arg("-c");
        run_cmd.arg("read input; echo nada");
        let timeout = Duration::from_secs(1);
        assert!(lazy_run(clash.testcases(), &mut run_cmd, &timeout)
            .into_iter()
            .all(|test_run| !test_run.is_successful()))
    }
}
