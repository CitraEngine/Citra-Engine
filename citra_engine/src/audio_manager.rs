/*
Citra Engine - A Nintendo 3DS first game engine
Copyright (C) 2025  Citra Engine

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::time::Duration;

pub trait AudioManager {
    /* Cache:
        - Preload audio into a decode ready state
        - bgm_play and sfx_play will automatically cache if not already cached
        - It is recommended to cache bgm tracks during scene loading
    */
    fn cache(&self, path: String);
    fn uncache(&self, path: String);
    fn uncache_all(&self);

    /* BGM:
        - Only one audio file at a time
        - Always loops
        - fading
        - Reccommended format: Ogg Vorbis ( 2Chn 32bps 48kHz )
    */
    fn bgm_play(&self, fade_in_time: Duration, target_volume: u8, path: String);
    fn bgm_pause(&self, fade_out_time: Duration);
    fn bgm_resume(&self, fade_in_time: Duration);
    fn bgm_stop(&self, fade_out_time: Duration);
    fn bgm_set_volume(&self, volume: u8, fade_time: Duration);
    fn bgm_set_time(&self, time: Duration);
    fn bgm_playing(&self) -> bool;

    /* SFX:
        - Multiple audio files
        - No looping
        - Only set volume on play
        - Reccommended format: WAV or raw PCM (2Chn 32bps 48kHz)
    */
    // if sfx buffer is full, the oldest sound effect is stopped to make room
    fn se_play(&self, volume: u8, path: String);
    fn se_stop_all(&self);
    fn se_num_playing(&self) -> usize;

    /* LOADING SCREEN:
        - The 3DS cannot load assets and submit gpu commands to display the loading screen at the same time on the same thread
        - Therefore we use the audio thread to submit gpu commands while the main thread loads assets
        - On the 3DS this will stop all sound processing
            - For non 3DS platforms you can either stop sound for accuracy or add your own loading screen music
    */
    fn ls_start(&self);
    // ls_stop should not return until the audio thread is ready to recieve commands again
    fn ls_stop(&self);
}

pub struct PlaceholderAudioManager {}
impl AudioManager for PlaceholderAudioManager {
    fn cache(&self, _: String) {}
    fn uncache(&self, _: String) {}
    fn uncache_all(&self) {}

    fn bgm_play(&self, _: Duration, _: u8, _: String) {}
    fn bgm_pause(&self, _: Duration) {}
    fn bgm_resume(&self, _: Duration) {}
    fn bgm_stop(&self, _: Duration) {}
    fn bgm_set_volume(&self, _: u8, _: Duration) {}
    fn bgm_set_time(&self, _: Duration) {}
    fn bgm_playing(&self) -> bool {false}

    fn se_play(&self, _: u8, _: String) {}
    fn se_stop_all(&self) {}
    fn se_num_playing(&self) -> usize {0}

    fn ls_start(&self) {}
    fn ls_stop(&self) {}
}