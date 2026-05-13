use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::Result;

use crate::polymarket::signals::MarketSignal;

pub struct SignalLogger {
    writer: BufWriter<File>,
}

impl SignalLogger {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn log(&mut self, signal: &MarketSignal) -> Result<()> {
        serde_json::to_writer(&mut self.writer, signal)?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()?;

        Ok(())
    }
}
