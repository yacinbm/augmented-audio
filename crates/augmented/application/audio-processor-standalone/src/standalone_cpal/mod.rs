use basedrop::Handle;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, Host, SampleRate, StreamConfig,
};
use ringbuf::{Consumer, Producer};

use audio_processor_traits::{
    AudioProcessor, AudioProcessorSettings, InterleavedAudioBuffer, MidiEventHandler,
};
use midi::{flush_midi_events, initialize_midi_host, MidiContext, MidiHost};

use crate::standalone_processor::{
    StandaloneAudioOnlyProcessor, StandaloneProcessor, StandaloneProcessorImpl,
};

mod midi;

/// Start an [`AudioProcessor`] / [`MidiEventHandler`] as a stand-alone cpal app and forward MIDI
/// messages received on all inputs to it.
///
/// Returns the [`cpal::Stream`]s and [`MidiHost`]. The audio-thread will keep running until these are
/// dropped.
pub fn audio_processor_start_with_midi<
    Processor: AudioProcessor<SampleType = f32> + MidiEventHandler + Send + 'static,
>(
    audio_processor: Processor,
    handle: &Handle,
) -> StandaloneHandles {
    let app = StandaloneProcessorImpl::new(audio_processor);
    standalone_start(app, Some(handle))
}

/// Start an [`AudioProcessor`] as a stand-alone cpal app>
///
/// Returns the [`cpal::Stream`] streams. The audio-thread will keep running until these are dropped.
pub fn audio_processor_start<Processor: AudioProcessor<SampleType = f32> + Send + 'static>(
    audio_processor: Processor,
) -> StandaloneHandles {
    let app = StandaloneAudioOnlyProcessor::new(audio_processor);
    standalone_start(app, None)
}

/// Handles to the CPAL streams and MIDI host. Playback will stop when these are dropped.
pub struct StandaloneHandles {
    pub input_stream: Option<cpal::Stream>,
    pub output_stream: cpal::Stream,
    pub midi_host: Option<MidiHost>,
}

/// Start a processor using CPAL.
pub fn standalone_start(
    mut app: impl StandaloneProcessor,
    handle: Option<&Handle>,
) -> StandaloneHandles {
    let _ = wisual_logger::try_init_from_env();

    let (midi_host, mut midi_context) = initialize_midi_host(&mut app, handle);

    // Audio set-up
    let host = cpal::default_host();
    log::info!("Using host: {}", host.id().name());
    let buffer_size = 512;
    let sample_rate = 44100;
    let accepts_input = app.options().accepts_input;
    let input_tuple = if accepts_input {
        Some(configure_input_device(&host, buffer_size, sample_rate))
    } else {
        None
    };
    let (output_device, output_config) = configure_output_device(host, buffer_size, sample_rate);

    let settings = AudioProcessorSettings::new(
        output_config.sample_rate.0 as f32,
        input_tuple
            .as_ref()
            .map(|(_, input_config)| input_config.channels.into())
            .unwrap_or(0),
        output_config.channels.into(),
        buffer_size,
    );
    app.processor().prepare(settings);

    let buffer = ringbuf::RingBuffer::new((buffer_size * 10) as usize);
    let (mut producer, mut consumer) = buffer.split();
    let input_stream = input_tuple.map(|(input_device, input_config)| {
        input_device
            .build_input_stream(
                &input_config,
                move |data: &[f32], _input_info: &cpal::InputCallbackInfo| {
                    input_stream_callback(&mut producer, data)
                },
                |err| {
                    log::error!("Input error: {:?}", err);
                },
            )
            .unwrap()
    });

    // Output callback section
    let num_channels = output_config.channels.into();
    let output_stream = output_device
        .build_output_stream(
            &output_config,
            move |data: &mut [f32], _output_info: &cpal::OutputCallbackInfo| {
                output_stream_with_context(
                    midi_context.as_mut(),
                    &mut app,
                    num_channels,
                    &mut consumer,
                    data,
                );
            },
            |err| {
                log::error!("Playback error: {:?}", err);
            },
        )
        .unwrap();

    output_stream.play().unwrap();
    if let Some(input_stream) = &input_stream {
        input_stream.play().unwrap();
    }
    log::info!("Audio streams started");

    StandaloneHandles {
        input_stream,
        output_stream,
        midi_host,
    }
}

fn configure_input_device(
    host: &Host,
    buffer_size: usize,
    sample_rate: usize,
) -> (cpal::Device, StreamConfig) {
    let input_device = host.default_input_device().unwrap();
    log::info!("Using input: {}", input_device.name().unwrap());
    let supported_configs = input_device.supported_input_configs().unwrap();

    let mut supports_stereo = false;
    for config in supported_configs {
        log::info!("  INPUT Supported config: {:?}", config);
        if config.channels() > 1 {
            supports_stereo = true;
        }
    }

    let input_config = input_device.default_input_config().unwrap();
    let mut input_config: StreamConfig = input_config.into();
    input_config.channels = if supports_stereo { 2 } else { 1 };
    input_config.sample_rate = SampleRate(sample_rate as u32);
    input_config.buffer_size = BufferSize::Fixed(buffer_size as u32);

    #[cfg(target_os = "ios")]
    {
        input_config.buffer_size = BufferSize::Default;
    }

    (input_device, input_config)
}

fn configure_output_device(
    host: Host,
    buffer_size: usize,
    sample_rate: usize,
) -> (cpal::Device, StreamConfig) {
    let output_device = host.default_output_device().unwrap();
    log::info!("Using output: {}", output_device.name().unwrap());
    let supported_configs = output_device.supported_input_configs().unwrap();
    for config in supported_configs {
        log::info!("  OUTPUT Supported config: {:?}", config);
    }
    let output_config = output_device.default_output_config().unwrap();
    let mut output_config: StreamConfig = output_config.into();
    output_config.channels = 2;
    output_config.sample_rate = SampleRate(sample_rate as u32);
    output_config.buffer_size = BufferSize::Fixed(buffer_size as u32);
    #[cfg(target_os = "ios")]
    {
        output_config.buffer_size = BufferSize::Default;
    }
    (output_device, output_config)
}

fn input_stream_callback(producer: &mut Producer<f32>, data: &[f32]) {
    for sample in data {
        while producer.push(*sample).is_err() {}
    }
}

fn output_stream_with_context<Processor: StandaloneProcessor>(
    midi_context: Option<&mut MidiContext>,
    processor: &mut Processor,
    num_channels: usize,
    consumer: &mut Consumer<f32>,
    data: &mut [f32],
) {
    for sample in data.iter_mut() {
        if let Some(input_sample) = consumer.pop() {
            *sample = input_sample;
        } else {
            break;
        }
    }

    // Collect MIDI
    flush_midi_events(midi_context, processor);

    let mut audio_buffer = InterleavedAudioBuffer::new(num_channels, data);
    processor.processor().process(&mut audio_buffer);
}