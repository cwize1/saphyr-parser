use std::{error::Error, fmt::{self, Display}};

use super::WriteEvent;

#[derive(Copy, Clone, Debug)]
pub enum WriteStateErrorType {
    /// The writer has been initialized.
    /// Waiting for: StreamStart.
    Start,
    /// The writer has finished.
    /// Expecting no more events.
    End,
    /// In the middle of a list of documents.
    /// Waiting for: DocumentStart, StreamEnd.
    DocumentList,
    /// Waiting for the end of the document.
    /// Waiting for: DocumentEnd.
    DocumentEnd,
    /// Waiting for a document's value.
    /// Waiting for: SequenceStart, MappingStart, Scalar.
    Document,
    /// In the middle of a sequence.
    /// Expecting: SequenceStart, MappingStart, Scalar, SequenceEnd.
    Sequence,
    /// In the middle of a map.
    /// Waiting for: SequenceStart, MappingStart, Scalar, MappingEnd.
    Mapping,
    /// In the middle of a map entry. Waiting for an entry's value.
    /// /// Waiting for: SequenceStart, MappingStart, Scalar,.
    MappingEntryValue,
    /// An error already occured. Cannot continue.
    ExistingError
}

#[derive(Clone, Debug)]
pub struct WriterStateError {
    pub state_type: WriteStateErrorType,
    pub event: WriteEvent,
}

impl WriterStateError {
    pub fn new(state_type: WriteStateErrorType, event: WriteEvent) -> WriterStateError {
        WriterStateError{state_type, event}
    }
}

impl Error for WriterStateError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl Display for WriterStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("invalid YAML write event, expecting ")?;
        let expecting = match self.state_type {
            WriteStateErrorType::Start => "start of stream (StreamStart)",
            WriteStateErrorType::End => "no more events due to end of stream",
            WriteStateErrorType::DocumentList => "document (DocumentStart, StreamEnd)",
            WriteStateErrorType::DocumentEnd => "end of document (DocumentEnd)",
            WriteStateErrorType::Document => "document value (SequenceStart, MappingStart, Scalar)",
            WriteStateErrorType::Sequence => "sequence item (SequenceStart, MappingStart, Scalar, SequenceEnd)",
            WriteStateErrorType::Mapping => "mapping entry key (SequenceStart, MappingStart, Scalar, MappingEnd)",
            WriteStateErrorType::MappingEntryValue => "mapping entry value (SequenceStart, MappingStart, Scalar)",
            WriteStateErrorType::ExistingError => "no more events due to previous error",
        };
        formatter.write_str(expecting)?;
        formatter.write_str(", got: ")?;
        Display::fmt(&self.event, formatter)?;
        Ok(())
    }
}
