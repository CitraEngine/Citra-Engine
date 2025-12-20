use std::{
    cell::{RefCell, RefMut},
    fmt::Display,
    fs::File,
    io::{BufReader, SeekFrom},
    mem::MaybeUninit,
    os::raw::c_void,
    pin::Pin,
    rc::Rc,
    sync::atomic::Ordering,
    time::Duration,
};

use citra_engine::{audio_manager::AudioManager, error::CitraError};
use ctru::{
    Error,
    linear::LinearAllocator,
    services::ndsp::{self, AudioFormat, Ndsp, OutputMode},
};
use ctru_sys::{
    DSP_FlushDataCache, LightEvent, LightEvent_Clear, LightEvent_Init, LightEvent_Signal,
    LightEvent_Wait, LightLock, LightLock_Init, LightLock_Lock, LightLock_Unlock, NDSP_CLIP_NORMAL,
    NDSP_INTERP_LINEAR, NDSP_WBUF_DONE, NDSP_WBUF_FREE, R_MODULE, R_SUMMARY, RESET_ONESHOT, RM_DSP,
    RS_NOTFOUND, ndspChnSetFormat, ndspChnSetInterp, ndspChnSetMix, ndspChnSetRate,
    ndspChnWaveBufAdd, ndspSetCallback, ndspSetClippingMode, ndspSetOutputCount, ndspWaveBuf,
    svcGetProcessorID, threadCreate, threadExit, threadJoin,
};
use symphonia::{
    core::{
        audio::{AudioBuffer, AudioBufferRef, RawSampleBuffer, SampleBuffer, Signal}, codecs::{CodecRegistry, Decoder, DecoderOptions}, conv::IntoSample, formats::{FormatOptions, FormatReader}, io::{MediaSourceStream, MediaSourceStreamOptions}, meta::MetadataOptions, probe::{Hint, Probe}, sample::SampleFormat
    },
    default::{get_codecs, get_probe},
};

use crate::panicking::{do_panic, panicking};

