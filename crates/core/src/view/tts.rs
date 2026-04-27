//! Text-to-Speech (TTS) UI View
//!
//! Provides UI controls for TTS playback in the reader.
//! Includes play/pause/stop buttons, speed control, and status display.
//!
//! This view is only available on platforms that support TTS (Android, Desktop).
//! It is not available on Kobo e-readers due to lack of audio hardware.

use crate::context::Context;
use crate::font::Fonts;
use crate::geom::Rectangle;
use crate::tts::{TtsEngine, TtsOptions, TtsSettings, TtsState, create_tts_engine, is_tts_supported};
use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::icon::Icon;
use crate::view::label::Label;
use crate::view::slider::Slider;
use crate::view::{Align, Bus, Event, Hub, Id, RenderQueue, View, ID_FEEDER};
use anyhow::{Error, Result};

pub const BUTTON_HEIGHT: i32 = 48;
pub const BUTTON_SPACING: i32 = 8;
pub const PADDING: i32 = 16;
pub const SLIDER_WIDTH: i32 = 150;

/// TTS Control View for reader integration
///
/// Provides a compact control bar with play/pause/stop buttons,
/// speed adjustment slider, and status display.
pub struct TtsView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    /// TTS engine instance
    tts_engine: Option<Box<dyn TtsEngine>>,
    /// Current TTS settings
    settings: TtsSettings,
    /// Current text to speak (if any)
    current_text: Option<String>,
    /// Whether TTS is supported on this platform
    tts_supported: bool,
}

