use std::io::{Error, ErrorKind};
use crate::audio_data::AudioData;

#[derive(Debug)]
pub struct StereoAudioPresentation {
    pub left_channel_points: Vec<f32>,
    pub right_channel_points: Vec<f32>,
}

pub(crate) struct RatedAudioData {
    pub audio_data: AudioData,
    pub sample_rate: u32,
}

impl RatedAudioData {
    pub(crate) fn new(audio_data: &AudioData, sample_rate: u32) -> Self {
        Self {
            audio_data: audio_data.clone(), sample_rate
        }
    }
}

impl TryFrom<&RatedAudioData> for StereoAudioPresentation {
    type Error = Error;

    fn try_from(rated_audio_data: &RatedAudioData) -> Result<Self, Self::Error> {
        let samples = &rated_audio_data.audio_data;
        if samples.channels != 1 && samples.channels != 2 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "only mono or stereo audio is supported"
            ));
        }
        let samples_per_interval = samples.sample_rate / rated_audio_data.sample_rate;
        let total_frames = samples.samples.len() / samples.channels as usize;
        let num_points = (total_frames + samples_per_interval as usize - 1) / samples_per_interval as usize;
        let mut left_channel_points = Vec::with_capacity(num_points);
        let mut right_channel_points = Vec::with_capacity(num_points);
        let mut frame_index = 0;
        while frame_index < total_frames {
            let sample_index = frame_index * samples.channels as usize;

            let first_sample = samples.samples[sample_index];

            let second_sample = if samples.channels == 2 {
                samples.samples[sample_index + 1]
            } else {
                first_sample
            };

            let first_normalized = (first_sample as f32 + 32768.0) / 65535.0;
            let second_normalized = (second_sample as f32 + 32768.0) / 65535.0;

            left_channel_points.push(first_normalized);
            right_channel_points.push(second_normalized);

            frame_index += samples_per_interval as usize;
        }
        Ok(StereoAudioPresentation {
            left_channel_points,
            right_channel_points,
        })
    }
}

#[cfg(test)]
mod audio_presentation_tests {
    use std::io::ErrorKind;
    use crate::audio_data::AudioData;
    use crate::audio_presentation::{RatedAudioData, StereoAudioPresentation};

    #[test]
    fn create_audio_presentation_from_audiodata_stereo() {
        let audio_data = AudioData {
            samples: vec![0, 0, 32767, -32768, -32768, 32767, 16384, -16384],
            channels: 2,
            sample_rate: 10,
        };
        let rated_audio_data = RatedAudioData {audio_data, sample_rate: 5};
        let result = StereoAudioPresentation::try_from(&rated_audio_data);
        assert!(result.is_ok());
        let presentation = result.unwrap();
        assert_eq!(presentation.left_channel_points.len(), 2);
        assert_eq!(presentation.right_channel_points.len(), 2);
        assert!((presentation.left_channel_points[0] - 0.5).abs() < 0.001);
        assert!((presentation.right_channel_points[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn create_audio_presentation_from_audiodata_mono() {
        let audio_data = AudioData {
            samples: vec![0, 0, 32767, -32768, -32768, 32767, 16384, -16384],
            channels: 1,
            sample_rate: 10
        };
        let rated_audio_data = RatedAudioData {audio_data, sample_rate: 5};
        let result = StereoAudioPresentation::try_from(&rated_audio_data);
        assert!(result.is_ok());
        let presentation = result.unwrap();
        assert_eq!(presentation.left_channel_points.len(), 4);
        assert_eq!(presentation.right_channel_points.len(), 4);
        assert!((presentation.left_channel_points[0] - 0.5).abs() < 0.001);
        assert!((presentation.right_channel_points[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn create_audio_presentation_from_audiodata_fail_too_many_chanels() {
        let audio_data = AudioData {
            samples: vec![0, 0, 32767, -32768, -32768, 32767, 16384, -16384],
            channels: 3,
            sample_rate: 10,
        };
        let rated_audio_data = RatedAudioData {audio_data, sample_rate: 5};
        let result = StereoAudioPresentation::try_from(&rated_audio_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);
    }
}