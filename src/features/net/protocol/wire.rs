//! Niskopoziomowe bufory zapisu i odczytu binarnego (`PacketWriter`, `PacketReader`) oraz trait `WireCodec`.

pub struct PacketWriter {
    pub buf: Vec<u8>,
}

impl PacketWriter {
    pub fn new(packet_id: u8) -> Self {
        let mut pw = Self {
            buf: Vec::with_capacity(64),
        };
        pw.buf.extend_from_slice(&[0, 0, 0, 0]);
        pw.write_u8(packet_id);
        pw
    }

    pub fn write_u8(&mut self, val: u8) {
        self.buf.push(val);
    }

    pub fn write_bool(&mut self, val: bool) {
        self.buf.push(if val { 1 } else { 0 });
    }

    pub fn write_u16(&mut self, val: u16) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_u32(&mut self, val: u32) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_u64(&mut self, val: u64) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_i32(&mut self, val: i32) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_f32(&mut self, val: f32) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_string(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let len = bytes.len().min(u16::MAX as usize) as u16;
        self.write_u16(len);
        self.buf.extend_from_slice(&bytes[..len as usize]);
    }

    pub fn finish(mut self) -> Vec<u8> {
        let payload_len = (self.buf.len() - 4) as u32;
        self.buf[0..4].copy_from_slice(&payload_len.to_le_bytes());
        self.buf
    }
}

pub struct PacketReader<'a> {
    pub(super) data: &'a [u8],
    pub(super) cursor: usize,
}

impl<'a> PacketReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.cursor)
    }

    pub fn read_u8(&mut self) -> Result<u8, &'static str> {
        if self.cursor < self.data.len() {
            let val = self.data[self.cursor];
            self.cursor += 1;
            Ok(val)
        } else {
            Err("Unexpected EOF reading u8")
        }
    }

    pub fn read_bool(&mut self) -> Result<bool, &'static str> {
        self.read_u8().map(|v| v != 0)
    }

    pub fn read_u16(&mut self) -> Result<u16, &'static str> {
        if self.cursor + 2 <= self.data.len() {
            let val = u16::from_le_bytes(self.data[self.cursor..self.cursor + 2].try_into().unwrap());
            self.cursor += 2;
            Ok(val)
        } else {
            Err("Unexpected EOF reading u16")
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, &'static str> {
        if self.cursor + 4 <= self.data.len() {
            let val = u32::from_le_bytes(self.data[self.cursor..self.cursor + 4].try_into().unwrap());
            self.cursor += 4;
            Ok(val)
        } else {
            Err("Unexpected EOF reading u32")
        }
    }

    pub fn read_u64(&mut self) -> Result<u64, &'static str> {
        if self.cursor + 8 <= self.data.len() {
            let val = u64::from_le_bytes(self.data[self.cursor..self.cursor + 8].try_into().unwrap());
            self.cursor += 8;
            Ok(val)
        } else {
            Err("Unexpected EOF reading u64")
        }
    }

    pub fn read_i32(&mut self) -> Result<i32, &'static str> {
        if self.cursor + 4 <= self.data.len() {
            let val = i32::from_le_bytes(self.data[self.cursor..self.cursor + 4].try_into().unwrap());
            self.cursor += 4;
            Ok(val)
        } else {
            Err("Unexpected EOF reading i32")
        }
    }

    pub fn read_f32(&mut self) -> Result<f32, &'static str> {
        if self.cursor + 4 <= self.data.len() {
            let val = f32::from_le_bytes(self.data[self.cursor..self.cursor + 4].try_into().unwrap());
            self.cursor += 4;
            Ok(val)
        } else {
            Err("Unexpected EOF reading f32")
        }
    }

    pub fn read_string(&mut self) -> Result<String, &'static str> {
        let len = self.read_u16()? as usize;
        if self.cursor + len <= self.data.len() {
            let s = String::from_utf8_lossy(&self.data[self.cursor..self.cursor + len]).into_owned();
            self.cursor += len;
            Ok(s)
        } else {
            Err("Unexpected EOF reading string")
        }
    }
}

pub(super) trait WireCodec: Sized {
    fn encode(&self, pw: &mut PacketWriter);
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str>;
}

macro_rules! impl_primitive_codec {
    ($($ty:ty => ($w:ident, $r:ident)),* $(,)?) => {
        $(
            impl WireCodec for $ty {
                #[inline(always)]
                fn encode(&self, pw: &mut PacketWriter) { pw.$w(*self); }
                #[inline(always)]
                fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> { pr.$r() }
            }
        )*
    };
}

impl_primitive_codec! {
    u8 => (write_u8, read_u8),
    bool => (write_bool, read_bool),
    u16 => (write_u16, read_u16),
    u32 => (write_u32, read_u32),
    u64 => (write_u64, read_u64),
    i32 => (write_i32, read_i32),
    f32 => (write_f32, read_f32),
}

impl WireCodec for String {
    #[inline(always)]
    fn encode(&self, pw: &mut PacketWriter) {
        pw.write_string(self);
    }
    #[inline(always)]
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        pr.read_string()
    }
}

impl<T: WireCodec> WireCodec for Option<T> {
    #[inline(always)]
    fn encode(&self, pw: &mut PacketWriter) {
        pw.write_bool(self.is_some());
        if let Some(v) = self {
            v.encode(pw);
        }
    }
    #[inline(always)]
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        if pr.read_bool()? {
            Ok(Some(T::decode(pr)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: WireCodec + Copy, const N: usize> WireCodec for [Option<T>; N] {
    fn encode(&self, pw: &mut PacketWriter) {
        for item in self {
            item.encode(pw);
        }
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        let mut arr = [None; N];
        for item in &mut arr {
            *item = Option::<T>::decode(pr)?;
        }
        Ok(arr)
    }
}

impl WireCodec for (i32, i32, i32) {
    fn encode(&self, pw: &mut PacketWriter) {
        self.0.encode(pw);
        self.1.encode(pw);
        self.2.encode(pw);
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        Ok((i32::decode(pr)?, i32::decode(pr)?, i32::decode(pr)?))
    }
}

impl WireCodec for (i32, i32, i32, u8) {
    fn encode(&self, pw: &mut PacketWriter) {
        self.0.encode(pw);
        self.1.encode(pw);
        self.2.encode(pw);
        self.3.encode(pw);
    }
    fn decode(pr: &mut PacketReader<'_>) -> Result<Self, &'static str> {
        Ok((
            i32::decode(pr)?,
            i32::decode(pr)?,
            i32::decode(pr)?,
            u8::decode(pr)?,
        ))
    }
}
