//! Execute process compose process.

use std::{
    io::Read,
    sync::{Arc, RwLock},
};

use process_wrap::tokio::*;
use tokio1::process::Command;

use crate::cli::cmd::{parent::ProcessComposeFlags, version::Version};

/// Environment variable for the path to the process compose binary,
/// if defined, used instead of default OS and process search strategy.
pub const ENV_PC_BIN: &str = "PC_BIN";

/// Comand line fully controlled by
/// - OS binary path search or `PC_BIN`
/// - command line arguments generated from `ProcessComposeFlags`
///
/// Returns a `CommandWrap` that can be used to interact with the spawned process, including adding wrappers.
pub fn process_compose(
    command: super::cmd::parent::ProcessComposeFlags,
) -> Result<CommandWrap, String> {
    let args: Vec<String> = command.try_into()?;
    let mut command = "process-compose".to_string();
    if let Some(pc_bin) = std::env::var_os(ENV_PC_BIN) {
        command = pc_bin.to_string_lossy().to_string();
    }

    let command = CommandWrap::with_new(command, |command| {
        command.args(&args);
    });

    Ok(command)
}

/// Simplest writer with infinite buffer.
///
/// # Safety
///
/// OoM can happen
#[derive(Debug)]
struct StdVecWrapper {
    writer: Arc<RwLock<Vec<u8>>>,
}

impl StdVecWrapper {
    fn new() -> (Self, Arc<RwLock<Vec<u8>>>) {
        let writer = Arc::new(RwLock::new(Vec::new()));
        (
            Self {
                writer: writer.clone(),
            },
            writer,
        )
    }
}

impl CommandWrapper for StdVecWrapper {
    fn pre_spawn(
        &mut self,
        command: &mut Command,
        _core: &CommandWrap,
    ) -> Result<(), std::io::Error> {
        let writer = self.writer.clone();
        let (mut process_reader, process_writer) = std::io::pipe()?;

        std::thread::spawn(move || {
            match process_reader.read_to_end(&mut writer.write().unwrap()) {
                Ok(_n) => {}
                Err(err) => match err.kind() {
                    std::io::ErrorKind::Interrupted => {}
                    _ => panic!("Error reading from pipe: {:?}", err),
                },
            }
        });

        command
            .stdout(process_writer.try_clone()?)
            .stderr(process_writer);
        Ok(())
    }
}

/// Gets vesion of current process-compose binary.
#[cfg(feature = "builder")]
pub async fn version() -> Version {
    let output = {
        let mut version = process_compose(
            ProcessComposeFlags::builder()
                .subcommand(super::cmd::parent::ProcessComposeCommand::Version)
                .build(),
        )
        .unwrap();
        let (out_wrapper, output) = StdVecWrapper::new();
        version.wrap(out_wrapper);
        let result = version.spawn().unwrap().wait().await.unwrap();
        assert!(result.success());
        output
    };

    let all = output.read().unwrap().to_vec();
    let version_str = String::from_utf8_lossy(&all);
    use core::str::FromStr;
    Version::from_str(version_str.trim()).unwrap()
}


#[allow(unused_imports)] // macro bug - should use self crate, not name tokio
use tokio1 as tokio;

#[cfg(test)]
#[tokio1::test]
async fn current_version() {
    let _ver = version().await;
}
