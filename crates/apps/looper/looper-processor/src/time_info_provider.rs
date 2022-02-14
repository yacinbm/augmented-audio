use derive_builder::Builder;
use mockall::automock;

use audio_processor_standalone::standalone_vst::vst;
use audio_processor_standalone::standalone_vst::vst::host::Host;
use audio_processor_standalone::standalone_vst::vst::plugin::HostCallback;
use augmented_playhead::{PlayHead, PlayHeadOptions};

#[derive(Builder)]
pub struct TimeInfo {
    tempo: Option<f64>,
    position_samples: f64,
    position_beats: Option<f64>,
}

impl TimeInfo {
    pub fn tempo(&self) -> Option<f64> {
        self.tempo
    }

    pub fn position_samples(&self) -> f64 {
        self.position_samples
    }

    pub fn position_beats(&self) -> Option<f64> {
        self.position_beats
    }
}

#[automock]
pub trait TimeInfoProvider {
    fn get_time_info(&self) -> TimeInfo;
    fn tick(&self);
}

pub struct TimeInfoProviderImpl {
    host_callback: Option<HostCallback>,
    playhead: PlayHead,
}

impl TimeInfoProvider for TimeInfoProviderImpl {
    fn get_time_info(&self) -> TimeInfo {
        let host_time_info = self
            .host_callback
            .map(|cb| {
                cb.get_time_info(
                    (vst::api::TimeInfoFlags::TEMPO_VALID | vst::api::TimeInfoFlags::PPQ_POS_VALID)
                        .bits(),
                )
            })
            .flatten()
            .map(|vst_time_info| TimeInfo {
                tempo: Some(vst_time_info.tempo),
                position_samples: vst_time_info.sample_pos,
                position_beats: Some(vst_time_info.ppq_pos),
            });

        host_time_info.unwrap_or_else(|| TimeInfo {
            tempo: self.playhead.options().tempo().map(|t| t as f64),
            position_samples: self.playhead.position_samples() as f64,
            position_beats: self
                .playhead
                .options()
                .tempo()
                .map(|_| self.playhead.position_beats()),
        })
    }

    fn tick(&self) {
        self.playhead.accept_samples(1);
    }
}

impl TimeInfoProviderImpl {
    pub fn new(host_callback: Option<HostCallback>) -> Self {
        TimeInfoProviderImpl {
            host_callback,
            playhead: PlayHead::new(PlayHeadOptions::new(None, None, None)),
        }
    }

    pub fn playhead(&self) -> &PlayHead {
        &self.playhead
    }

    pub fn set_tempo(&self, tempo: u32) {
        self.playhead.set_tempo(tempo);
    }

    pub fn set_sample_rate(&self, sample_rate: f32) {
        self.playhead.set_sample_rate(sample_rate);
    }
}

#[cfg(test)]
mod test {
    use audio_processor_testing_helpers::assert_f_eq;

    use super::*;

    #[test]
    fn test_time_info_provider_without_tempo_doesnt_move() {
        let time_info_provider = TimeInfoProviderImpl::new(None);
        time_info_provider.set_sample_rate(1000.0);
        time_info_provider.tick();
        time_info_provider.tick();
        time_info_provider.tick();
        time_info_provider.tick();
        assert!(time_info_provider
            .get_time_info()
            .position_beats()
            .is_none());
        assert_f_eq!(time_info_provider.get_time_info().position_samples(), 4.0);
    }

    #[test]
    fn test_time_info_provider_with_tempo_keep_track_of_beats() {
        let time_info_provider = TimeInfoProviderImpl::new(None);
        time_info_provider.set_sample_rate(100.0);
        time_info_provider.set_tempo(60);

        let time_info = time_info_provider.get_time_info();
        assert!(time_info.position_beats().is_some());
        assert_eq!(time_info.position_beats(), Some(0.0));
        assert_eq!(time_info.position_samples(), 0.0);

        for _i in 0..100 {
            time_info_provider.tick();
        }
        let time_info = time_info_provider.get_time_info();
        assert!(time_info.position_beats().is_some());
        assert_f_eq!(time_info.position_beats().unwrap(), 1.0);
        assert_f_eq!(time_info.position_samples(), 100.0);
    }
}