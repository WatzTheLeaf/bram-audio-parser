use std::io;
use std::io::{Error, ErrorKind};
use crate::wav_binary::WavBinary;

#[derive(Debug, Clone)]
pub(crate) struct AudioData {
    pub samples: Vec<i16>,
    pub channels: u16,
    pub sample_rate: u32
}

impl TryFrom<&WavBinary> for AudioData {
    type Error = Error;

    fn try_from(wav: &WavBinary) -> Result<Self, Self::Error> {
        if !wav.check() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "not a valid wav file"
            ));
        }
        let (channels, sample_rate) = Self::read_format_info(&wav.data)?;
        let samples = Self::extract_samples(&wav.data)?;

        Ok(AudioData {
            samples,
            channels,
            sample_rate,
        })
    }
}

impl AudioData {
    fn find_data_chunk(data: &[u8]) -> Option<usize> {
        let mut pos = 12;
        while pos + 8 <= data.len() {
            let chunk_id = &data[pos..pos + 4];
            let chunk_size = u32::from_le_bytes([
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
            ]) as usize;
            if chunk_id == b"data" {
                return Some(pos);
            }
            pos += 8 + chunk_size;
        }
        None
    }

    fn read_format_info(data: &[u8]) -> io::Result<(u16, u32)> {
        let mut pos = 12;
        while pos + 8 < data.len() {
            let chunk_id = &data[pos..pos + 4];
            if chunk_id == b"fmt " {
                let channels = u16::from_le_bytes([
                    data[pos + 10],
                    data[pos + 11],
                ]);
                let sample_rate = u32::from_le_bytes([
                    data[pos + 12],
                    data[pos + 13],
                    data[pos + 14],
                    data[pos + 15],
                ]);
                return Ok((channels, sample_rate));
            }
            let chunk_size = u32::from_le_bytes([
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
            ]) as usize;

            pos += 8 + chunk_size;
        }
        Err(Error::new(
            ErrorKind::InvalidData,
            "no chunk format found"
        ))
    }

    fn extract_samples(data: &[u8]) -> io::Result<Vec<i16>> {
        let data_pos = Self::find_data_chunk(data)
            .ok_or_else(|| Error::new(
                ErrorKind::InvalidData,
                "no data chunk found"
            ))?;
        if data_pos + 8 > data.len() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "incomplete data chunk"
            ));
        }

        let data_size = u32::from_le_bytes([
            data[data_pos + 4],
            data[data_pos + 5],
            data[data_pos + 6],
            data[data_pos + 7],
        ]) as usize;

        let audio_start = data_pos + 8;
        let audio_end = audio_start + data_size;

        if audio_end > data.len() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Données audio incomplètes"
            ));
        }

        let audio_bytes = &data[audio_start..audio_end];
        Self::bytes_to_i16_samples(audio_bytes)
    }
    
    fn bytes_to_i16_samples(bytes: &[u8]) -> io::Result<Vec<i16>> {

        if bytes.len() % 2 != 0 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "invalid binary data"
            ));
        }

        let num_samples = bytes.len() / 2;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let byte_index = i * 2;
            let sample = i16::from_le_bytes([
                bytes[byte_index],
                bytes[byte_index + 1],
            ]);
            samples.push(sample);
        }

        Ok(samples)
    }
}

#[cfg(test)]
mod audio_data_tests {
    use std::io::ErrorKind;
    use crate::audio_data::AudioData;
    use crate::wav_binary::WavBinary;

    #[test]
    fn create_audio_data_from_wavbinary() {
        let wav_data = vec![
            b'R', b'I', b'F', b'F',
            0x24, 0x00, 0x00, 0x00,
            b'W', b'A', b'V', b'E',
            b'f', b'm', b't', b' ',
            0x10, 0x00, 0x00, 0x00,
            0x01, 0x00,
            0x02, 0x00,
            0x44, 0xAC, 0x00, 0x00,
            0x10, 0xB1, 0x02, 0x00,
            0x04, 0x00,
            0x10, 0x00,
            b'd', b'a', b't', b'a',
            0x08, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0x7F,
            0x00, 0x80, 0x01, 0x00,
        ];
        let wav = WavBinary { data: wav_data };
        let result = AudioData::try_from(&wav);
        assert!(result.is_ok());
        let audio = result.unwrap();
        assert_eq!(audio.channels, 2);
        assert_eq!(audio.sample_rate, 44100);
        assert_eq!(audio.samples.len(), 4);
    }