fn get_sample_size(format: &SampleFormat) -> i32 {
    match *format {
        SampleFormat::S8 | SampleFormat::U8 => 1,
        SampleFormat::S16 | SampleFormat::U16 => 2,
        SampleFormat::S24 | SampleFormat::U24 => 3,
        SampleFormat::F32 | SampleFormat::S32 | SampleFormat::U32 => 4,
        SampleFormat::F64 => 8,
    }
}
fn to_ndsp_format(format: &SampleFormat, channels: i32) -> Result<ndsp::AudioFormat, CitraError> {
    if !(1..=2).contains(&channels) {
        return Err(CitraError::PlatformError(format!(
            "Invalid number of channels: '{}'",
            channels
        )));
    }
    match (*format, channels) {
        (SampleFormat::S8, 1) => Ok(ndsp::AudioFormat::PCM8Mono),
        (SampleFormat::S8, 2) => Ok(ndsp::AudioFormat::PCM8Stereo),
        (SampleFormat::S16, 1) => Ok(ndsp::AudioFormat::PCM8Mono),
        (SampleFormat::S16, 2) => Ok(ndsp::AudioFormat::PCM16Stereo),
        _ => Err(CitraError::PlatformError(format!(
            "Unsupported Audio Format: {:?}",
            format
        ))),
    }
}
fn from_ndsp_format(format: &AudioFormat) -> SampleFormat {
    match *format {
        AudioFormat::PCM8Mono | AudioFormat::PCM8Stereo => SampleFormat::S8,
        AudioFormat::PCM16Mono | AudioFormat::PCM16Stereo => SampleFormat::S16,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AudioFomatting {
    pub frequency: f32,
    pub format: SampleFormat,
    pub channels: i32,
}

fn decode(
    reader: &mut Box<dyn FormatReader>,
    decoder: &mut Box<dyn Decoder>,
    buffer: *mut u8,
    size: i32,
    recursion_depth: usize,
) -> Result<i32, CitraError> {
    let res: AudioBuffer<i16> = decoder.decode(&reader.next_packet().map_err(|e| CitraError::PlatformError(e.to_string()))?).map_err(|e| CitraError::PlatformError(e.to_string()))?.make_equivalent();
    res.fill(|a, b| -> Result<(), symphonia::core::errors::Error> {
        Ok(())
    });

    if res < 0 {
        unsafe { buffer.write_bytes(0, size as usize) };
    } else if res < size {
        unsafe {
            buffer
                .offset(res as isize)
                .write_bytes(0, (size - res) as usize)
        };
    }

    if decode_info.finished && decode_info.do_loop && recursion_depth < 10 {
        decode_info.loop_count += 1;
        rewind_decoder(&mut decoder)?;
        if size - res > 0 {
            let res2 = decode(decoder, buffer, size, recursion_depth + 1)?;
            if res2 <= 0 {
                return Ok(res);
            }
            return Ok(res + res2);
        }
    }

    if recursion_depth >= 10 {
        return Err(CitraError::PlatformError(
            "Decode: Max recursion depth exceeded!".to_owned(),
        ));
    }

    Ok(res)
}
fn rewind_decoder(decoder: &mut Box<dyn Decoder>) -> Result<(), CitraError> {
    decoder.
}

pub struct DecodeInfo {
    pub finished: bool,
    pub do_loop: bool,
    pub loop_count: isize,
}
impl Default for DecodeInfo {
    fn default() -> Self {
        DecodeInfo {
            finished: false,
            do_loop: false,
            loop_count: 0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum AudioCommand {
    Terminate,
}

const CHANNELS: usize = 23;

struct SfxMan {
    pub buf: [ndspWaveBuf; CHANNELS],
}

const BUF_SIZE: u32 = 4 * 1024;
const BGM_CHANNEL: i32 = 23;
const NUM_BUFS: usize = 3;

const MEDIA_STREAM_OPTIONS: MediaSourceStreamOptions = MediaSourceStreamOptions {
    buffer_len: BUF_SIZE as usize * NUM_BUFS,
};

struct Bgm {
    pub buf: [ndspWaveBuf; NUM_BUFS],
    pub audio_buffer: Vec<i16, LinearAllocator>,
    pub reader: Option<Box<dyn FormatReader>>,
    pub decoder: Option<Box<dyn Decoder>>,
    pub volume: f32,
}
impl Bgm {
    pub fn set_channel_format(
        &mut self,
        formatting: &AudioFomatting,
    ) -> Result<SampleFormat, CitraError> {
        let ndsp_format = to_ndsp_format(&formatting.format, formatting.channels)?;
        unsafe {
            ndspChnSetFormat(BGM_CHANNEL, ndsp_format as u8 as u16);
        };
        Ok(from_ndsp_format(&ndsp_format))
    }
}

struct MusicUserData {
    pub audio_mutex: LightLock,
    pub audio_event: LightEvent,
    pub inner: MusicUserDataInner,
}
impl MusicUserData {
    pub fn lock<F, R>(&mut self, func: F) -> R
    where
        F: FnOnce(&mut MusicUserDataInner) -> R,
    {
        unsafe { LightLock_Lock(&mut self.audio_mutex) };
        let result = func(&mut self.inner);
        unsafe { LightLock_Unlock(&mut self.audio_mutex) };
        result
    }
}
impl Drop for MusicUserData {
    fn drop(&mut self) {
        unsafe {
            LightEvent_Clear(&mut self.audio_event);
        }
    }
}

struct MusicUserDataInner {
    pub fill_block: usize,
    pub bgm: Bgm,
    pub sfx_man: SfxMan,
    pub audio_bound_command_queue: Vec<AudioCommand>,
    pub master_volume: f32,
}

pub struct N3dsAudioManager {
    ndsp: Ndsp,
    user_data: Pin<Box<MusicUserData>>,
    audio_thread: ctru_sys::Thread,
    codec_registry: &'static CodecRegistry,
    probe: &'static Probe,
}
impl N3dsAudioManager {
    pub fn new() -> Result<Self, CitraError> {
        let ndsp_res = Ndsp::new();
        if let Err(Error::Os(os_err)) = ndsp_res
            && R_SUMMARY(os_err) == RS_NOTFOUND
            && R_MODULE(os_err) == RM_DSP
        {
            return Err(CitraError::PlatformError(
                "Please dump your DSP firmware, guides are on https://3ds.hacks.guide".to_owned(),
            ));
        }
        let mut ndsp = ndsp_res.map_err(|e| CitraError::PlatformError(e.to_string()))?;
        unsafe { ndspSetOutputCount(1) };
        ndsp.set_output_mode(OutputMode::Stereo);
        unsafe { ndspSetClippingMode(NDSP_CLIP_NORMAL) };

        for i in 0..CHANNELS {
            unsafe {
                ndspChnSetInterp(i as i32, NDSP_INTERP_LINEAR);
            }
        }

        let mut audio_mutex = MaybeUninit::uninit();
        unsafe {
            LightLock_Init(audio_mutex.as_mut_ptr());
        }
        let audio_mutex = unsafe { audio_mutex.assume_init() };
        let mut audio_event = MaybeUninit::uninit();
        unsafe {
            LightEvent_Init(audio_event.as_mut_ptr(), RESET_ONESHOT);
        }
        let audio_event = unsafe { audio_event.assume_init() };

        let audio_buffer = Vec::with_capacity_in(BUF_SIZE as usize * NUM_BUFS, alloc)

        let mut user_data = Box::pin(MusicUserData {
            audio_mutex,
            audio_event,
            inner: MusicUserDataInner {
                fill_block: 0,
                bgm: Bgm {
                    buf: [ndspWaveBuf::default(); NUM_BUFS],
                    audio_buffer,
                    reader: None,
                    decoder: None,
                    volume: 1.0,
                },
                sfx_man: SfxMan {
                    buf: [ndspWaveBuf::default(); CHANNELS],
                },
                audio_bound_command_queue: Vec::with_capacity(32),
                master_volume: 1.0,
            },
        });

        // not initalizing is oki here because the DSP does not require the audio buffer to be zeroed when no sound is desired
        unsafe {
            user_data
                .inner
                .bgm
                .audio_buffer
                .set_len(BUF_SIZE as usize * NUM_BUFS);
        }
        for i in 0..NUM_BUFS {
            unsafe {
                user_data.inner.bgm.buf[i].__bindgen_anon_1.data_vaddr = user_data
                    .inner
                    .bgm
                    .audio_buffer
                    .as_ptr()
                    .offset((i * BUF_SIZE as usize).try_into().unwrap())
                    .cast()
            }
            user_data.inner.bgm.buf[i].nsamples = BUF_SIZE / 4u32;
            user_data.inner.bgm.buf[i].status = NDSP_WBUF_DONE;
        }

        unsafe {
            ndspSetCallback(
                Some(n3ds_dsp_callback),
                (user_data.as_mut().get_mut() as *mut MusicUserData).cast(),
            );
        }

        let audio_thread = unsafe {
            threadCreate(
                Some(n3ds_audio_thread),
                (user_data.as_mut().get_mut() as *mut MusicUserData).cast(),
                32768,
                0x18,
                1,
                true,
            )
        };

        let codec_registry = get_codecs();
        let probe = get_probe();

        Ok(Self {
            ndsp,
            user_data,
            audio_thread,
            codec_registry,
            probe,
        })
    }
}
impl AudioManager for N3dsAudioManager {
    fn cache(&mut self, path: String) {
        todo!()
    }
    fn uncache(&mut self, path: String) {
        todo!()
    }
    fn uncache_all(&mut self) {
        todo!()
    }

    fn bgm_play(
        &mut self,
        _fade_in_time: Duration,
        target_volume: f32,
        path: String,
    ) -> Result<(), CitraError> {
        let media_source_stream = MediaSourceStream::new(
            Box::new(File::open(path).map_err(|e| CitraError::PlatformError(e.to_string()))?),
            MEDIA_STREAM_OPTIONS,
        );
        let demuxer = self
            .probe
            .format(
                &Hint::default(),
                media_source_stream,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| CitraError::PlatformError(e.to_string()))?;
        let track = demuxer.format.as_ref().default_track().unwrap();
        let formatting = AudioFomatting {
            frequency: track.codec_params.sample_rate.unwrap() as f32,
            format: track.codec_params.sample_format.unwrap(),
            channels: track.codec_params.channels.unwrap().count() as i32,
        };
        let decoder = self
            .codec_registry
            .make(&track.codec_params, &DecoderOptions { verify: false })
            .map_err(|e| CitraError::PlatformError(e.to_string()))?;

        self.user_data
            .lock(move |music_data| -> Result<(), CitraError> {
                music_data.bgm.reader = Some(demuxer.format);
                music_data.bgm.decoder = Some(decoder);

                let out_format = music_data.bgm.set_channel_format(&formatting)?;
                unsafe { ndspChnSetRate(BGM_CHANNEL, formatting.frequency) };
                let sample_size = get_sample_size(&out_format);
                let n_samples = BUF_SIZE / (sample_size * formatting.channels) as u32;

                for buf in &mut music_data.bgm.buf {
                    buf.nsamples = n_samples;
                }

                music_data.bgm.volume = target_volume;
                Ok(())
            })?;
        Ok(())
    }
    fn bgm_pause(&mut self, fade_out_time: Duration) {
        todo!()
    }
    fn bgm_resume(&mut self, fade_in_time: Duration) {
        todo!()
    }
    fn bgm_stop(&mut self, _fade_out_time: Duration) {
        self.user_data.lock(|music_data| {
            music_data.bgm.reader = None;
            music_data.bgm.decoder = None;
        });
    }
    fn bgm_set_volume(&mut self, volume: f32, fade_time: Duration) {
        self.user_data.lock(|music_data| {
            music_data.bgm.volume = volume;
        });
    }
    fn bgm_set_time(&mut self, time: Duration) {
        todo!()
    }
    fn bgm_playing(&self) -> bool {
        todo!()
    }

    fn se_play(&mut self, volume: u8, path: String) {
        todo!()
    }
    fn se_stop_all(&mut self) {
        todo!()
    }
    fn se_num_playing(&self) -> usize {
        todo!()
    }

    fn ls_start(&mut self) {
        todo!()
    }
    fn ls_stop(&mut self) {
        todo!()
    }
}
impl Drop for N3dsAudioManager {
    fn drop(&mut self) {
        self.user_data
            .inner
            .audio_bound_command_queue
            .push(AudioCommand::Terminate);
        unsafe { LightEvent_Signal(&mut self.user_data.audio_event as *mut _) };
        let _ = unsafe { threadJoin(self.audio_thread, 1000000000) };
        let is_panicing = panicking.load(Ordering::Relaxed);
        if is_panicing {
            do_panic(true);
        }
    }
}

unsafe extern "C" fn n3ds_dsp_callback(userdata: *mut c_void) {
    let user_data: &mut MusicUserData = unsafe { &mut *userdata.cast() };

    let mut signal_audio_thread = false;
    user_data.lock(|music_data| {
        if music_data.bgm.buf[music_data.fill_block].status == NDSP_WBUF_DONE {
            music_data.bgm.buf[music_data.fill_block].status = NDSP_WBUF_FREE;
            signal_audio_thread = true;
        }
    });
    if signal_audio_thread {
        unsafe { LightEvent_Signal(&mut user_data.audio_event) };
    }
}

unsafe extern "C" fn n3ds_audio_thread(userdata: *mut c_void) {
    if unsafe { svcGetProcessorID() } != 1 {
        panic!("Audio thread is on the wrong core")
    }

    let user_data: &mut MusicUserData = unsafe { &mut *userdata.cast() };

    let mut should_exit = false;
    let mut mix = [0.0; 12];

    while !should_exit {
        unsafe { LightEvent_Wait(&mut user_data.audio_event) };
        user_data.lock(|music_data| {
            while let Some(command) = music_data.audio_bound_command_queue.pop() {
                if command == AudioCommand::Terminate {
                    should_exit = true;
                    break;
                }
            }

            let target_block = music_data.fill_block;
            music_data.fill_block += 1;
            music_data.fill_block %= NUM_BUFS;

            if let Some(decoder) = music_data.bgm.decoder.as_mut() && let Some(reader) = music_data.bgm.reader.as_mut() {
                unsafe {
                    decode(
                        reader,
                        decoder,
                        music_data.bgm.buf[target_block]
                            .__bindgen_anon_1
                            .data_pcm16
                            .cast(),
                        BUF_SIZE as i32,
                        0,
                    )
                    .unwrap();
                    let _ = DSP_FlushDataCache(
                        music_data.bgm.buf[target_block]
                            .__bindgen_anon_1
                            .data_pcm16
                            .cast(),
                        BUF_SIZE,
                    );
                    mix[0] = music_data.master_volume;
                    mix[1] = music_data.master_volume;
                    ndspChnSetMix(BGM_CHANNEL, mix.as_mut_ptr());
                    ndspChnWaveBufAdd(BGM_CHANNEL, &mut music_data.bgm.buf[target_block]);
                }
            } else {
                music_data.bgm.buf[target_block].status = NDSP_WBUF_DONE;
            }
        });
        if panicking.load(Ordering::Relaxed) {
            break;
        }
    }

    unsafe { threadExit(0) };
}
