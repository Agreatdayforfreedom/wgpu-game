use rodio::{cpal::FromSample, Decoder, Source};

pub struct Audio {
    #[allow(dead_code)]
    stream: rodio::OutputStream,
    #[allow(dead_code)]
    stream_handle: rodio::OutputStreamHandle,
    sink: rodio::Sink,
    tracks: rodio::Sink,
    sinks: Vec<rodio::Sink>,
    spatial_sink: rodio::SpatialSink,
}
const explosion_bytes: &[u8] = include_bytes!("./assets/explosion.wav");

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
        let music = include_bytes!("./assets/peaceful.mp3");
        let d = rodio::Decoder::new(std::io::Cursor::new(music).clone()).unwrap();

        Self {
            stream,
            stream_handle,
            sink,
            sinks: vec![],
            tracks,
            spatial_sink,
        }
    }

    pub fn start_track<S>(&self, bytes: S)
    where
        S: rodio::Source + Send + 'static,
        S::Item: rodio::Sample + Send,
        f32: FromSample<S::Item>,
    {
        self.tracks.append(bytes);
    }

    pub fn push(&mut self) {
        let sink = rodio::Sink::try_new(&self.stream_handle).unwrap();
        let d = rodio::Decoder::new(std::io::Cursor::new(explosion_bytes).clone()).unwrap();
        sink.set_volume(0.2);
        sink.append(d);
        self.sinks.push(sink);
    }

    //todo
    pub fn len(&self) {
        println!("len: {}", self.sinks.len());
    }

    pub fn queue<S>(&self, bytes: S)
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
}
