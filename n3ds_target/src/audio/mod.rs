pub struct N3dsAudioManager {}
impl N3dsAudioManager {
    pub fn new() -> Result<Self, CitraError> {
        Ok(N3dsAudioManager {})
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
        todo!()
    }
    fn bgm_pause(&mut self, fade_out_time: Duration) {
        todo!()
    }
    fn bgm_resume(&mut self, fade_in_time: Duration) {
        todo!()
    }
    fn bgm_stop(&mut self, _fade_out_time: Duration) {
        todo!()
    }
    fn bgm_set_volume(&mut self, volume: f32, fade_time: Duration) {
        todo!()
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
