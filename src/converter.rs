use std::io::{Cursor, Write};
use std::rc::Rc;
use std::cell::RefCell;
use zip::ZipWriter;
use zip::write::FileOptions;

/// A custom writer that owns its buffer and can be safely boxed.
/// This avoids lifetime issues with Cursor which borrows its data.
pub struct VecWriter {
    pub data: Rc<RefCell<Vec<u8>>>,
}

impl Write for VecWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Convert RAR archive bytes to ZIP archive bytes
pub fn convert_rar_to_zip(rar_bytes: &[u8]) -> Result<Vec<u8>, String> {
    // Parse the RAR archive from bytes
    let archive = rars::ArchiveReader::read(rar_bytes)
        .map_err(|e| format!("Failed to parse RAR archive: {}", e))?;

    // Collect extracted files as (filename, data)
    let files = RefCell::new(Vec::new());

    // Extract all files from RAR
    archive
        .extract_to(None, |meta: &rars::ExtractedEntryMeta| {
            // Skip directories
            if meta.is_directory {
                return Ok(Box::new(std::io::sink()));
            }

            // Create owned buffer for this file wrapped in Rc<RefCell<_>>
            let file_data = Rc::new(RefCell::new(Vec::new()));

            let filename = String::from_utf8_lossy(meta.name_bytes()).to_string();
            files.borrow_mut().push((filename, file_data.clone()));

            // Return boxed writer that writes to the Rc<RefCell<Vec<u8>>>
            Ok(Box::new(VecWriter { data: file_data }))
        })
        .map_err(|e| format!("Failed to extract RAR archive: {}", e))?;

    // Create ZIP archive and write all extracted files
    let mut zip_buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut zip_buf));

        for (filename, file_data_rc) in files.into_inner() {
            // Borrow the data from the RefCell (Rc is dropped at end of loop)
            let file_data = file_data_rc.borrow().clone();
            
            let options: FileOptions<()> = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            zip.start_file(&filename, options)
                .map_err(|e| format!("Failed to create ZIP entry '{}': {}", filename, e))?;

            zip.write_all(&file_data)
                .map_err(|e| format!("Failed to write file '{}' to ZIP: {}", filename, e))?;
        }

        zip.finish()
            .map_err(|e| format!("Failed to finalize ZIP archive: {}", e))?;
    }

    Ok(zip_buf)
}
