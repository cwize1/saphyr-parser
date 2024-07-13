use std::fmt;

use crate::{char_traits, ScalarValue};

use super::{writer_state_error::WriteStateErrorType, WriteError, WriteEvent, WriteResult, YamlWriter};

pub enum StackFrame {
    Start(StartFrame),
    End(EndFrame),
    Docs(DocsFrame),
    Doc(DocFrame),
    DocEnd(DocEndFrame),
    Sequence(SequenceFrame),
    Mapping(MappingFrame),
    MappingItemValue(MappingItemValueFrame),
    ValSequence(ValSequenceFrame),
    ValMapping(ValMappingFrame),
}

impl StackFrame {
    pub fn run(self, parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        match self {
            StackFrame::Start(frame) => frame.run(parent, event),
            StackFrame::End(frame) => frame.run(parent, event),
            StackFrame::Docs(frame) => frame.run(parent, event),
            StackFrame::Doc(frame) => frame.run(parent, event),
            StackFrame::DocEnd(frame) => frame.run(parent, event),
            StackFrame::Sequence(frame) => frame.run(parent, event),
            StackFrame::Mapping(frame) => frame.run(parent, event),
            StackFrame::MappingItemValue(frame) => frame.run(parent, event),
            StackFrame::ValSequence(frame) => frame.run(parent, event),
            StackFrame::ValMapping(frame) => frame.run(parent, event),
        }
    }
}

pub trait StackFrameRun {
    fn run(&self, parent: &mut YamlWriter, event: WriteEvent) -> WriteResult;
}

pub struct StartFrame {
}

impl StartFrame {
    pub fn new() -> StartFrame {
        return StartFrame{}
    }
}

impl StackFrameRun for StartFrame {
    fn run(&self, parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        match event {
            WriteEvent::StreamStart => {
                parent.stack_push(StackFrame::Docs(DocsFrame::new(true)));
                Ok(())
            },
            _ => Err(WriteError::new_state_error(WriteStateErrorType::Start, &event)),
        }
    }
}

pub struct EndFrame {
}

impl EndFrame {
    pub fn new() -> EndFrame {
        return EndFrame{}
    }
}

impl StackFrameRun for EndFrame {
    fn run(&self, _parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        match event {
            _ => Err(WriteError::new_state_error(WriteStateErrorType::End, &event)),
        }
    }
}

pub struct DocsFrame {
    first: bool,
}

impl DocsFrame {
    pub fn new(first: bool) -> DocsFrame {
        return DocsFrame{ first }
    }
}

impl StackFrameRun for DocsFrame {
    fn run(&self, parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        match event {
            WriteEvent::DocumentStart => {
                if !self.first {
                    writeln!(parent.writer())?;
                }
                if !self.first || !parent.is_omit_first_doc_separator() {
                    writeln!(parent.writer(), "---")?;
                }

                parent.stack_push(StackFrame::Docs(DocsFrame::new(false)));
                parent.stack_push(StackFrame::Doc(DocFrame::new()));
                Ok(())
            },
            WriteEvent::StreamEnd => {
                parent.stack_push(StackFrame::End(EndFrame::new()));
                Ok(())
            },
            _ => Err(WriteError::new_state_error(WriteStateErrorType::DocumentList, &event)),
        }
    }
}

pub struct DocFrame {
}

impl DocFrame {
    pub fn new() -> DocFrame {
        return DocFrame{}
    }
}

impl StackFrameRun for DocFrame {
    fn run(&self, parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        match event {
            WriteEvent::SequenceStart | WriteEvent::MappingStart | WriteEvent::Scalar(_) => {
                parent.stack_push(StackFrame::DocEnd(DocEndFrame::new()));
                run_node(parent, event)?;
                Ok(())
            }
            WriteEvent::DocumentEnd => {
                Ok(())
            },
            _ => Err(WriteError::new_state_error(WriteStateErrorType::Document, &event)),
        }
    }
}

pub struct DocEndFrame {
}

impl DocEndFrame {
    pub fn new() -> DocEndFrame {
        return DocEndFrame{}
    }
}

impl StackFrameRun for DocEndFrame {
    fn run(&self, _parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        match event {
            WriteEvent::DocumentEnd => {
                Ok(())
            },
            _ => Err(WriteError::new_state_error(WriteStateErrorType::DocumentEnd, &event)),
        }
    }
}

