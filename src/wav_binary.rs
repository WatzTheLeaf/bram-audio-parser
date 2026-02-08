use std::{fs, io};
use std::io::{Error, ErrorKind};
use std::path::Path;

#[derive(Debug)]
pub(crate) struct WavBinary {
    pub data: Vec<u8>,
}

impl WavBinary {

    pub(crate) fn from_file(path: &str) -> io::Result<Self> {
        let path_object = Path::new(path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"));
        if !path_object {
             return Err(Error::new(ErrorKind::InvalidInput, "not a wav file"))
        }
        let data = fs::read(path)?;
        Ok(WavBinary { data })
    }

    pub(crate) fn check(&self) -> bool {
        self.data.len() >= 12
            && &self.data[0..4] == b"RIFF"
            && &self.data[8..12] == b"WAVE"
    }

}

#[cfg(test)]
mod wav_binary_tests {
    use std::fs;
    use std::io::{ErrorKind, Write};
    use crate::wav_binary::WavBinary;

    #[test]
    fn check_wavbinary_is_valid() {
        let mut data: Vec<u8> = Vec::new();
        data.extend_from_slice(b"RIFF");
        data.extend_from_slice(&[0, 0, 0, 0]);
        data.extend_from_slice(b"WAVE");
        let my_struct = WavBinary { data };
        assert_eq!(my_struct.check(), true);
    }

    #[test]
    fn check_wavbinary_is_invalid_riff() {
        let mut data: Vec<u8> = Vec::new();
        data.extend_from_slice(b"XRdz");
        data.extend_from_slice(&[0, 0, 0, 0]);
        data.extend_from_slice(b"WAVE");
        let my_struct = WavBinary { data };
        assert_eq!(my_struct.check(), false);
    }

    #[test]
    fn check_wavbinary_is_invalid_wave() {
        let mut data: Vec<u8> = Vec::new();
        data.extend_from_slice(b"RIFF");
        data.extend_from_slice(&[0, 0, 0, 0]);
        data.extend_from_slice(b"abcd");
        let my_struct = WavBinary { data };
        assert_eq!(my_struct.check(), false);
    }

    #[test]
    fn check_wavbinary_is_invalid_size() {
        let mut data: Vec<u8> = Vec::new();
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
        let my_struct = WavBinary { data };
        assert_eq!(my_struct.check(), false);
    }

    #[test]
    fn check_wavbinary_is_invalid_empty_data() {
        let my_struct = WavBinary { data: Vec::new() };
        assert_eq!(my_struct.check(), false);
    }

    #[test]
    fn load_data_from_file_works() {
        let temp_path = create_temp_wav_file("mytest.wav");
        let result = WavBinary::from_file(temp_path.to_str().unwrap());
        assert!(result.is_ok());
        let wav = result.unwrap();
        assert!(wav.check());
        cleanup_temp_file(&temp_path);
    }

    #[test]
    fn load_data_from_file_works_with_uppercase_extension() {
        let temp_path = create_temp_wav_file("mytestuppercase.WAV");
        let result = WavBinary::from_file(temp_path.to_str().unwrap());
        assert!(result.is_ok());
        let wav = result.unwrap();
        assert!(wav.check());
        cleanup_temp_file(&temp_path);
    }

    #[test]
    fn load_data_from_file_fails_on_invalid_extension() {
        let temp_path = create_temp_file("failure.dat", b"RIFF\x00\x00\x00\x00WAVE");
        let result = WavBinary::from_file(temp_path.to_str().unwrap());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
        cleanup_temp_file(&temp_path);
    }

    #[test]
    fn load_data_from_file_fails_on_nonexistent_file() {
        let result = WavBinary::from_file("./nonexistent_file.wav");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
    }

    fn create_temp_file(filename: &str, content: &[u8]) -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(filename);
        fs::write(&path, content).unwrap();
        path
    }

    fn create_temp_wav_file(filename: &str) -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(filename);

        let mut file = fs::File::create(&path).unwrap();

        // WAV minimal (Claude 3 generated)
        file.write_all(b"RIFF").unwrap();
        file.write_all(&36u32.to_le_bytes()).unwrap();
        file.write_all(b"WAVE").unwrap();
        file.write_all(b"fmt ").unwrap();
        file.write_all(&16u32.to_le_bytes()).unwrap();
        file.write_all(&1u16.to_le_bytes()).unwrap();
        file.write_all(&2u16.to_le_bytes()).unwrap();
        file.write_all(&44100u32.to_le_bytes()).unwrap();
        file.write_all(&176400u32.to_le_bytes()).unwrap();
        file.write_all(&4u16.to_le_bytes()).unwrap();
        file.write_all(&16u16.to_le_bytes()).unwrap();
        file.write_all(b"data").unwrap();
        file.write_all(&0u32.to_le_bytes()).unwrap();

        path
    }

    fn cleanup_temp_file(path: &std::path::PathBuf) {
        let _ = fs::remove_file(path); // Be careful with this
    }

}