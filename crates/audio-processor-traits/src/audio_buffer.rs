/// Passing around `&mut [f32]` as audio buffers isn't good because:
///
/// * Some libraries / APIs will use interleaved buffers
/// * Some will not
/// * If you pick one all your processor code is bound to a buffer layout
/// * If there's an abstraction on top the processor code can work for any buffer layout while
///   still having the sample performance
/// * Currently `AudioProcessor` is made to work with cpal interleaved buffers; it then needs
///   conversion to work with VST.
/// * That's very unfortunate. I'd like to write a single processor that can work with both buffer
///   types with no overhead.
pub trait AudioBuffer {
    type SampleType: num::Float + Sync + Send;

    /// The number of channels in this buffer
    fn num_channels(&self) -> usize;

    /// The number of samples in this buffer
    fn num_samples(&self) -> usize;

    /// Get a ref to an INPUT sample in this buffer
    fn get(&self, channel: usize, sample: usize) -> &Self::SampleType;

    /// Get a mutable ref to an OUTPUT sample in this buffer
    ///
    /// On some implementations this may yield a different value than `.get`.
    fn get_mut(&mut self, channel: usize, sample: usize) -> &mut Self::SampleType;

    /// Set an OUTPUT sample in this buffer
    fn set(&mut self, channel: usize, sample: usize, value: Self::SampleType);

    /// Create a read only iterator
    fn iter(&self) -> AudioBufferIterator<Self> {
        AudioBufferIterator::new(&self)
    }
}

pub struct AudioBufferIterator<'a, BufferType: AudioBuffer + ?Sized> {
    position: usize,
    buffer: &'a BufferType,
}

impl<'a, BufferType: AudioBuffer + ?Sized> AudioBufferIterator<'a, BufferType> {
    pub fn new(buffer: &'a BufferType) -> Self {
        AudioBufferIterator {
            position: 0,
            buffer,
        }
    }
}

impl<'a, BufferType: AudioBuffer> Iterator for AudioBufferIterator<'a, BufferType> {
    type Item = AudioFrameReference<'a, BufferType>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.buffer.num_samples() {
            return None;
        }

        let reference = AudioFrameReference::new(self.buffer, self.position);
        self.position += 1;
        Some(reference)
    }
}

pub struct AudioFrameReference<'a, BufferType> {
    sample_index: usize,
    buffer: &'a BufferType,
}

impl<'a, BufferType> AudioFrameReference<'a, BufferType> {
    fn new(buffer: &'a BufferType, sample_index: usize) -> Self {
        AudioFrameReference {
            sample_index,
            buffer,
        }
    }

    pub fn iter(&self) -> AudioFrameReferenceIterator<'a, BufferType> {
        AudioFrameReferenceIterator::new(self.buffer, self.sample_index)
    }
}

pub struct AudioFrameReferenceIterator<'a, BufferType> {
    buffer: &'a BufferType,
    sample_index: usize,
    channel_index: usize,
}

impl<'a, BufferType> AudioFrameReferenceIterator<'a, BufferType> {
    fn new(buffer: &'a BufferType, sample_index: usize) -> Self {
        AudioFrameReferenceIterator {
            buffer,
            sample_index,
            channel_index: 0,
        }
    }
}

impl<'a, BufferType: AudioBuffer> Iterator for AudioFrameReferenceIterator<'a, BufferType> {
    type Item = &'a BufferType::SampleType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel_index >= self.buffer.num_channels() {
            None
        } else {
            let r = self.buffer.get(self.channel_index, self.sample_index);
            self.channel_index += 1;
            Some(r)
        }
    }
}

/// An AudioBuffer that stores samples as interleaved frames, used for CPAL.
///
/// Example layout:
///
/// [
///   0, 0, // <- left_sample, right_sample,
///   ...,
/// ]
pub struct InterleavedAudioBuffer<'a, SampleType> {
    num_channels: usize,
    inner: &'a mut [SampleType],
}

impl<'a, SampleType> InterleavedAudioBuffer<'a, SampleType> {
    pub fn new(num_channels: usize, inner: &'a mut [SampleType]) -> Self {
        Self {
            num_channels,
            inner,
        }
    }

