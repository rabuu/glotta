macro_rules! emit {
    ($e:expr, $($arg:tt)*) => {
        $e.write(format_args!($($arg)*))
    };
}

macro_rules! emitln {
    ($e:expr) => {
        $e.newline()
    };
    ($e:expr, $($arg:tt)*) => {
        $e.writeln(format_args!($($arg)*))
    };
}

pub(crate) use emit;
pub(crate) use emitln;

use std::{fmt, io};

pub struct Emitter<O: io::Write> {
    out: O,
    indent_width: usize,
    indent: usize,
    at_line_start: bool,
}

impl<O: io::Write> Emitter<O> {
    pub fn new(out: O, indent_width: usize) -> Self {
        Self {
            out,
            indent_width,
            indent: 0,
            at_line_start: true,
        }
    }

    pub fn write(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        if self.at_line_start {
            write!(self.out, "{}", " ".repeat(self.indent_width * self.indent))?;
            self.at_line_start = false;
        }
        self.out.write_fmt(args)
    }

    pub fn writeln(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        self.write(args)?;
        self.newline()?;
        Ok(())
    }

    pub fn indent(&mut self) {
        self.indent = self.indent.saturating_add(1);
    }

    pub fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    pub fn newline(&mut self) -> io::Result<()> {
        self.at_line_start = true;
        writeln!(self.out)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}
