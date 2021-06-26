#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{Menu, MenuItem, Window};

use app_state::AppState;
use plugin_host_lib::audio_io::{AudioIOService, AudioIOServiceResult, DevicesList};

mod app_state;
mod volume_publisher;

#[tauri::command]
fn list_devices_command(host_id: Option<String>) -> AudioIOServiceResult<DevicesList> {
  log::info!("Listing devices");
  AudioIOService::devices_list(host_id)
}

#[tauri::command]
fn list_hosts_command() -> Vec<String> {
  log::info!("Listing hosts");
  AudioIOService::hosts()
}

#[tauri::command]
fn subscribe_to_volume_command(state: tauri::State<AppState>, window: Window) {
  log::info!("Setting-up fake volume event emitter");
  let host = state.inner().host();
  std::thread::spawn(move || loop {
    let (volume_left, volume_right) = host.lock().unwrap().current_volume();
    let js_string = format!(
      "window.volume1={};window.volume2={};",
      volume_left, volume_right
    );
    // TODO fix this
    let _ = window.eval(&js_string);
    std::thread::sleep(std::time::Duration::from_millis(50));
  });
  log::info!("Volume event loop will emit volume every 100ms");
}

#[tauri::command]
fn unsubscribe_to_volume_command(_window: Window) {
  // TODO implement unsubscribe
  log::info!("Cleaning-up emitter");
}

#[tauri::command]
fn set_audio_driver_command(state: tauri::State<AppState>, host_id: String) {
  log::info!("Setting audio driver {}", host_id);
  state.set_host_id(host_id);
}

#[tauri::command]
fn set_input_device_command(state: tauri::State<AppState>, input_device_id: String) {
  log::info!("Setting input device {}", input_device_id);
  state.set_input_device_id(input_device_id);
}

#[tauri::command]
fn set_output_device_command(state: tauri::State<AppState>, output_device_id: String) {
  log::info!("Setting output device {}", output_device_id);
  state.set_output_device_id(output_device_id);
}

#[tauri::command]
fn set_input_file_command(state: tauri::State<AppState>, input_file: String) {
  log::info!("Setting audio input file {}", input_file);
  state.set_input_file(input_file);
}

#[tauri::command]
fn set_plugin_path_command(state: tauri::State<AppState>, path: String) {
  log::info!("Setting plugin path {}", path);
  state.set_plugin_path(path);
}

fn main() {
  wisual_logger::init_from_env();
  let mut plugin_host = plugin_host_lib::TestPluginHost::default();
  if let Err(err) = plugin_host.start() {
    log::error!("Failed to start host: {}", err);
  }
  let mut menus = Vec::new();
  menus.push(Menu::new(
    "plugin-host",
    vec![
      MenuItem::About(String::from("plugin-host")),
      MenuItem::Separator,
      MenuItem::Hide,
      MenuItem::HideOthers,
      MenuItem::ShowAll,
      MenuItem::Separator,
      MenuItem::Quit,
    ],
  ));

  tauri::Builder::default()
    .manage(AppState::new(plugin_host))
    .invoke_handler(tauri::generate_handler![
      set_audio_driver_command,
      set_input_device_command,
      set_output_device_command,
      set_input_file_command,
      set_plugin_path_command,
      list_devices_command,
      list_hosts_command,
      subscribe_to_volume_command,
      unsubscribe_to_volume_command,
    ])
    .menu(menus)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
