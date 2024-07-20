use std::fmt::{self, Display};

use crate::{Event, ScalarValue};

/// Contains information for the next node to write to the YAML document.
#[derive(Clone, PartialEq, Debug, Eq)]
pub enum WriteEvent<'a> {
    /// Reserved for internal use.
    Nothing,
    /// Event generated at the very beginning of parsing.
    StreamStart,
    /// Last event that will be generated by the parser. Signals EOF.
    StreamEnd,
    /// The YAML start document directive (`---`).
    DocumentStart,
    /// The YAML end document directive (`...`).
    DocumentEnd,
    /// The start of a YAML sequence (array).
    SequenceStart,
    /// The end of a YAML sequence (array).
    SequenceEnd,
    /// The start of a YAML mapping (object, hash).
    MappingStart,
    /// The end of a YAML mapping (object, hash).
    MappingEnd,
    /// A value.
    Scalar(ScalarValue<'a>),
}

impl Display for WriteEvent<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WriteEvent::Nothing => formatter.write_str("Nothing"),
            WriteEvent::StreamStart => formatter.write_str("StreamStart"),
            WriteEvent::StreamEnd => formatter.write_str("StreamEnd"),
            WriteEvent::DocumentStart => formatter.write_str("DocumentStart"),
            WriteEvent::DocumentEnd => formatter.write_str("DocumentEnd"),
            WriteEvent::SequenceStart => formatter.write_str("SequenceStart"),
            WriteEvent::SequenceEnd => formatter.write_str("SequenceEnd"),
            WriteEvent::MappingStart => formatter.write_str("MappingStart"),
            WriteEvent::MappingEnd => formatter.write_str("MappingEnd"),
            WriteEvent::Scalar(scalar_value) => Display::fmt(scalar_value, formatter),
        }
    }
}

impl Into<WriteEvent<'_>> for Event {
    fn into(self) -> WriteEvent<'static> {
        match self {
            Event::Nothing => WriteEvent::Nothing,
            Event::StreamStart => WriteEvent::StreamStart,
            Event::StreamEnd => WriteEvent::StreamEnd,
            Event::DocumentStart => WriteEvent::DocumentStart,
            Event::DocumentEnd => WriteEvent::DocumentEnd,
            Event::SequenceStart(_, _) => WriteEvent::SequenceStart,
            Event::SequenceEnd => WriteEvent::SequenceEnd,
            Event::MappingStart(_, _) => WriteEvent::MappingStart,
            Event::MappingEnd => WriteEvent::MappingEnd,
            Event::Alias(_) => todo!(),
            Event::Scalar(value, style, anchor_id, tag) => {
                WriteEvent::Scalar(ScalarValue::from_scalar_event(value, style, anchor_id, tag))
            }
        }
    }
}
