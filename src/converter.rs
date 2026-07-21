use anyhow::{Context, Result};
use js_sys::Uint8Array;
use std::cell::RefCell;
use std::io::{Cursor, Write};
use std::rc::Rc;
use wasm_bindgen_futures::JsFuture;
use web_sys::File;
use zip::ZipWriter;
use zip::write::FileOptions;

pub type ZipData = Vec<u8>;

/// A custom writer that owns its buffer and can be safely boxed.
/// This avoids lifetime issues with Cursor which borrows its data.
pub struct VecWriter {
    pub data: Rc<RefCell<ZipData>>,
}

#[derive(Debug, Clone)]
pub struct ZipFile {
    pub name: String,
    pub data: Vec<u8>,
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
pub fn convert_rar_to_zip(rar_bytes: &[u8]) -> Result<ZipData> {
    // Parse the RAR archive from bytes
    let archive = rars::ArchiveReader::read(rar_bytes).context("Failed to parse RAR archive")?;

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
        .context("Failed to extract RAR archive")?;

    // Create ZIP archive and write all extracted files
    let mut zip_buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut zip_buf));

        for (filename, file_data_rc) in files.into_inner() {
            // Borrow the data from the RefCell (Rc is dropped at end of loop)
            let file_data = file_data_rc.borrow().clone();

            let options: FileOptions<()> =
                FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            zip.start_file(&filename, options)
                .with_context(|| format!("Failed to create ZIP entry '{}'", filename))?;

            zip.write_all(&file_data)
                .with_context(|| format!("Failed to write file '{}' to ZIP", filename))?;
        }

        zip.finish().context("Failed to finalize ZIP archive")?;
    }

    Ok(zip_buf)
}

/// Read a File as bytes
pub async fn read_file_as_bytes(file: &File) -> Result<Vec<u8>> {
    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| anyhow::anyhow!("Failed to read file as array buffer"))?;

    let array = Uint8Array::new(&array_buffer);
    Ok(array.to_vec())
}

/// Convert RAR file to ZIP with error handling
pub async fn convert_file(file: &File) -> Result<ZipFile> {
    let rar_bytes = read_file_as_bytes(file).await?;
    let data = convert_rar_to_zip(&rar_bytes)?;
    let name = file
        .name()
        .trim_end_matches(".rar")
        .trim_end_matches(".RAR")
        .to_string()
        + ".zip";
    Ok(ZipFile { name, data })
}
