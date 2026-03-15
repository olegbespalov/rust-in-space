use crate::components::AudioSettings;
use crate::resources::AudioAssets;
use macroquad::audio::{
    play_sound, play_sound_once, set_sound_volume, stop_sound, PlaySoundParams,
};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioCue {
    UiMove,
    UiConfirm,
    PlayerShot,
    EnemyShot,
    ExplosionSmall,
    ExplosionBig,
    PickupScrap,
    PickupHealth,
    ShieldOn,
    ShieldHit,
    ShipHit,
    MissionSuccess,
    GameOver,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MusicTrack {
    Menu,
    Gameplay,
}

pub struct AudioState {
    settings: AudioSettings,
    current_music: Option<MusicTrack>,
    engine_loop_active: bool,
    engine_loop_playing: bool,
    engine_loop_intensity: f32,
    paused: bool,
}

impl AudioState {
    pub fn new(settings: AudioSettings) -> Self {
        Self {
            settings,
            current_music: None,
            engine_loop_active: false,
            engine_loop_playing: false,
            engine_loop_intensity: 0.0,
            paused: false,
        }
    }

    pub fn play_sfx(&self, assets: &AudioAssets, cue: AudioCue) {
        if self.settings.audio_muted {
            return;
        }

        let sound = match cue {
            AudioCue::UiMove => &assets.ui_move,
            AudioCue::UiConfirm => &assets.ui_confirm,
            AudioCue::PlayerShot => &assets.player_shot,
            AudioCue::EnemyShot => &assets.enemy_shot,
            AudioCue::ExplosionSmall => &assets.explosion_small,
            AudioCue::ExplosionBig => &assets.explosion_big,
            AudioCue::PickupScrap => &assets.pickup_scrap,
            AudioCue::PickupHealth => &assets.pickup_health,
            AudioCue::ShieldOn => &assets.shield_on,
            AudioCue::ShieldHit => &assets.shield_hit,
            AudioCue::ShipHit => &assets.ship_hit,
            AudioCue::MissionSuccess => &assets.mission_success,
            AudioCue::GameOver => &assets.game_over,
        };

        if let Some(sound) = sound {
            set_sound_volume(sound, self.sfx_volume());
            play_sound_once(sound);
        }
    }

    pub fn play_music(&mut self, assets: &AudioAssets, track: MusicTrack) {
        if self.current_music == Some(track) {
            return;
        }

        self.stop_music(assets);
        self.current_music = Some(track);

        if self.settings.audio_muted || self.paused {
            return;
        }

        if let Some(sound) = self.music_sound(assets, track) {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume: self.music_volume(),
                },
            );
        }
    }

    pub fn stop_music(&mut self, assets: &AudioAssets) {
        if let Some(track) = self.current_music.take() {
            if let Some(sound) = self.music_sound(assets, track) {
                stop_sound(sound);
            }
        }
    }

    pub fn set_paused(&mut self, assets: &AudioAssets, paused: bool) {
        if self.paused == paused {
            return;
        }

        self.paused = paused;

        if paused {
            if let Some(track) = self.current_music {
                if let Some(sound) = self.music_sound(assets, track) {
                    stop_sound(sound);
                }
            }

            if let Some(sound) = &assets.engine_loop {
                stop_sound(sound);
                self.engine_loop_playing = false;
            }
            return;
        }

        if let Some(track) = self.current_music {
            if let Some(sound) = self.music_sound(assets, track) {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: true,
                        volume: self.music_volume(),
                    },
                );
            }
        }

        if self.engine_loop_active {
            self.set_engine_active(assets, true, self.engine_loop_intensity);
        }
    }

    pub fn set_engine_active(&mut self, assets: &AudioAssets, active: bool, intensity: f32) {
        self.engine_loop_intensity = intensity.clamp(0.0, 1.0);
        self.engine_loop_active = active;

        if !active {
            if let Some(sound) = &assets.engine_loop {
                stop_sound(sound);
            }
            self.engine_loop_playing = false;
            return;
        }

        if self.settings.audio_muted || self.paused {
            return;
        }

        if let Some(sound) = &assets.engine_loop {
            if self.engine_loop_playing {
                set_sound_volume(sound, self.sfx_volume() * self.engine_loop_intensity);
            } else {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: true,
                        volume: self.sfx_volume() * self.engine_loop_intensity,
                    },
                );
                self.engine_loop_playing = true;
            }
        }
    }

    pub fn settings(&self) -> &AudioSettings {
        &self.settings
    }

    pub fn adjust_master_volume(&mut self, assets: &AudioAssets, delta: f32) {
        self.settings.master_volume = (self.settings.master_volume + delta).clamp(0.0, 1.0);
        self.refresh_active_audio(assets);
    }

    pub fn adjust_music_volume(&mut self, assets: &AudioAssets, delta: f32) {
        self.settings.music_volume = (self.settings.music_volume + delta).clamp(0.0, 1.0);
        self.refresh_active_audio(assets);
    }

    pub fn adjust_sfx_volume(&mut self, assets: &AudioAssets, delta: f32) {
        self.settings.sfx_volume = (self.settings.sfx_volume + delta).clamp(0.0, 1.0);
        self.refresh_active_audio(assets);
    }

    pub fn toggle_mute(&mut self, assets: &AudioAssets) {
        self.settings.audio_muted = !self.settings.audio_muted;
        self.refresh_active_audio(assets);
    }

    fn refresh_active_audio(&mut self, assets: &AudioAssets) {
        if self.settings.audio_muted {
            if let Some(track) = self.current_music {
                if let Some(sound) = self.music_sound(assets, track) {
                    stop_sound(sound);
                }
            }
            if let Some(sound) = &assets.engine_loop {
                stop_sound(sound);
                self.engine_loop_playing = false;
            }
            return;
        }

        if let Some(track) = self.current_music {
            if let Some(sound) = self.music_sound(assets, track) {
                stop_sound(sound);
                if !self.paused {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: true,
                            volume: self.music_volume(),
                        },
                    );
                }
            }
        }

        if let Some(sound) = &assets.engine_loop {
            if self.engine_loop_active && !self.paused {
                if self.engine_loop_playing {
                    set_sound_volume(sound, self.sfx_volume() * self.engine_loop_intensity);
                } else {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: true,
                            volume: self.sfx_volume() * self.engine_loop_intensity,
                        },
                    );
                    self.engine_loop_playing = true;
                }
            } else {
                stop_sound(sound);
                self.engine_loop_playing = false;
            }
        }
    }

    fn music_sound<'a>(
        &self,
        assets: &'a AudioAssets,
        track: MusicTrack,
    ) -> &'a Option<macroquad::audio::Sound> {
        match track {
            MusicTrack::Menu => &assets.menu_loop,
            MusicTrack::Gameplay => &assets.gameplay_loop,
        }
    }

    fn music_volume(&self) -> f32 {
        (self.settings.master_volume * self.settings.music_volume).clamp(0.0, 1.0)
    }

    fn sfx_volume(&self) -> f32 {
        (self.settings.master_volume * self.settings.sfx_volume).clamp(0.0, 1.0)
    }
}