fn run_node(parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
    match event {
        WriteEvent::SequenceStart => {
            parent.stack_push(StackFrame::Sequence(SequenceFrame::new(true)));
            Ok(())
        },
        WriteEvent::MappingStart => {
            parent.stack_push(StackFrame::Mapping(MappingFrame::new(true)));
            Ok(())
        },
        WriteEvent::Scalar(scalar_value) => {
            run_scalar(parent, scalar_value)?;
            Ok(())
        }
        // Callers do not pass in other types of events.
        _ => unreachable!(),
    }
}

pub struct SequenceFrame {
    first: bool,
}

impl SequenceFrame {
    pub fn new(first: bool) -> SequenceFrame {
        return SequenceFrame{ first }
    }
}

impl StackFrameRun for SequenceFrame {
    fn run(&self, parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        match event {
            WriteEvent::SequenceStart | WriteEvent::MappingStart | WriteEvent::Scalar(_) => {
                if self.first {
                    parent.level += 1;
                } else {
                    writeln!(parent.writer())?;
                    parent.write_indent()?;
                }

                parent.writer().write_str("-")?;
                parent.stack_push(StackFrame::Sequence(SequenceFrame::new(false)));
                run_val(parent, event, true)?;
                Ok(())
            },
            WriteEvent::SequenceEnd => {
                if self.first {
                    parent.writer().write_str("[]")?;
                } else {
                    parent.level -= 1;
                }
                Ok(())
            },
            _ => Err(WriteError::new_state_error(WriteStateErrorType::Sequence, &event)),
        }
    }
}

pub struct MappingFrame {
    first: bool,
}

impl MappingFrame {
    pub fn new(first: bool) -> MappingFrame {
        return MappingFrame{ first }
    }
}

impl StackFrameRun for MappingFrame {
    fn run(&self, parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        match event {
            WriteEvent::SequenceStart | WriteEvent::MappingStart | WriteEvent::Scalar(_) => {
                if self.first {
                    parent.level += 1;
                } else {
                    writeln!(parent.writer())?;
                    parent.write_indent()?;
                }

                let complex_key = matches!(event, WriteEvent::SequenceStart | WriteEvent::MappingStart);
                parent.stack_push(StackFrame::MappingItemValue(MappingItemValueFrame::new(complex_key)));

                if complex_key {
                    parent.writer().write_str("?")?;
                    run_val(parent, event, true)?;
                } else {
                    run_node(parent, event)?;
                }
                Ok(())
            },
            WriteEvent::MappingEnd => {
                if self.first {
                    parent.writer().write_str("{}")?;
                } else {
                    parent.level -= 1;
                }
                Ok(())
            },
            _ => Err(WriteError::new_state_error(WriteStateErrorType::Mapping, &event)),
        }
    }
}

pub struct MappingItemValueFrame {
    complex_key: bool,
}

impl MappingItemValueFrame {
    pub fn new(complex_key: bool) -> MappingItemValueFrame {
        return MappingItemValueFrame{ complex_key }
    }
}

impl StackFrameRun for MappingItemValueFrame {
    fn run(&self, parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        match event {
            WriteEvent::SequenceStart | WriteEvent::MappingStart | WriteEvent::Scalar(_) => {
                if self.complex_key {
                    writeln!(parent.writer())?;
                    parent.write_indent()?;
                }
        
                parent.writer().write_str(":")?;
        
                parent.stack_push(StackFrame::Mapping(MappingFrame::new(false)));
                run_val(parent, event, self.complex_key)?;
                Ok(())
            }
            _ => Err(WriteError::new_state_error(WriteStateErrorType::MappingEntryValue, &event)),
        }
    }
}

fn run_val(parent: &mut YamlWriter, event: WriteEvent, inline: bool) -> WriteResult {
    match event {
        WriteEvent::SequenceStart => {
            parent.stack_push(StackFrame::ValSequence(ValSequenceFrame::new(inline)));
            Ok(())
        }
        WriteEvent::MappingStart => {
            parent.stack_push(StackFrame::ValMapping(ValMappingFrame::new(inline)));
            Ok(())
        },
        WriteEvent::Scalar(scalar_value) => {
            write!(parent.writer(), " ")?;
            run_scalar(parent, scalar_value)?;
            Ok(())
        }
        // Callers do not pass in other types of events.
        _ => unreachable!(),
    }
}

