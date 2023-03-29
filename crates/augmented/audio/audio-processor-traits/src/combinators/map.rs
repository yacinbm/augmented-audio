// Augmented Audio: Audio libraries and applications
// Copyright (c) 2022 Pedro Tacla Yamada
//
// The MIT License (MIT)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use crate::{AudioBuffer, AudioContext, AudioProcessor};

struct MapProcessor<P, F> {
    processor: P,
    f: F,
}

impl<P, F> AudioProcessor for MapProcessor<P, F>
where
    P: AudioProcessor,
    F: FnMut(&mut AudioContext, &mut [P::SampleType]),
{
    type SampleType = P::SampleType;

    fn process<BufferType: AudioBuffer<SampleType = Self::SampleType>>(
        &mut self,
        context: &mut AudioContext,
        output: &mut BufferType,
    ) {
        self.processor.process(context, output);
        for frame in output.frames_mut() {
            (self.f)(context, frame);
        }
    }
}

pub fn map_processor<P: AudioProcessor, F: FnMut(&mut AudioContext, &mut [P::SampleType])>(
    processor: P,
    f: F,
) -> impl AudioProcessor<SampleType = P::SampleType> {
    MapProcessor { processor, f }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::combinators::function::mono_generator_function;
    use crate::{BufferProcessor, VecAudioBuffer};

    #[test]
    fn test_map_processor() {
        let input_p = mono_generator_function(|_| 1.0);
        let mut output = map_processor(BufferProcessor(input_p), |_, frame| {
            frame[0] = 2.0;
        });
        let mut ctx = AudioContext::default();
        let mut output_buffer = VecAudioBuffer::new_with(vec![0.0; 6], 2, 3);
        output.process(&mut ctx, &mut output_buffer);
        assert_eq!(
            output_buffer.slice(),
            [
                2.0, 0.5, // 1
                2.0, 0.5, // 2
                2.0, 0.5, // 3
            ]
        );
    }
}