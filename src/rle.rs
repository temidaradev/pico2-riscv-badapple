pub const FRAME_SIZE: usize = 1024;

#[inline(always)]
pub fn put_bytes(
    frame_buffer: &mut [u8; FRAME_SIZE],
    byte: u8,
    count: i32,
    bytes_written: &mut usize,
) {
    let count_usize = count as usize;
    let end = (*bytes_written + count_usize).min(FRAME_SIZE);
    frame_buffer[*bytes_written..end].fill(byte);
    *bytes_written = end;
}

#[inline(always)]
pub fn decode_rle(
    frame_buffer: &mut [u8; FRAME_SIZE],
    c: u8,
    runlength: &mut i32,
    c_to_dup: &mut i32,
    bytes_written: &mut usize,
) {
    if *c_to_dup == -1 {
        if c == 0x55 || c == 0xaa {
            *c_to_dup = c as i32;
        } else {
            put_bytes(frame_buffer, c, 1, bytes_written);
        }
    } else {
        if *runlength == -1 {
            if c == 0 {
                put_bytes(frame_buffer, (*c_to_dup & 0xff) as u8, 1, bytes_written);
                *c_to_dup = -1;
            } else if (c & 0x80) == 0 {
                let val = if *c_to_dup == 0x55 { 0 } else { 255 };
                put_bytes(frame_buffer, val, c as i32, bytes_written);
                *c_to_dup = -1;
            } else {
                *runlength = (c & 0x7f) as i32;
            }
        } else {
            *runlength |= (c as i32) << 7;
            let val = if *c_to_dup == 0x55 { 0 } else { 255 };
            put_bytes(frame_buffer, val, *runlength, bytes_written);
            *c_to_dup = -1;
            *runlength = -1;
        }
    }
}

pub struct RleDecoder {
    pub src_pos: usize,
    pub runlength: i32,
    pub c_to_dup: i32,
    pub bytes_written: usize,
}

impl RleDecoder {
    pub fn new() -> Self {
        Self {
            src_pos: 0,
            runlength: -1,
            c_to_dup: -1,
            bytes_written: 0,
        }
    }

    pub fn reset(&mut self) {
        self.src_pos = 0;
        self.runlength = -1;
        self.c_to_dup = -1;
        self.bytes_written = 0;
    }

    pub fn decode_frame(&mut self, frame_buffer: &mut [u8; FRAME_SIZE], rle_data: &[u8]) -> bool {
        if self.src_pos >= rle_data.len() {
            self.reset();
        }

        while self.bytes_written < FRAME_SIZE && self.src_pos < rle_data.len() {
            decode_rle(
                frame_buffer,
                rle_data[self.src_pos],
                &mut self.runlength,
                &mut self.c_to_dup,
                &mut self.bytes_written,
            );
            self.src_pos += 1;
        }

        if self.bytes_written == FRAME_SIZE {
            self.bytes_written = 0;
            true
        } else {
            false
        }
    }
}