pub struct ValSequenceFrame {
    pub inline: bool,
}

impl ValSequenceFrame {
    pub fn new(inline: bool) -> ValSequenceFrame {
        return ValSequenceFrame{ inline }
    }
}

impl StackFrameRun for ValSequenceFrame {
    fn run(&self, parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        let is_empty = match event {
            WriteEvent::SequenceStart | WriteEvent::MappingStart | WriteEvent::Scalar(_) => false,
            WriteEvent::SequenceEnd => true,
            _ => return Err(WriteError::new_state_error(WriteStateErrorType::Sequence, &event)),
        };

        if (self.inline && parent.is_compact()) || is_empty {
            parent.writer().write_str(" ")?;
        } else {
            writeln!(parent.writer())?;
            parent.level += 1;
            parent.write_indent()?;
            parent.level -= 1;
        }

        SequenceFrame::new(true).run(parent, event)?;
        Ok(())
    }
}

pub struct ValMappingFrame {
    pub inline: bool,
}

impl ValMappingFrame {
    pub fn new(inline: bool) -> ValMappingFrame {
        return ValMappingFrame{ inline }
    }
}

impl StackFrameRun for ValMappingFrame {
    fn run(&self, parent: &mut YamlWriter, event: WriteEvent) -> WriteResult {
        let is_empty = match event {
            WriteEvent::SequenceStart | WriteEvent::MappingStart | WriteEvent::Scalar(_) => false,
            WriteEvent::MappingEnd => true,
            _ => return Err(WriteError::new_state_error(WriteStateErrorType::Mapping, &event)),
        };

        if (self.inline && parent.is_compact()) || is_empty {
            parent.writer().write_str(" ")?;
        } else {
            writeln!(parent.writer())?;
            parent.level += 1;
            parent.write_indent()?;
            parent.level -= 1;
        }

        MappingFrame::new(true).run(parent, event)?;
        Ok(())
    }
}

fn run_scalar(parent: &mut YamlWriter, scalar_value: ScalarValue) -> WriteResult {
    match scalar_value {
        ScalarValue::String(ref v) => {
            if parent.is_multiline_strings()
                && v.contains('\n')
                && char_traits::is_valid_literal_block_scalar(v)
            {
                write_literal_block(parent, v)?;
            } else if need_quotes(v) {
                escape_str(parent.writer(), v)?;
            } else {
                parent.writer().write_str(v)?;
            }
            Ok(())
        }
        ScalarValue::Boolean(v) => {
            if v {
                parent.writer().write_str("true")?;
            } else {
                parent.writer().write_str("false")?;
            }
            Ok(())
        }
        ScalarValue::Integer(v) => {
            write!(parent.writer(), "{v}")?;
            Ok(())
        }
        ScalarValue::Real(ref v) => {
            parent.writer().write_str(v)?;
            Ok(())
        }
        ScalarValue::Null | ScalarValue::BadValue => {
            parent.writer().write_str("~")?;
            Ok(())
        }
    }
}

fn write_literal_block(parent: &mut YamlWriter, v: &str) -> WriteResult {
    let ends_with_newline = v.ends_with('\n');
    if ends_with_newline {
        parent.writer().write_str("|")?;
    } else {
        parent.writer().write_str("|-")?;
    }

    parent.level += 1;
    // lines() will omit the last line if it is empty.
    for line in v.lines() {
        writeln!(parent.writer())?;
        parent.write_indent()?;
        // It's literal text, so don't escape special chars.
        parent.writer().write_str(line)?;
    }
    parent.level -= 1;
    Ok(())
}

