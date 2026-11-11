use crate::card::LocInfo;
use crate::list::List;
use std::cell::{Cell, RefCell};

// Port of ProgressiveBuffer: the response buffer, read and written as arrays of T, reading 0 past the end.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProgressiveBuffer {
    pub data: List<u8, 32>,
}

macro_rules! readers {
    ($($at:ident $type:ty),*) => {$(
        pub fn $at(&self, position: usize) -> $type {
            const SIZE: usize = size_of::<$type>();
            self.data
                .get(position * SIZE..(position + 1) * SIZE)
                .and_then(|bytes| bytes.try_into().ok())
                .map_or(0, <$type>::from_ne_bytes)
        }
    )*};
}

macro_rules! writers {
    ($($set:ident $type:ty),*) => {$(
        pub fn $set(&mut self, position: usize, value: $type) {
            const SIZE: usize = size_of::<$type>();
            if self.data.len() < (position + 1) * SIZE {
                self.data.resize((position + 1) * SIZE, 0);
            }
            if let Some(bytes) = self.data.get_mut(position * SIZE..(position + 1) * SIZE) {
                bytes.copy_from_slice(&value.to_ne_bytes());
            }
        }
    )*};
}

impl ProgressiveBuffer {
    readers!(at_u8 u8, at_u16 u16, at_i16 i16, at_i32 i32, at_u32 u32, at_u64 u64);
    writers!(set_u8 u8, set_i16 i16, set_i32 i32, set_u32 u32, set_u64 u64);

    pub fn bit_get(&self, position: usize) -> bool {
        self.data.get(position / 8).is_some_and(|byte| byte & (1 << (position % 8)) != 0)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

// The messages framed as OCG_DuelGetMessage does; `mute` skips what a rerun already wrote; cells for [Duel::check_mut].
#[derive(Debug, Clone, Default)]
pub struct Trace {
    pub buffer: RefCell<Option<Vec<u8>>>,
    pub count: Cell<u64>,
    pub mute: Cell<u64>,
}

impl Trace {
    pub fn message(&mut self, id: u8) -> Message<'_> {
        *self.count.get_mut() += 1;
        let mute = self.mute.get_mut();
        if *mute > 0 {
            *mute -= 1;
            return Message(None);
        }
        let buffer = self.buffer.get_mut().as_mut().map(|buffer| {
            let start = buffer.len();
            buffer.extend([0, 0, 0, 0, id]);
            (buffer, start)
        });
        Message(buffer)
    }
}

pub struct Message<'a>(Option<(&'a mut Vec<u8>, usize)>);

impl Message<'_> {
    pub(crate) fn bytes(&mut self, bytes: &[u8]) -> &mut Self {
        if let Some((buffer, _)) = &mut self.0 {
            buffer.extend_from_slice(bytes);
        }
        self
    }

    pub fn u8(&mut self, value: u8) -> &mut Self {
        self.bytes(&[value])
    }

    pub fn u16(&mut self, value: u16) -> &mut Self {
        self.bytes(&value.to_ne_bytes())
    }

    pub fn u32(&mut self, value: u32) -> &mut Self {
        self.bytes(&value.to_ne_bytes())
    }

    pub fn u64(&mut self, value: u64) -> &mut Self {
        self.bytes(&value.to_ne_bytes())
    }

    pub fn loc(&mut self, info: LocInfo) -> &mut Self {
        self.u8(info.controler).u8(info.location).u32(info.sequence).u32(info.position)
    }
}

impl Drop for Message<'_> {
    fn drop(&mut self) {
        if let Some((buffer, start)) = &mut self.0 {
            let length = (buffer.len() - *start - 4) as u32;
            if let Some(prefix) = buffer.get_mut(*start..*start + 4) {
                prefix.copy_from_slice(&length.to_ne_bytes());
            }
        }
    }
}