    pub fn inner(&self) -> &[SampleType] {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut [SampleType] {
        &mut self.inner
    }
}

impl<'a, SampleType: num::Float + Sync + Send> AudioBuffer
    for InterleavedAudioBuffer<'a, SampleType>
{
    type SampleType = SampleType;

    fn num_channels(&self) -> usize {
        self.num_channels
    }

    fn num_samples(&self) -> usize {
        self.inner.len() / self.num_channels
    }

    fn get(&self, channel: usize, sample: usize) -> &SampleType {
        &self.inner[sample * self.num_channels + channel]
    }

    fn get_mut(&mut self, channel: usize, sample: usize) -> &mut SampleType {
        &mut self.inner[sample * self.num_channels + channel]
    }

    fn set(&mut self, channel: usize, sample: usize, value: SampleType) {
        let sample_ref = self.get_mut(channel, sample);
        *sample_ref = value;
    }
}

/// An AudioBuffer that stores samples as separate buffer slices. Similar the VST, but unused due to
/// an explicit wrapper on top of rust-vst also being exported.
///
/// Example:
/// `[left_channel_ptr, right_channel_ptr]`
///
/// `left_channel = [0, 1, 2, 3, 4]`
pub struct SliceAudioBuffer<'a, SampleType> {
    channels: &'a mut [&'a mut [SampleType]],
}

impl<'a, SampleType> SliceAudioBuffer<'a, SampleType> {
    pub fn new(channels: &'a mut [&'a mut [SampleType]]) -> Self {
        Self { channels }
    }
}

impl<'a, SampleType: num::Float + Sync + Send> AudioBuffer for SliceAudioBuffer<'a, SampleType> {
    type SampleType = SampleType;

    fn num_channels(&self) -> usize {
        self.channels.len()
    }

    fn num_samples(&self) -> usize {
        if self.channels.is_empty() {
            0
        } else {
            self.channels[0].len()
        }
    }

    fn get(&self, channel: usize, sample: usize) -> &Self::SampleType {
        &self.channels[channel][sample]
    }

    fn get_mut(&mut self, channel: usize, sample: usize) -> &mut Self::SampleType {
        &mut self.channels[channel][sample]
    }

    fn set(&mut self, channel: usize, sample: usize, value: Self::SampleType) {
        self.channels[channel][sample] = value;
    }
}

#[cfg(feature = "vst_support")]
pub mod vst {
    use super::*;

    /// Wraps a VST buffer with a generic AudioBuffer.
    ///
    /// ## NOTE:
    /// Due to Rust VST using different references for input & output buffers the API here is
    /// slightly dubious.
    ///
    /// `audio_buffer.get(channel, sample)` will return a sample from the INPUT buffer.
    /// Meanwhile `audio_buffer.get_mut(channel, sample)` will return a sample from the OUTPUT
    /// buffer.
    ///
    /// This means it might be that `audio_buffer.get(channel, sample)` is different to
    /// `audio_buffer.get_mut(channel, sample)`.
    pub struct VSTAudioBuffer<'a, SampleType: num::Float> {
        inputs: ::vst::buffer::Inputs<'a, SampleType>,
        outputs: ::vst::buffer::Outputs<'a, SampleType>,
    }

    impl<'a, SampleType: num::Float> VSTAudioBuffer<'a, SampleType> {
        pub fn new(
            inputs: ::vst::buffer::Inputs<'a, SampleType>,
            outputs: ::vst::buffer::Outputs<'a, SampleType>,
        ) -> Self {
            VSTAudioBuffer { inputs, outputs }
        }

        pub fn with_buffer(buffer: &'a mut ::vst::buffer::AudioBuffer<'a, SampleType>) -> Self {
            let (inputs, outputs) = buffer.split();
            Self::new(inputs, outputs)
        }
    }

    impl<'a, SampleType: num::Float + Sync + Send> AudioBuffer for VSTAudioBuffer<'a, SampleType> {
        type SampleType = SampleType;

        fn num_channels(&self) -> usize {
            self.outputs.len()
        }

        fn num_samples(&self) -> usize {
            if self.outputs.is_empty() {
                0
            } else {
                self.outputs.get(0).len()
            }
        }

        fn get(&self, channel: usize, sample: usize) -> &Self::SampleType {
            &self.inputs.get(channel)[sample]
        }

        fn get_mut(&mut self, channel: usize, sample: usize) -> &mut Self::SampleType {
            &mut self.outputs.get_mut(channel)[sample]
        }

        fn set(&mut self, channel: usize, sample: usize, value: Self::SampleType) {
            self.outputs.get_mut(channel)[sample] = value;
        }
    }
}
