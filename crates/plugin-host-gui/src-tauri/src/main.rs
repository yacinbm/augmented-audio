#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};

use tauri::{GlobalWindowEvent, /* Menu, MenuItem, */ WindowEvent};

use app_state::AppState;
use commands::*;

mod app_state;
mod commands;
mod models;
mod volume_publisher;

fn main() {
  wisual_logger::init_from_env();
  let mut plugin_host = plugin_host_lib::TestPluginHost::default();
  if let Err(err) = plugin_host.start() {
    log::error!("Failed to start host: {}", err);
  }
  // let main_menus = vec![
  //   MenuItem::About(String::from("plugin-host")),
  //   MenuItem::Separator,
  //   MenuItem::Hide,
  //   MenuItem::HideOthers,
  //   MenuItem::ShowAll,
  //   MenuItem::Separator,
  //   MenuItem::Quit,
  // ];
  // let mut main_menu = Menu::new();
  // for item in main_menus {
  //   main_menu = main_menu.add_native_item(item);
  // }

  let app_state = Arc::new(Mutex::new(AppState::new(plugin_host)));

  tauri::Builder::default()
    .manage(app_state.clone())
    .on_window_event({
      let state = app_state.clone();
      move |window_event| on_window_event(&state, window_event)
    })
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
      play_command,
      pause_command,
      stop_command,
    ])
    // .menu(main_menu)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn on_window_event(state: &Arc<Mutex<AppState>>, window_event: GlobalWindowEvent) {
  match window_event.event() {
    WindowEvent::CloseRequested => {
      let mut state = state.lock().unwrap();
      let volume_publisher_service = state.volume_publisher_service();
      volume_publisher_service.stop();
    }
    WindowEvent::Resized(_) => {}
    WindowEvent::Moved(_) => {}
    WindowEvent::Destroyed => {}
    WindowEvent::Focused(_) => {}
    WindowEvent::ScaleFactorChanged { .. } => {}
    _ => {}
  }
}
