use std::fs::File;
use std::io::{BufWriter, Write};
use zip::read::ZipArchive;

pub fn unzip(input_path: &str, output_path: &str) -> std::io::Result<()> {
    let file = File::open(input_path)?;
    let mut archive = ZipArchive::new(file)?;

    if archive.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No files in ZIP archive",
        ));
    }

    let mut zip_file = archive.by_index(0).expect("Failed to access ZIP entry");
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);

    // Copy the contents of the file to the output file
    std::io::copy(&mut zip_file, &mut writer)?;
    writer.flush()?;
    println!("Unzip completed.");
    Ok(())
}
