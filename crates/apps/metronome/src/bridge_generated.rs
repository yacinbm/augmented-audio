#![allow(
    non_camel_case_types,
    unused,
    clippy::redundant_closure,
    clippy::useless_conversion,
    clippy::unit_arg,
    clippy::double_parens,
    non_snake_case,
    clippy::too_many_arguments
)]
// AUTO GENERATED FILE, DO NOT EDIT.
// Generated by `flutter_rust_bridge`@ 1.78.0.

use crate::api::*;
use core::panic::UnwindSafe;
use flutter_rust_bridge::*;
use std::ffi::c_void;
use std::sync::Arc;

// Section: imports

// Section: wire functions

fn wire_initialize_impl(
    port_: MessagePort,
    options: impl Wire2Api<InitializeOptions> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "initialize",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_options = options.wire2api();
            move |task_callback| initialize(api_options)
        },
    )
}
fn wire_deinitialize_impl(port_: MessagePort) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "deinitialize",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || move |task_callback| deinitialize(),
    )
}
fn wire_set_is_playing_impl(port_: MessagePort, value: impl Wire2Api<bool> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "set_is_playing",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_value = value.wire2api();
            move |task_callback| set_is_playing(api_value)
        },
    )
}
fn wire_set_tempo_impl(port_: MessagePort, value: impl Wire2Api<f32> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "set_tempo",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_value = value.wire2api();
            move |task_callback| set_tempo(api_value)
        },
    )
}
fn wire_set_volume_impl(port_: MessagePort, value: impl Wire2Api<f32> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "set_volume",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_value = value.wire2api();
            move |task_callback| set_volume(api_value)
        },
    )
}
fn wire_set_beats_per_bar_impl(port_: MessagePort, value: impl Wire2Api<i32> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "set_beats_per_bar",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_value = value.wire2api();
            move |task_callback| set_beats_per_bar(api_value)
        },
    )
}
fn wire_set_sound_impl(
    port_: MessagePort,
    value: impl Wire2Api<MetronomeSoundTypeTag> + UnwindSafe,
) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "set_sound",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_value = value.wire2api();
            move |task_callback| set_sound(api_value)
        },
    )
}
fn wire_get_playhead_impl(port_: MessagePort) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "get_playhead",
            port: Some(port_),
            mode: FfiCallMode::Stream,
        },
        move || move |task_callback| get_playhead(task_callback.stream_sink()),
    )
}
// Section: wrapper structs

// Section: static checks

// Section: allocate functions

// Section: related functions

// Section: impl Wire2Api

pub trait Wire2Api<T> {
    fn wire2api(self) -> T;
}

impl<T, S> Wire2Api<Option<T>> for *mut S
where
    *mut S: Wire2Api<T>,
{
    fn wire2api(self) -> Option<T> {
        (!self.is_null()).then(|| self.wire2api())
    }
}

impl Wire2Api<bool> for bool {
    fn wire2api(self) -> bool {
        self
    }
}

impl Wire2Api<f32> for f32 {
    fn wire2api(self) -> f32 {
        self
    }
}
impl Wire2Api<i32> for i32 {
    fn wire2api(self) -> i32 {
        self
    }
}

impl Wire2Api<MetronomeSoundTypeTag> for i32 {
    fn wire2api(self) -> MetronomeSoundTypeTag {
        match self {
            0 => MetronomeSoundTypeTag::Sine,
            1 => MetronomeSoundTypeTag::Tube,
            2 => MetronomeSoundTypeTag::Glass,
            3 => MetronomeSoundTypeTag::Snap,
            _ => unreachable!("Invalid variant for MetronomeSoundTypeTag: {}", self),
        }
    }
}

impl Wire2Api<u8> for u8 {
    fn wire2api(self) -> u8 {
        self
    }
}

// Section: impl IntoDart

// Section: executor

support::lazy_static! {
    pub static ref FLUTTER_RUST_BRIDGE_HANDLER: support::DefaultHandler = Default::default();
}

#[cfg(not(target_family = "wasm"))]
#[path = "bridge_generated.io.rs"]
mod io;
#[cfg(not(target_family = "wasm"))]
pub use io::*;
