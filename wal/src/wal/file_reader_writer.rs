use wal::WritableFile;

#[derive(Debug)]
struct WritableFileWriter {
    writable_file_ : WritableFile;
    max_buffer_size_ :isize;
}   