use std::io;
use crate::audio_data::AudioData;
use crate::audio_presentation::{RatedAudioData, StereoAudioPresentation};
use crate::wav_binary::WavBinary;

mod wav_binary;
mod audio_data;
mod audio_presentation;

pub fn load_presentation(path: &str, rate: u32) -> io::Result<StereoAudioPresentation> {
    let wavbin = WavBinary::from_file(path)?;
    let audiodata = AudioData::try_from(&wavbin)?;
    let ratedaudiodata = RatedAudioData::new(&audiodata, rate);
    let presentation = StereoAudioPresentation::try_from(&ratedaudiodata)?;
    Ok(presentation)
}