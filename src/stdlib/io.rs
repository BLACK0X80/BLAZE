use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, BufReader, BufWriter, Seek, SeekFrom};
use std::path::Path;

pub struct BlazeFile {
    file: File,
}

impl BlazeFile {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self { file })
    }

    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self { file })
    }

    pub fn open_write<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)?;
        Ok(Self { file })
    }

    pub fn open_append<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)?;
        Ok(Self { file })
    }

    pub fn read_to_string(&mut self) -> io::Result<String> {
        let mut contents = String::new();
        self.file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn read_to_bytes(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn read_exact(&mut self, len: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; len];
        self.file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn write_str(&mut self, content: &str) -> io::Result<usize> {
        self.file.write(content.as_bytes())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.file.write(bytes)
    }

    pub fn write_line(&mut self, line: &str) -> io::Result<()> {
        self.file.write_all(line.as_bytes())?;
        self.file.write_all(b"\n")?;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }

    pub fn seek(&mut self, pos: u64) -> io::Result<u64> {
        self.file.seek(SeekFrom::Start(pos))
    }

    pub fn seek_from_current(&mut self, offset: i64) -> io::Result<u64> {
        self.file.seek(SeekFrom::Current(offset))
    }

    pub fn seek_from_end(&mut self, offset: i64) -> io::Result<u64> {
        self.file.seek(SeekFrom::End(offset))
    }

    pub fn metadata(&self) -> io::Result<std::fs::Metadata> {
        self.file.metadata()
    }

    pub fn size(&self) -> io::Result<u64> {
        Ok(self.metadata()?.len())
    }
}

pub struct BufferedReader {
    reader: BufReader<File>,
}

impl BufferedReader {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
        })
    }

    pub fn with_capacity<P: AsRef<Path>>(path: P, capacity: usize) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::with_capacity(capacity, file),
        })
    }

    pub fn read_line(&mut self) -> io::Result<String> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        Ok(line)
    }

    pub fn read_lines(&mut self) -> io::Result<Vec<String>> {
        let mut lines = Vec::new();
        loop {
            let mut line = String::new();
            let bytes_read = self.reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }
            lines.push(line);
        }
        Ok(lines)
    }

    pub fn read_to_string(&mut self) -> io::Result<String> {
        let mut contents = String::new();
        self.reader.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn read_to_bytes(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

pub struct BufferedWriter {
    writer: BufWriter<File>,
}

impl BufferedWriter {
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn with_capacity<P: AsRef<Path>>(path: P, capacity: usize) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::with_capacity(capacity, file),
        })
    }

    pub fn write_str(&mut self, content: &str) -> io::Result<usize> {
        self.writer.write(content.as_bytes())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writer.write(bytes)
    }

    pub fn write_line(&mut self, line: &str) -> io::Result<()> {
        self.writer.write_all(line.as_bytes())?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl Drop for BufferedWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    std::fs::read_to_string(path)
}

pub fn read_file_bytes<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    std::fs::read(path)
}

pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> io::Result<()> {
    std::fs::write(path, content)
}

pub fn write_file_bytes<P: AsRef<Path>>(path: P, bytes: &[u8]) -> io::Result<()> {
    std::fs::write(path, bytes)
}

pub fn append_file<P: AsRef<Path>>(path: P, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

pub fn delete_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    std::fs::remove_file(path)
}

pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    std::fs::copy(from, to)
}

pub fn rename_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
    std::fs::rename(from, to)
}

pub fn create_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    std::fs::create_dir(path)
}

pub fn create_directory_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    std::fs::create_dir_all(path)
}

pub fn delete_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    std::fs::remove_dir(path)
}

pub fn delete_directory_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    std::fs::remove_dir_all(path)
}

pub fn list_directory<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            entries.push(name.to_string());
        }
    }
    Ok(entries)
}

pub struct Console;

impl Console {
    pub fn print(message: &str) {
        print!("{}", message);
    }

    pub fn println(message: &str) {
        println!("{}", message);
    }

    pub fn read_line() -> io::Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    pub fn read_line_with_prompt(prompt: &str) -> io::Result<String> {
        print!("{}", prompt);
        io::stdout().flush()?;
        Self::read_line()
    }

    pub fn clear() {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().ok();
    }
}