/// Check if the string requires quoting.
/// Strings starting with any of the following characters must be quoted.
/// :, &, *, ?, |, -, <, >, =, !, %, @
/// Strings containing any of the following characters must be quoted.
/// {, }, \[, t \], ,, #, `
///
/// If the string contains any of the following control characters, it must be escaped with double quotes:
/// \0, \x01, \x02, \x03, \x04, \x05, \x06, \a, \b, \t, \n, \v, \f, \r, \x0e, \x0f, \x10, \x11, \x12, \x13, \x14, \x15, \x16, \x17, \x18, \x19, \x1a, \e, \x1c, \x1d, \x1e, \x1f, \N, \_, \L, \P
///
/// Finally, there are other cases when the strings must be quoted, no matter if you're using single or double quotes:
/// * When the string is true or false (otherwise, it would be treated as a boolean value);
/// * When the string is null or ~ (otherwise, it would be considered as a null value);
/// * When the string looks like a number, such as integers (e.g. 2, 14, etc.), floats (e.g. 2.6, 14.9) and exponential numbers (e.g. 12e7, etc.) (otherwise, it would be treated as a numeric value);
/// * When the string looks like a date (e.g. 2014-12-31) (otherwise it would be automatically converted into a Unix timestamp).
#[allow(clippy::doc_markdown)]
fn need_quotes(string: &str) -> bool {
    fn need_quotes_spaces(string: &str) -> bool {
        string.starts_with(' ') || string.ends_with(' ')
    }

    string.is_empty()
        || need_quotes_spaces(string)
        || string.starts_with(|character: char| {
            matches!(
                character,
                '&' | '*' | '?' | '|' | '-' | '<' | '>' | '=' | '!' | '%' | '@'
            )
        })
        || string.contains(|character: char| {
            matches!(character, ':'
            | '{'
            | '}'
            | '['
            | ']'
            | ','
            | '#'
            | '`'
            | '\"'
            | '\''
            | '\\'
            | '\0'..='\x06'
            | '\t'
            | '\n'
            | '\r'
            | '\x0e'..='\x1a'
            | '\x1c'..='\x1f')
        })
        || [
            // http://yaml.org/type/bool.html
            // Note: 'y', 'Y', 'n', 'N', is not quoted deliberately, as in libyaml. PyYAML also parse
            // them as string, not booleans, although it is violating the YAML 1.1 specification.
            // See https://github.com/dtolnay/serde-yaml/pull/83#discussion_r152628088.
            "yes", "Yes", "YES", "no", "No", "NO", "True", "TRUE", "true", "False", "FALSE",
            "false", "on", "On", "ON", "off", "Off", "OFF",
            // http://yaml.org/type/null.html
            "null", "Null", "NULL", "~",
        ]
        .contains(&string)
        || string.starts_with('.')
        || string.starts_with("0x")
        || string.parse::<i64>().is_ok()
        || string.parse::<f64>().is_ok()
}

// from serialize::json
fn escape_str(wr: &mut dyn fmt::Write, v: &str) -> Result<(), fmt::Error> {
    wr.write_str("\"")?;

    let mut start = 0;

    for (i, byte) in v.bytes().enumerate() {
        let escaped = match byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x00' => "\\u0000",
            b'\x01' => "\\u0001",
            b'\x02' => "\\u0002",
            b'\x03' => "\\u0003",
            b'\x04' => "\\u0004",
            b'\x05' => "\\u0005",
            b'\x06' => "\\u0006",
            b'\x07' => "\\u0007",
            b'\x08' => "\\b",
            b'\t' => "\\t",
            b'\n' => "\\n",
            b'\x0b' => "\\u000b",
            b'\x0c' => "\\f",
            b'\r' => "\\r",
            b'\x0e' => "\\u000e",
            b'\x0f' => "\\u000f",
            b'\x10' => "\\u0010",
            b'\x11' => "\\u0011",
            b'\x12' => "\\u0012",
            b'\x13' => "\\u0013",
            b'\x14' => "\\u0014",
            b'\x15' => "\\u0015",
            b'\x16' => "\\u0016",
            b'\x17' => "\\u0017",
            b'\x18' => "\\u0018",
            b'\x19' => "\\u0019",
            b'\x1a' => "\\u001a",
            b'\x1b' => "\\u001b",
            b'\x1c' => "\\u001c",
            b'\x1d' => "\\u001d",
            b'\x1e' => "\\u001e",
            b'\x1f' => "\\u001f",
            b'\x7f' => "\\u007f",
            _ => continue,
        };

        if start < i {
            wr.write_str(&v[start..i])?;
        }

        wr.write_str(escaped)?;

        start = i + 1;
    }

    if start != v.len() {
        wr.write_str(&v[start..])?;
    }

    wr.write_str("\"")?;
    Ok(())
}
