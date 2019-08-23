extern crate boxfnonce;
extern crate ipc_channel;
extern crate servo_media;
extern crate servo_media_audio;
extern crate servo_media_player;
extern crate servo_media_streams;
extern crate servo_media_traits;
extern crate servo_media_webrtc;

use boxfnonce::SendBoxFnOnce;
use ipc_channel::ipc::IpcSender;
use servo_media::{Backend, BackendInit, SupportsMediaType};
use servo_media_audio::block::Chunk;
use servo_media_audio::context::{AudioContext, AudioContextOptions};
use servo_media_audio::decoder::{AudioDecoder, AudioDecoderCallbacks, AudioDecoderOptions};
use servo_media_audio::render_thread::AudioRenderThreadMsg;
use servo_media_audio::sink::{AudioSink, AudioSinkError};
use servo_media_audio::AudioBackend;
use servo_media_player::context::PlayerGLContext;
use servo_media_player::{frame, Player, PlayerError, PlayerEvent, StreamType};
use servo_media_streams::capture::MediaTrackConstraintSet;
use servo_media_streams::registry::{register_stream, unregister_stream, MediaStreamId};
use servo_media_streams::{MediaOutput, MediaStream, MediaStreamType};
use servo_media_traits::{ClientContextId, MediaInstance};
use servo_media_webrtc::{
    thread, BundlePolicy, IceCandidate, SessionDescription, WebRtcBackend, WebRtcController,
    WebRtcControllerBackend, WebRtcSignaller, WebrtcResult,
};
use std::any::Any;
use std::ops::Range;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};

pub struct DummyBackend;

impl BackendInit for DummyBackend {
    fn init() -> Box<dyn Backend> {
        Box::new(DummyBackend)
    }
}

impl Backend for DummyBackend {
    fn create_audiostream(&self) -> MediaStreamId {
        register_stream(Arc::new(Mutex::new(DummyMediaStream {
            id: MediaStreamId::new(),
        })))
    }

    fn create_videostream(&self) -> MediaStreamId {
        register_stream(Arc::new(Mutex::new(DummyMediaStream {
            id: MediaStreamId::new(),
        })))
    }

    fn create_stream_output(&self) -> Box<dyn MediaOutput> {
        Box::new(DummyMediaOutput)
    }

    fn create_audioinput_stream(&self, _: MediaTrackConstraintSet) -> Option<MediaStreamId> {
        Some(register_stream(Arc::new(Mutex::new(DummyMediaStream {
            id: MediaStreamId::new(),
        }))))
    }

    fn create_videoinput_stream(&self, _: MediaTrackConstraintSet) -> Option<MediaStreamId> {
        Some(register_stream(Arc::new(Mutex::new(DummyMediaStream {
            id: MediaStreamId::new(),
        }))))
    }

    fn create_player(
        &self,
        _id: &ClientContextId,
        _: StreamType,
        _: IpcSender<PlayerEvent>,
        _: Option<Arc<Mutex<dyn frame::FrameRenderer>>>,
        _: Box<dyn PlayerGLContext>,
    ) -> Arc<Mutex<dyn Player>> {
        Arc::new(Mutex::new(DummyPlayer))
    }

    fn create_audio_context(
        &self,
        _id: &ClientContextId,
        options: AudioContextOptions,
    ) -> Arc<Mutex<AudioContext>> {
        let (sender, _) = mpsc::channel();
        let sender = Arc::new(Mutex::new(sender));
        Arc::new(Mutex::new(AudioContext::new::<Self>(
            0,
            &ClientContextId::build(1, 1),
            sender,
            options,
        )))
    }

    fn create_webrtc(&self, signaller: Box<dyn WebRtcSignaller>) -> WebRtcController {
        WebRtcController::new::<Self>(signaller)
    }

    fn can_play_type(&self, _media_type: &str) -> SupportsMediaType {
        SupportsMediaType::No
    }
}

impl AudioBackend for DummyBackend {
    type Sink = DummyAudioSink;
    fn make_decoder() -> Box<dyn AudioDecoder> {
        Box::new(DummyAudioDecoder)
    }

    fn make_sink() -> Result<Self::Sink, AudioSinkError> {
        Ok(DummyAudioSink)
    }
}

pub struct DummyPlayer;

impl Player for DummyPlayer {
    fn play(&self) -> Result<(), PlayerError> {
        Ok(())
    }
    fn pause(&self) -> Result<(), PlayerError> {
        Ok(())
    }
    fn stop(&self) -> Result<(), PlayerError> {
        Ok(())
    }
    fn seek(&self, _: f64) -> Result<(), PlayerError> {
        Ok(())
    }

