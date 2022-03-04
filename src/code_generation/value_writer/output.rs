use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};

pub trait Output: Write {
    type Pos: Eq;
    fn position(&self) -> Self::Pos;
    fn truncate(&mut self, new_len: Self::Pos);
}

#[cfg(test)]
impl Output for Vec<u8> {
    type Pos = usize;

    fn position(&self) -> usize {
        self.len()
    }

    fn truncate(&mut self, new_len: usize) {
        Vec::truncate(self, new_len)
    }
}

impl Output for BufWriter<File> {
    type Pos = u64;

    fn position(&self) -> u64 {
        self.get_ref().stream_position().expect("Current position")
    }

    fn truncate(&mut self, new_len: u64) {
        self.get_mut().set_len(new_len).expect("Set len");
        self.seek(SeekFrom::End(0)).expect("Move cursor to end");
    }
}