    #[test]
    fn create_audio_data_from_wavbinary_fails_if_check_fails() {
        let wav_data = vec![
            b'R', b'I', b'F', b'F',
            0x24, 0x00, 0x00, 0x00,
            b'N', b'o', b'n', b'e',
            b'f', b'm', b't', b' ',
            0x10, 0x00, 0x00, 0x00,
            0x01, 0x00,
            0x02, 0x00,
            0x44, 0xAC, 0x00, 0x00,
            0x10, 0xB1, 0x02, 0x00,
            0x04, 0x00,
            0x10, 0x00,
            b'd', b'a', b't', b'a',
            0x08, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0x7F,
            0x00, 0x80, 0x01, 0x00,
        ];
        let wav = WavBinary { data: wav_data };
        let result = AudioData::try_from(&wav);
        assert!(result.is_err());
    }

    #[test]
    fn find_data_chuncks_works_if_present() {
        let wav_data = vec![
            b'R', b'I', b'F', b'F',
            0x24, 0x00, 0x00, 0x00,
            b'W', b'A', b'V', b'E',
            b'f', b'm', b't', b' ',
            0x10, 0x00, 0x00, 0x00,
            0x01, 0x00,
            0x02, 0x00,
            0x44, 0xAC, 0x00, 0x00,
            0x10, 0xB1, 0x02, 0x00,
            0x04, 0x00,
            0x10, 0x00,
            b'd', b'a', b't', b'a',
            0x04, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0x7F,
        ];
        let result = AudioData::find_data_chunk(&wav_data);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 36);
    }

    #[test]
    fn find_data_chuncks_returns_none_if_not_present() {
        let wav_data = vec![
            b'R', b'I', b'F', b'F',
            0x24, 0x00, 0x00, 0x00,
            b'W', b'A', b'V', b'E',
            b'f', b'm', b't', b' ',
            0x10, 0x00, 0x00, 0x00,
            0x01, 0x00,
            0x02, 0x00,
        ];
        let result = AudioData::find_data_chunk(&wav_data);
        assert!(result.is_none());
    }

    #[test]
    fn read_format_info_works_if_valid() {
        let wav_data = vec![
            b'R', b'I', b'F', b'F',
            0x24, 0x00, 0x00, 0x00,
            b'W', b'A', b'V', b'E',
            b'f', b'm', b't', b' ',
            0x10, 0x00, 0x00, 0x00,
            0x01, 0x00,
            0x02, 0x00,
            0x44, 0xAC, 0x00, 0x00,
            0x10, 0xB1, 0x02, 0x00,
            0x04, 0x00,
            0x10, 0x00,
        ];
        let result = AudioData::read_format_info(&wav_data);
        assert!(result.is_ok());
        let (channels, sample_rate) = result.unwrap();
        assert_eq!(channels, 2);
        assert_eq!(sample_rate, 44100);
    }

    #[test]
    fn read_format_info_returns_error_if_invalid() {
        let wav_data = vec![
            b'R', b'I', b'F', b'F',
            0x24, 0x00, 0x00, 0x00,
            b'W', b'A', b'V', b'E',
            b'd', b'a', b't', b'a',
            0x04, 0x00, 0x00, 0x00,
        ];
        let result = AudioData::read_format_info(&wav_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn extract_samples_works_with_valid_data() {
        let wav_data = vec![
            b'R', b'I', b'F', b'F',
            0x24, 0x00, 0x00, 0x00,
            b'W', b'A', b'V', b'E',
            b'd', b'a', b't', b'a',
            0x08, 0x00, 0x00, 0x00,
            0x00, 0x00,
            0xFF, 0x7F,
            0x00, 0x80,
            0x01, 0x00,
        ];
        let result = AudioData::extract_samples(&wav_data);
        assert!(result.is_ok());
        let samples = result.unwrap();
        assert_eq!(samples.len(), 4);
        assert_eq!(samples[0], 0);
        assert_eq!(samples[1], 32767);
        assert_eq!(samples[2], -32768);
        assert_eq!(samples[3], 1);
    }

    #[test]
    fn extract_samples_returns_error_if_invalid_no_data() {
        let wav_data = vec![
            b'R', b'I', b'F', b'F',
            0x24, 0x00, 0x00, 0x00,
            b'W', b'A', b'V', b'E',
        ];
        let result = AudioData::extract_samples(&wav_data);
        assert!(result.is_err());
    }

    #[test]
    fn bytes_to_i16_samples_return_error_if_invalid() {
        let bytes = vec![0x00, 0x00, 0xFF];
        let result = AudioData::bytes_to_i16_samples(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn bytes_to_i16_samples_return_expected_values() {
        let bytes = vec![0x00, 0x00, 0xFF, 0x7F, 0x00, 0x80, 0x01, 0x00];
        let result = AudioData::bytes_to_i16_samples(&bytes);
        assert!(result.is_ok());
        let samples = result.unwrap();
        assert_eq!(samples.len(), 4);
        assert_eq!(samples[0], 0);
        assert_eq!(samples[1], 32767);
        assert_eq!(samples[2], -32768);
        assert_eq!(samples[3], 1);
    }
}