    fn set_mute(&self, _: bool) -> Result<(), PlayerError> {
        Ok(())
    }

    fn set_volume(&self, _: f64) -> Result<(), PlayerError> {
        Ok(())
    }

    fn set_input_size(&self, _: u64) -> Result<(), PlayerError> {
        Ok(())
    }
    fn set_rate(&self, _: f64) -> Result<(), PlayerError> {
        Ok(())
    }
    fn push_data(&self, _: Vec<u8>) -> Result<(), PlayerError> {
        Ok(())
    }
    fn end_of_stream(&self) -> Result<(), PlayerError> {
        Ok(())
    }
    fn buffered(&self) -> Result<Vec<Range<f64>>, PlayerError> {
        Ok(vec![])
    }

    fn set_stream(&self, _: &MediaStreamId, _: bool) -> Result<(), PlayerError> {
        Ok(())
    }

    fn render_use_gl(&self) -> bool {
        false
    }
    fn set_audio_track_enabled(&self, _: i32,  _: bool) -> Result<(), PlayerError> { Ok(()) }
    fn set_video_track_enabled(&self, _: i32,  _: bool) -> Result<(), PlayerError> { Ok(()) }
}

impl WebRtcBackend for DummyBackend {
    type Controller = DummyWebRtcController;
    fn construct_webrtc_controller(
        _: Box<dyn WebRtcSignaller>,
        _: WebRtcController,
    ) -> Self::Controller {
        DummyWebRtcController
    }
}

pub struct DummyAudioDecoder;

impl AudioDecoder for DummyAudioDecoder {
    fn decode(&self, _: Vec<u8>, _: AudioDecoderCallbacks, _: Option<AudioDecoderOptions>) {}
}

pub struct DummyMediaStream {
    id: MediaStreamId,
}

impl MediaStream for DummyMediaStream {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn set_id(&mut self, _: MediaStreamId) {}

    fn ty(&self) -> MediaStreamType {
        MediaStreamType::Audio
    }
}

impl Drop for DummyMediaStream {
    fn drop(&mut self) {
        unregister_stream(&self.id);
    }
}

pub struct DummyAudioSink;

impl AudioSink for DummyAudioSink {
    fn init(&self, _: f32, _: Sender<AudioRenderThreadMsg>) -> Result<(), AudioSinkError> {
        Ok(())
    }
    fn play(&self) -> Result<(), AudioSinkError> {
        Ok(())
    }
    fn stop(&self) -> Result<(), AudioSinkError> {
        Ok(())
    }
    fn has_enough_data(&self) -> bool {
        true
    }
    fn push_data(&self, _: Chunk) -> Result<(), AudioSinkError> {
        Ok(())
    }
    fn set_eos_callback(&self, _: Box<dyn Fn(Box<dyn AsRef<[f32]>>) + Send + Sync + 'static>) {}
}

pub struct DummyMediaOutput;
impl MediaOutput for DummyMediaOutput {
    fn add_stream(&mut self, _stream: &MediaStreamId) {}
}

pub struct DummyWebRtcController;

impl WebRtcControllerBackend for DummyWebRtcController {
    fn configure(&mut self, _: &str, _: BundlePolicy) -> WebrtcResult {
        Ok(())
    }
    fn set_remote_description(
        &mut self,
        _: SessionDescription,
        _: SendBoxFnOnce<'static, ()>,
    ) -> WebrtcResult {
        Ok(())
    }
    fn set_local_description(
        &mut self,
        _: SessionDescription,
        _: SendBoxFnOnce<'static, ()>,
    ) -> WebrtcResult {
        Ok(())
    }
    fn add_ice_candidate(&mut self, _: IceCandidate) -> WebrtcResult {
        Ok(())
    }
    fn create_offer(&mut self, _: SendBoxFnOnce<'static, (SessionDescription,)>) -> WebrtcResult {
        Ok(())
    }
    fn create_answer(&mut self, _: SendBoxFnOnce<'static, (SessionDescription,)>) -> WebrtcResult {
        Ok(())
    }
    fn add_stream(&mut self, _: &MediaStreamId) -> WebrtcResult {
        Ok(())
    }
    fn internal_event(&mut self, _: thread::InternalEvent) -> WebrtcResult {
        Ok(())
    }
    fn quit(&mut self) {}
}

impl MediaInstance for DummyPlayer {
    fn get_id(&self) -> usize {
        0
    }

    fn mute(&self, _val: bool) -> Result<(), ()> {
        Ok(())
    }
}
