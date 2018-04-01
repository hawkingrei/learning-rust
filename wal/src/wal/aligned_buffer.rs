use alloc::raw_vec::RawVec;
use std::cmp::min;
use std::fmt::{self, Debug, Formatter};
use std::mem;
use std::mem::align_of;
use std::{ptr, slice};
#[inline]
fn truncate_to_page_boundary(page_size: usize, s: usize) -> usize {
    assert!((s % page_size) == 0);
    s - (s & (page_size - 1))
}

#[inline]
fn round_up(x: usize, y: usize) -> usize {
    return ((x + y - 1) / y) * y;
}
#[inline]
fn Rounddown(x: usize, y: usize) -> usize {
    return (x / y) * y;
}

pub struct AlignedBuffer {
    alignment_: usize,
    buf_: RawVec<u8>,
    capacity_: usize,
    cursize_: usize,
    bufstart_: *mut u8,
}

impl Default for AlignedBuffer {
    fn default() -> Self {
        AlignedBuffer {
            alignment_: 0,
            buf_: RawVec::with_capacity(1),
            capacity_: 0,
            cursize_: 0,
            bufstart_: ptr::null_mut::<u8>(),
        }
    }
}

impl AlignedBuffer {
    fn get_alignment(&self) -> usize {
        return self.alignment_;
    }

    pub fn get_capacity(&self) -> usize {
        return self.capacity_;
    }

    pub fn get_current_size(&self) -> usize {
        return self.cursize_;
    }

    fn alignment(&mut self, alignment: usize) {
        self.alignment_ = alignment;
    }

    pub fn allocate_new_buffer(&mut self, requested_cacacity: usize, copy_data: bool) {
        assert!(self.alignment_ > 0);
        assert!((self.alignment_ & (self.alignment_ - 1)) == 0);
        if (copy_data && requested_cacacity < self.cursize_) {
            // If we are downsizing to a capacity that is smaller than the current
            // data in the buffer. Ignore the request.
            return;
        }

        let new_capacity = round_up(requested_cacacity, self.alignment_);
        let new_buf = RawVec::with_capacity_zeroed(new_capacity + 1);
        //let new_bufstart_offset = self.buf_.ptr().align_offset(self.alignment_);
        //let new_bufstart;
        unsafe {
            //new_bufstart = self.buf_.ptr().offset(new_bufstart_offset as isize);
            if copy_data {
                //ptr::write()
                ptr::copy_nonoverlapping(new_buf.ptr(), self.bufstart_, self.cursize_);
            } else {
                self.cursize_ = 0;
            }
        }

        self.bufstart_ = new_buf.ptr();
        self.capacity_ = new_capacity;
        self.buf_ = new_buf;
    }

    fn append(&mut self, src: Vec<u8>) -> usize {
        let append_size = src.len();
        assert!(self.capacity_ > self.cursize_);
        let buffer_remaining = self.capacity_ - self.cursize_;
        let to_copy = min(append_size, buffer_remaining);
        if to_copy > 0 {
            unsafe {
                ptr::copy_nonoverlapping(
                    src.as_ptr(),
                    self.bufstart_.offset(self.cursize_ as isize),
                    to_copy,
                );
            }
            self.cursize_ += to_copy;
        }
        to_copy
    }

    fn read(&mut self, offset: usize, read_size: usize) -> Vec<u8> {
        let mut result = vec![0; read_size];
        let mut to_read = 0;
        if (offset < self.cursize_) {
            to_read = min(self.cursize_ - offset, read_size);
        }
        if (to_read > 0) {
            unsafe {
                ptr::copy_nonoverlapping(
                    self.bufstart_.offset(offset as isize),
                    result.as_mut_ptr(),
                    to_read,
                );
            }
        }
        result
    }

    fn pad_to_aligment_with(&mut self, padding: u8) {
        let total_size = round_up(self.cursize_, self.alignment_);
        let pad_size = total_size - self.cursize_;
        if pad_size > 0 {
            unsafe {
                ptr::write_bytes(
                    self.bufstart_.offset(self.cursize_ as isize),
                    padding,
                    pad_size,
                );
            }
            self.cursize_ += pad_size;
        }
    }

    fn pad_with(&mut self, pad_size: usize, padding: u8) {
        assert!((pad_size + self.cursize_) <= self.capacity_);
        unsafe {
            ptr::write_bytes(
                self.bufstart_.offset(self.cursize_ as isize),
                padding,
                pad_size,
            );
        }
        self.cursize_ += pad_size;
    }

    // After a partial flush move the tail to the beginning of the buffer
    fn refit_tail(&mut self, tail_offset: usize, tail_size: usize) {
        if (tail_size > 0) {
            unsafe {
                ptr::copy(
                    self.bufstart_,
                    self.bufstart_.offset(tail_offset as isize),
                    tail_size,
                );
            }
        }
        self.cursize_ = tail_size;
    }

    fn size(&mut self, cursize: usize) {
        self.cursize_ = cursize;
    }
}

impl Debug for AlignedBuffer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "AlignedBuffer[alignment: {}, start: {:?},align start: {:?}, buf: {}]",
            self.alignment_,
            self.buf_.ptr(),
            self.bufstart_,
            escape(unsafe { slice::from_raw_parts(self.buf_.ptr(), self.buf_.cap()) })
        )
    }
}

pub fn escape(data: &[u8]) -> String {
    let mut escaped = Vec::with_capacity(data.len() * 4);
    for &c in data {
        match c {
            b'\n' => escaped.extend_from_slice(br"\n"),
            b'\r' => escaped.extend_from_slice(br"\r"),
            b'\t' => escaped.extend_from_slice(br"\t"),
            b'"' => escaped.extend_from_slice(b"\\\""),
            b'\\' => escaped.extend_from_slice(br"\\"),
            _ => {
                if c >= 0x20 && c < 0x7f {
                    // c is printable
                    escaped.push(c);
                } else {
                    escaped.push(b'\\');
                    escaped.push(b'0' + (c >> 6));
                    escaped.push(b'0' + ((c >> 3) & 7));
                    escaped.push(b'0' + (c & 7));
                }
            }
        }
    }
    escaped.shrink_to_fit();
    unsafe { String::from_utf8_unchecked(escaped) }
}

#[test]
fn test_aligned_buffer() {
    let mut buf: AlignedBuffer = Default::default();
    buf.alignment(4);
    buf.allocate_new_buffer(16, false);
    let appended = buf.append(String::from("abc").into_bytes());
    let result = buf.read(0, appended);
    assert_eq!(result.len(), 3);
    unsafe {
        assert_eq!(String::from_utf8_unchecked(result), String::from("abc"));
    }
}
