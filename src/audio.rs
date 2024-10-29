use std::io::Cursor;

use rodio::cpal::FromSample;

pub struct Audio {
    #[allow(dead_code)]
    stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    #[allow(dead_code)]
    sink: rodio::Sink,
    tracks: rodio::Sink,
    sinks: Vec<rodio::Sink>,
    #[allow(dead_code)]
    spatial_sink: rodio::SpatialSink,
}

pub enum Sounds {
    MainTheme,
    Explosion,
    Shoot,
}

impl Sounds {
    fn bytes(&self) -> Cursor<&'static [u8]> {
        match self {
            Self::Explosion => Cursor::new(include_bytes!("./assets/explosion.wav") as &[u8]),
            Self::Shoot => Cursor::new(include_bytes!("./assets/shoot.wav") as &[u8]),
            Self::MainTheme => Cursor::new(include_bytes!("./assets/main_theme.mp3") as &[u8]),
        }
    }

    fn volume(&self) -> f32 {
        match self {
            Self::Explosion => 0.2,
            Self::Shoot => 0.05,
            Self::MainTheme => 1.0,
        }
    }
}

impl Audio {
    pub fn new() -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        let tracks = rodio::Sink::try_new(&stream_handle).unwrap();
        tracks.set_volume(1.0);
        sink.set_volume(0.05);

        let spatial_sink = rodio::SpatialSink::try_new(
            &stream_handle,
            [0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        )
        .unwrap();

        Self {
            stream,
            stream_handle,
            sink,
            sinks: vec![],
            tracks,
            spatial_sink,
        }
    }

    pub fn start_track(&self, sound: Sounds) {
        let source = rodio::Decoder::new(sound.bytes().clone()).unwrap();
        self.tracks.set_volume(sound.volume());
        self.tracks.append(source);
    }

    pub fn push(&mut self, sound: Sounds, volume: f32) {
        let sink = rodio::Sink::try_new(&self.stream_handle).unwrap();
        let source = rodio::Decoder::new(sound.bytes().clone()).unwrap();

        let volume = sound.volume() * volume;

        sink.set_volume(volume);
        sink.append(source);
        self.sinks.push(sink);
    }

    pub fn _queue<S>(&self, bytes: S)
    where
        S: rodio::Source + Send + 'static,
        S::Item: rodio::Sample + Send,
        f32: FromSample<S::Item>,
    {
        self.sink.clear();
        self.sink.append(bytes);
        if self.sink.is_paused() && !self.sink.empty() {
            self.sink.play();
        }
    }

    pub fn update(&mut self) {
        self.sinks.retain(|s| !s.empty());
    }
}
