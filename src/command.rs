use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use nota_codec::{Decoder, NotaDecode, NotaRecord, NotaSum};

use crate::error::{Error, Result};
use crate::{NiriFocusSource, SystemTarget};

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserveFocus {
    pub target: SystemTarget,
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscribeFocus {
    pub target: SystemTarget,
}

#[derive(NotaSum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    ObserveFocus(ObserveFocus),
    SubscribeFocus(SubscribeFocus),
}

impl Input {
    pub fn from_nota(text: &str) -> Result<Self> {
        let mut decoder = Decoder::new(text);
        Ok(Self::decode(&mut decoder)?)
    }

    pub fn run(self, source: &NiriFocusSource, mut output: impl Write) -> Result<()> {
        match self {
            Self::ObserveFocus(command) => {
                let observation = source.observe(command.target)?;
                writeln!(output, "{}", observation.to_nota())?;
                Ok(())
            }
            Self::SubscribeFocus(command) => source.subscribe(command.target, output),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLine {
    arguments: Vec<OsString>,
}

impl CommandLine {
    pub fn from_environment() -> Self {
        Self::from_arguments(std::env::args_os().skip(1))
    }

    pub fn from_arguments<I, S>(arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self {
            arguments: arguments.into_iter().map(Into::into).collect(),
        }
    }

    pub fn run(&self, source: &NiriFocusSource, output: impl Write) -> Result<()> {
        self.decode_input()?.run(source, output)
    }

    pub fn decode_input(&self) -> Result<Input> {
        let Some(first) = self.arguments.first() else {
            return Err(Error::MissingInput);
        };
        self.require_single_argument()?;

        if CommandLineArgument::new(first).starts_inline_record() {
            let Some(text) = first.to_str() else {
                return Err(Error::InvalidInlineNotaArgument {
                    got: format!("{first:?}"),
                });
            };
            Input::from_nota(text)
        } else {
            InputFile::from_path(PathBuf::from(first)).decode()
        }
    }

    fn require_single_argument(&self) -> Result<()> {
        if let Some(argument) = self.arguments.get(1) {
            return Err(Error::UnexpectedArgument {
                got: argument.to_string_lossy().to_string(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CommandLineArgument<'a> {
    argument: &'a OsString,
}

impl<'a> CommandLineArgument<'a> {
    fn new(argument: &'a OsString) -> Self {
        Self { argument }
    }

    fn starts_inline_record(self) -> bool {
        self.argument
            .as_encoded_bytes()
            .first()
            .is_some_and(|byte| *byte == b'(')
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InputFile {
    path: PathBuf,
}

impl InputFile {
    fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    fn decode(&self) -> Result<Input> {
        let text = std::fs::read_to_string(&self.path).map_err(|source| Error::InputFileRead {
            path: self.path.clone(),
            source,
        })?;
        Input::from_nota(&text)
    }
}
