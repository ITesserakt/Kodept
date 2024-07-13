use std::io::Write;
use std::sync::{Arc, Mutex};

use codespan_reporting::term::termcolor::{ColorSpec, StandardStream, WriteColor};

pub type CodespanSettings = kodept_macros::error::traits::CodespanSettings<StreamOutput>;

#[derive(Clone)]
pub enum SupportColor {
    Yes,
    No,
}

#[derive(Clone)]
pub enum StreamOutput {
    Standard(Arc<Mutex<StandardStream>>),
    NoOp,
}

const POISON_LOCK_ERROR: &str = "Lock was poisoned";

impl Write for StreamOutput {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            StreamOutput::Standard(x) => x.lock().expect(POISON_LOCK_ERROR).write(buf),
            StreamOutput::NoOp => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            StreamOutput::Standard(x) => x.lock().expect(POISON_LOCK_ERROR).flush(),
            StreamOutput::NoOp => Ok(()),
        }
    }
}

impl WriteColor for StreamOutput {
    fn supports_color(&self) -> bool {
        match self {
            StreamOutput::Standard(x) => {
                let guard = x.lock();
                match guard {
                    Ok(it) => it.supports_color(),
                    Err(it) => it.get_ref().supports_color(),
                }
            }
            StreamOutput::NoOp => false,
        }
    }

    fn set_color(&mut self, spec: &ColorSpec) -> std::io::Result<()> {
        match self {
            StreamOutput::Standard(x) => x.lock().expect(POISON_LOCK_ERROR).set_color(spec),
            StreamOutput::NoOp => Ok(()),
        }
    }

    fn reset(&mut self) -> std::io::Result<()> {
        match self {
            StreamOutput::Standard(x) => x.lock().expect(POISON_LOCK_ERROR).reset(),
            StreamOutput::NoOp => Ok(()),
        }
    }
}
