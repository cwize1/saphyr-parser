//! Home to the YAML Parser.
//!
//! The writer is used to write YAML documents. It takes a stream of events containing YAML node information.

mod write_error;
mod write_event;
mod write_event_type;
mod write_stack;
mod writer_state_error;

use std::fmt;

pub use write_error::WriteError;
pub use write_event::WriteEvent;
use write_stack::{StackFrame, StartFrame};
use writer_state_error::WriteStateErrorType;


/// A YAML writer.
pub struct YamlWriter<'a> {
    writer: &'a mut dyn fmt::Write,
    ident_size: usize,
    compact: bool,
    multiline_strings: bool,
    omit_first_doc_separator: bool,
    level: isize,
    stack: Vec<StackFrame>,
}

/// A convenience alias for writer functions that may fail without returning a value.
pub type WriteResult = Result<(), WriteError>;

impl YamlWriter<'_> {
    /// Create a new emitter serializing into `writer`.
    pub fn new<'b>(writer: &'b mut dyn fmt::Write) -> YamlWriter<'b> {
        YamlWriter {
            writer,
            ident_size: 2,
            compact: true,
            multiline_strings: true,
            omit_first_doc_separator: true,
            level: -1,
            stack: vec![StackFrame::Start(StartFrame::new())],
        }
    }

    /// Set 'compact inline notation' on or off, as described for block
    /// [sequences](http://www.yaml.org/spec/1.2/spec.html#id2797382)
    /// and
    /// [mappings](http://www.yaml.org/spec/1.2/spec.html#id2798057).
    ///
    /// In this form, blocks cannot have any properties (such as anchors
    /// or tags), which should be OK, because this emitter doesn't
    /// (currently) emit those anyways.
    ///
    /// TODO(ethiraric, 2024/04/02): We can support those now.
    pub fn compact(&mut self, compact: bool) {
        self.compact = compact;
    }

    /// Determine if this emitter is using 'compact inline notation'.
    #[must_use]
    pub fn is_compact(&self) -> bool {
        self.compact
    }

    /// Render strings containing multiple lines in [literal style].
    ///
    /// [literal style]: https://yaml.org/spec/1.2/spec.html#id2795688
    pub fn multiline_strings(&mut self, multiline_strings: bool) {
        self.multiline_strings = multiline_strings;
    }

    /// Determine if this emitter will emit multiline strings when appropriate.
    #[must_use]
    pub fn is_multiline_strings(&self) -> bool {
        self.multiline_strings
    }

    /// Don't write the YAML start document directive (`---`) for the first document.
    pub fn omit_first_doc_separator(&mut self, omit_first_doc_separator: bool) {
        self.omit_first_doc_separator = omit_first_doc_separator;
    }

    /// Determine if this writer will write the YAML start document directive (`---`) for the first document.
    #[must_use]
    pub fn is_omit_first_doc_separator(&self) -> bool {
        self.omit_first_doc_separator
    }

    /// Writes an event.
    pub fn event(&mut self, event: WriteEvent) -> WriteResult {
        let stack_frame = self.stack.pop();
        match stack_frame {
            Some(stack_frame) => {
                let result = stack_frame.run(self, event);
                if let Err(_) = result {
                    // Recovery isn't supported.
                    // So, clear the stack to prevent event() from being called again.
                    self.stack.clear();
                }
                result
            },
            None => Err(WriteError::new_state_error(WriteStateErrorType::ExistingError, &event)),
        }
    }

    pub(self) fn writer<'b>(&'b mut self) -> &'b mut dyn fmt::Write {
        self.writer
    }

    pub(self) fn stack_push(&mut self, frame: StackFrame) {
        self.stack.push(frame);
    }

    fn write_indent(&mut self) -> WriteResult {
        if self.level <= 0 {
            return Ok(());
        }
        let num = self.level as usize * self.ident_size;
        for _ in 0..num {
            write!(self.writer, " ")?;
        }
        Ok(())
    }
}
