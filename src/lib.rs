use std::io;
use crate::audio_data::AudioData;
use crate::audio_presentation::StereoAudioPresentation;
use crate::wav_binary::WavBinary;

mod wav_binary;
mod audio_data;
mod audio_presentation;

pub fn load_audio_presentation(path: &str) -> io::Result<StereoAudioPresentation> {
    let wavbin = WavBinary::from_file(path)?;
    let audiodata = AudioData::try_from(&wavbin)?;
    let presentation = StereoAudioPresentation::try_from(&audiodata)?;
    Ok(presentation)
}