impl TtsView {
    /// Create a new TTS view
    ///
    /// # Arguments
    /// * `rect` - The rectangle for the view
    /// * `rq` - Render queue for initial setup
    /// * `context` - Application context
    pub fn new(
        rect: Rectangle,
        _rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<Self, Error> {
        let id = ID_FEEDER.next();
        let mut children = Vec::new();
        let tts_supported = is_tts_supported();

        // Try to initialize TTS engine if supported
        let tts_engine = if tts_supported {
            match create_tts_engine() {
                Ok(mut engine) => {
                    if let Err(e) = engine.initialize() {
                        log::warn!("Failed to initialize TTS engine: {}", e);
                        None
                    } else {
                        Some(engine)
                    }
                }
                Err(e) => {
                    log::warn!("TTS not available: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let settings = TtsSettings::default();
        let mut x = rect.min.x + PADDING;
        let y = rect.min.y + PADDING;

        if tts_engine.is_some() {
            // Play button
            let play_btn = Icon::new(
                rect![x, y, x + BUTTON_HEIGHT, y + BUTTON_HEIGHT],
                Event::Select(crate::view::EntryId::TtsPlay),
                "play",
            );
            children.push(Box::new(play_btn) as Box<dyn View>);
            x += BUTTON_HEIGHT + BUTTON_SPACING;

            // Pause button
            let pause_btn = Icon::new(
                rect![x, y, x + BUTTON_HEIGHT, y + BUTTON_HEIGHT],
                Event::Select(crate::view::EntryId::TtsPause),
                "pause",
            );
            children.push(Box::new(pause_btn) as Box<dyn View>);
            x += BUTTON_HEIGHT + BUTTON_SPACING;

            // Stop button
            let stop_btn = Icon::new(
                rect![x, y, x + BUTTON_HEIGHT, y + BUTTON_HEIGHT],
                Event::Select(crate::view::EntryId::TtsStop),
                "stop",
            );
            children.push(Box::new(stop_btn) as Box<dyn View>);
            x += BUTTON_HEIGHT + BUTTON_SPACING * 2;

            // Speed slider label
            let speed_label = Label::new(
                rect![x, y, x + 60, y + BUTTON_HEIGHT],
                "Speed:".to_string(),
                Align::Left(0),
            );
            children.push(Box::new(speed_label) as Box<dyn View>);
            x += 60 + BUTTON_SPACING;

            // Speed slider (0.5 to 2.0, default 1.0)
            let speed_slider = Slider::new(
                rect![x, y + (BUTTON_HEIGHT - 24) / 2, x + SLIDER_WIDTH, y + (BUTTON_HEIGHT + 24) / 2],
                Event::Select(crate::view::EntryId::TtsSetRate(1.0)),
                0.5,  // min
                2.0,  // max
                1.0,  // initial value
                0.1,  // step
            );
            children.push(Box::new(speed_slider) as Box<dyn View>);
            x += SLIDER_WIDTH + BUTTON_SPACING * 2;

            // Status label
            let status_label = Label::new(
                rect![x, y, rect.max.x - PADDING, y + BUTTON_HEIGHT],
                "Ready".to_string(),
                Align::Left(0),
            );
            children.push(Box::new(status_label) as Box<dyn View>);
        } else {
            // TTS not available message
            let msg_label = Label::new(
                rect![
                    rect.min.x + PADDING,
                    y,
                    rect.max.x - PADDING,
                    y + BUTTON_HEIGHT
                ],
                "Text-to-Speech not available on this device".to_string(),
                Align::Center,
            );
            children.push(Box::new(msg_label) as Box<dyn View>);
        }

        Ok(TtsView {
            id,
            rect,
            children,
            tts_engine,
            settings,
            current_text: None,
            tts_supported,
        })
    }

    /// Speak the given text
    ///
    /// # Arguments
    /// * `text` - The text to speak
    pub fn speak(&mut self, text: &str) -> Result<()> {
        if let Some(ref mut engine) = self.tts_engine {
            let options = TtsOptions {
                rate: self.settings.rate,
                volume: self.settings.volume,
                pitch: self.settings.pitch,
                language: self.settings.voice_id.clone(), // Using voice_id as language placeholder
                interrupt: true,
            };

            engine.speak(text, options)?;
            self.current_text = Some(text.to_string());
            self.update_status_label("Speaking...");
        } else {
            return Err(Error::msg("TTS engine not available"));
        }

        Ok(())
    }

    /// Stop current speech
    pub fn stop(&mut self) -> Result<()> {
        if let Some(ref mut engine) = self.tts_engine {
            engine.stop()?;
            self.update_status_label("Stopped");
        }
        Ok(())
    }

    /// Pause current speech (if supported)
    pub fn pause(&mut self) -> Result<()> {
        if let Some(ref mut engine) = self.tts_engine {
            engine.pause()?;
            self.update_status_label("Paused");
        }
        Ok(())
    }

    /// Set speech rate
    pub fn set_rate(&mut self, rate: f32) -> Result<()> {
        self.settings.rate = rate.clamp(0.5, 2.0);

        if let Some(ref mut engine) = self.tts_engine {
            engine.set_rate(self.settings.rate)?;
        }

        Ok(())
    }

    /// Get current speech rate
    pub fn rate(&self) -> f32 {
        self.settings.rate
    }

    /// Check if TTS is currently speaking
    pub fn is_speaking(&self) -> bool {
        if let Some(ref engine) = self.tts_engine {
            matches!(engine.state(), TtsState::Speaking)
        } else {
            false
        }
    }

    /// Get current TTS state
    pub fn state(&self) -> TtsState {
        if let Some(ref engine) = self.tts_engine {
            engine.state()
        } else {
            TtsState::Error
        }
    }

    /// Update the status label text
    fn update_status_label(&mut self, status: &str) {
        // Find the status label (last child) and update it
        if let Some(last_child) = self.children.last_mut() {
            // Note: In a full implementation, we'd downcast to Label and update text
            // For now, we log the status
            log::info!("TTS Status: {}", status);
        }
    }

    /// Check if TTS is supported on this platform
    pub fn is_tts_supported(&self) -> bool {
        self.tts_supported
    }

    /// Check if TTS engine is ready
    pub fn is_tts_ready(&self) -> bool {
        self.tts_engine.is_some()
    }
}

impl View for TtsView {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        _bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match evt {
            Event::Select(entry_id) => {
                match entry_id {
                    crate::view::EntryId::TtsPlay => {
                        // Play/Resume
                        if let Some(ref text) = self.current_text {
                            let _ = self.speak(text);
                        }
                        true
                    }
                    crate::view::EntryId::TtsPause => {
                        let _ = self.pause();
                        true
                    }
                    crate::view::EntryId::TtsStop => {
                        let _ = self.stop();
                        true
                    }
                    crate::view::EntryId::TtsSetRate(rate) => {
                        let _ = self.set_rate(*rate);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn render(&self, _fb: &mut dyn crate::framebuffer::Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
        // Rendering is handled by children
    }

    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        &mut self.rect
    }

    fn children(&self) -> &Vec<Box<dyn View>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
        &mut self.children
    }

    fn id(&self) -> Id {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_settings_default() {
        let settings = TtsSettings::default();
        assert!(!settings.enabled);
        assert_eq!(settings.rate, 1.0);
        assert_eq!(settings.volume, 1.0);
        assert_eq!(settings.pitch, 1.0);
    }

    #[test]
    fn test_tts_state_transitions() {
        // Test that TtsState enum works correctly
        assert_ne!(TtsState::Idle, TtsState::Speaking);
        assert_ne!(TtsState::Paused, TtsState::Error);
    }
}
