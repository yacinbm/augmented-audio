use std::sync::{Arc, Mutex};
use std::time::Duration;

use plugin_host_lib::audio_io::test_plugin_host::TestPluginHost;
use plugin_host_lib::audio_io::AudioIOService;

use crate::config::AppConfig;
use crate::volume_publisher::VolumePublisherService;

pub type AppStateRef = Arc<Mutex<AppState>>;

pub struct AppState {
  host: Arc<Mutex<TestPluginHost>>,
  audio_io_service: AudioIOService,
  volume_publisher_service: VolumePublisherService<TestPluginHost>,
}

impl AppState {
  pub fn new(host: plugin_host_lib::TestPluginHost, app_config: AppConfig) -> Self {
    let host = Arc::new(Mutex::new(host));
    AppState {
      host: host.clone(),
      audio_io_service: AudioIOService::new(host.clone(), app_config.storage_config.clone()),
      volume_publisher_service: VolumePublisherService::new(host, Duration::from_millis(100)),
    }
  }

  pub fn try_reload(&mut self) {
    match self.audio_io_service.reload() {
      Ok(_) => {
        log::info!("Reloaded Audio IO configuration from disk");
      }
      Err(err) => {
        log::warn!("Failed to load Audio IO configuration from disk: {:?}", err);
      }
    }
  }

  pub fn host(&self) -> &Arc<Mutex<TestPluginHost>> {
    &self.host
  }

  pub fn audio_io_service(&mut self) -> &mut AudioIOService {
    &mut self.audio_io_service
  }

  pub fn volume_publisher_service(&mut self) -> &mut VolumePublisherService<TestPluginHost> {
    &mut self.volume_publisher_service
  }
}
