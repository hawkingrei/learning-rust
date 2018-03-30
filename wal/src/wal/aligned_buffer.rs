use alloc::raw_vec::RawVec;

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

struct AlignedBuffer {
    alignment_: usize,
    buf_: RawVec<u8>,
    capacity_: usize,
    cursize_: usize,
    bufstart_: usize,
}

impl AlignedBuffer {
    fn alignment(&mut self, alignment: usize) {
        self.alignment_ = alignment;
    }

    fn allocate_new_buffer(&mut self, requested_cacacity: usize, copy_data: bool) {
        assert!(self.alignment_ > 0);
        assert!((self.alignment_ & (self.alignment_ - 1)) == 0);
        if (copy_data && requested_cacacity < self.cursize_) {
            // If we are downsizing to a capacity that is smaller than the current
            // data in the buffer. Ignore the request.
            return;
        }

        let new_capacity = round_up(requested_cacacity, self.alignment_);
        self.buf_ = RawVec::with_capacity(new_capacity);
    }
}
