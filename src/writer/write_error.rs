use std::{error::Error, fmt::{self, Display}};

use super::{writer_state_error::{WriteStateErrorType, WriterStateError}, WriteEvent};

/// An error when writing YAML.
#[derive(Clone, Debug)]
pub enum WriteError {
    /// A formatting error.
    FmtError(fmt::Error),
    /// The writer was in the wrong state for the event provided.
    StateError(WriterStateError),
}

impl WriteError {
    pub(crate) fn new_state_error(state_type: WriteStateErrorType, event: WriteEvent) -> WriteError {
        WriteError::StateError(WriterStateError{state_type, event})
    }
}

impl Error for WriteError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl Display for WriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WriteError::FmtError(err) => Display::fmt(err, formatter),
            WriteError::StateError(err) => Display::fmt(err, formatter),
        }
    }
}

impl From<fmt::Error> for WriteError {
    fn from(f: fmt::Error) -> Self {
        WriteError::FmtError(f)
    }
}

impl From<WriterStateError> for WriteError {
    fn from(f: WriterStateError) -> Self {
        WriteError::StateError(f)
    }
}
