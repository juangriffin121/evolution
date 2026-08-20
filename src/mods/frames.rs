use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::mods::blobs::Blob;

#[derive(Serialize, Deserialize)]
pub struct FrameBlob {
    pub x: f32,
    pub y: f32,
    pub is_prey: bool,
    pub energy: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Frame {
    pub age: usize,
    pub blobs: Vec<FrameBlob>,
}

#[derive(Debug)]
pub struct FrameWriter {
    writer: BufWriter<File>,
}

impl FrameWriter {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = File::create(path)?;

        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn write_frame(&mut self, age: usize, blobs: &[Blob]) -> std::io::Result<()> {
        self.writer.write_all(&(age as u64).to_le_bytes())?;

        self.writer.write_all(&(blobs.len() as u32).to_le_bytes())?;

        for blob in blobs {
            self.writer.write_all(&blob.position.0.to_le_bytes())?;
            self.writer.write_all(&blob.position.1.to_le_bytes())?;

            let blob_type = blob.blob_type as u8;
            self.writer.write_all(&[blob_type])?;

            self.writer.write_all(&blob.energy.to_le_bytes())?;
        }

        Ok(())
    }
}
