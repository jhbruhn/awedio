use crate::{NextSample, Sound};

type SoundGenerator = Box<dyn FnMut() -> Option<Box<dyn Sound>> + Send>;

/// Call a generator function that produces a [`Sound`]. Then call the generator
/// again each time the previously returned `Sound` has completed, continuing
/// for as long as the generator function does not return `None` (possibly
/// forever).
///
/// This can be used to create sounds that loop forever without storing all
/// samples in memory.
pub struct SoundsFromFn {
    generator: SoundGenerator,
    current: Option<Box<dyn Sound>>,
    current_channel_count: u16,
    current_sample_rate: u32,
}

impl SoundsFromFn {
    /// Call `generator` to generate Sounds that will be played to completion.
    /// If `generator` returns None, this Sound will be Finished and `generator`
    /// will no longer be called.
    ///
    /// ## Examples
    /// Play an audio file forever.
    ///
    /// ```rust
    /// # fn no_run() {
    /// use awedio::sounds::{SoundsFromFn, open_file};
    ///
    /// let generator = || Some(open_file("test.wav").unwrap());
    /// let forever_sound = SoundsFromFn::new(Box::new(generator));
    /// # }
    /// ```
    pub fn new(mut generator: SoundGenerator) -> Self {
        let current = generator();
        let mut to_return = Self {
            generator,
            current,
            current_channel_count: 0,
            current_sample_rate: 0,
        };
        to_return.update_metadata();
        to_return
    }

    fn update_metadata(&mut self) {
        self.current_channel_count = self.channel_count();
        self.current_sample_rate = self.sample_rate();
    }
}

impl Sound for SoundsFromFn {
    fn channel_count(&self) -> u16 {
        self.current
            .as_ref()
            .map(|s| s.channel_count())
            .unwrap_or(1)
    }

    fn sample_rate(&self) -> u32 {
        self.current
            .as_ref()
            .map(|s| s.sample_rate())
            .unwrap_or(1000)
    }

    fn on_start_of_batch(&mut self) {
        if let Some(current) = &mut self.current {
            current.on_start_of_batch();
        }
    }

    fn next_sample(&mut self) -> NextSample {
        loop {
            let Some(current) = &mut self.current else {
                return NextSample::Finished;
            };
            let sample = current.next_sample();
            match sample {
                NextSample::MetadataChanged => {
                    self.update_metadata();
                    return sample;
                }
                NextSample::Sample(_) | NextSample::Paused => return sample,
                NextSample::Finished => {
                    let old_channel_count = self.current_channel_count;
                    let old_sample_rate = self.current_sample_rate;
                    self.current = (self.generator)();
                    self.update_metadata();
                    if self.current.is_none() {
                        return NextSample::Finished;
                    }
                    if old_sample_rate != self.sample_rate()
                        || old_channel_count != self.channel_count()
                    {
                        return NextSample::MetadataChanged;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "./tests/sounds_from_fn.rs"]
mod tests;
