use super::button::Button;
use super::icon::Icon;
use super::label::Label;
use super::slider::Slider;
use super::{Align, Bus, EntryId, Event, Hub, Id, RenderQueue, SliderId, View, ViewId, ID_FEEDER};
use super::{BORDER_RADIUS_MEDIUM, SMALL_BAR_HEIGHT, THICKNESS_LARGE};
use crate::color::{BLACK, WHITE};
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::font::{font_from_style, Fonts, NORMAL_STYLE};
use crate::framebuffer::Framebuffer;
use crate::geom::{BorderSpec, CornerSpec, Rectangle};
use crate::log_error;
use crate::settings::Settings;
use crate::unit::scale_by_dpi;

const LABEL_SAVE: &str = "Save";

pub struct SettingsEditor {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    dirty: bool,
}

impl SettingsEditor {
    pub fn new(context: &mut Context) -> SettingsEditor {
        let id = ID_FEEDER.next();
        let fonts = &mut context.fonts;
        let settings = &context.settings;
        let mut children = Vec::new();
        let dpi = CURRENT_DEVICE.dpi;
        let (width, height) = context.display.dims;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as i32;
        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;

        let (x_height, padding) = {
            let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
            (font.x_heights.0 as i32, font.em() as i32)
        };

        let window_width = width as i32 - 2 * padding;
        let window_height = small_height * 2 + 2 * padding + x_height;

        let max_label_width = {
            let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
            [
                "Frontlight",
                "WiFi",
                "Inverted",
                "Sleep Cover",
                "Auto Suspend (min)",
                "Auto Power Off (h)",
                "Auto Dual Page",
                "Finished Action",
                "Language",
                "UI Font",
            ]
            .iter()
            .map(|t| font.plan(t, None, None).width)
            .max()
            .expect("scale failed") as i32
        };

        let dx = (width as i32 - window_width) / 2;
        let dy = (height as i32 - window_height) / 3;

        let rect = rect![dx, dy, dx + window_width, dy + window_height];

        let corners = CornerSpec::Detailed {
            north_west: 0,
            north_east: border_radius - thickness,
            south_east: 0,
            south_west: 0,
        };

        let close_icon = Icon::new(
            "close",
            rect![
                rect.max.x - small_height,
                rect.min.y + thickness,
                rect.max.x - thickness,
                rect.min.y + small_height
            ],
            Event::Close(ViewId::SettingsEditor),
        )
        .corners(Some(corners));

        children.push(Box::new(close_icon) as Box<dyn View>);

        let label = Label::new(
            rect![
                rect.min.x + small_height,
                rect.min.y + thickness,
                rect.max.x - small_height,
                rect.min.y + small_height
            ],
            "Settings".to_string(),
            Align::Center,
        );

        children.push(Box::new(label) as Box<dyn View>);

        let mut y_pos = rect.min.y + 2 * small_height;

        let frontlight_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Frontlight".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(frontlight_label) as Box<dyn View>);

        let frontlight_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let frontlight_toggle = Button::new(
            frontlight_rect,
            Event::Select(EntryId::ToggleFrontlight),
            if settings.frontlight { "On" } else { "Off" }.to_string(),
        );
        children.push(Box::new(frontlight_toggle) as Box<dyn View>);

        y_pos += small_height;

        let wifi_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "WiFi".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(wifi_label) as Box<dyn View>);

        let wifi_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let wifi_toggle = Button::new(
            wifi_rect,
            Event::Select(EntryId::ToggleWifi),
            if settings.wifi { "On" } else { "Off" }.to_string(),
        );
        children.push(Box::new(wifi_toggle) as Box<dyn View>);

        y_pos += small_height;

        let inverted_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Inverted".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(inverted_label) as Box<dyn View>);

        let inverted_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let inverted_toggle = Button::new(
            inverted_rect,
            Event::Select(EntryId::ToggleInverted),
            if settings.inverted { "On" } else { "Off" }.to_string(),
        );
        children.push(Box::new(inverted_toggle) as Box<dyn View>);

        y_pos += small_height;

        let sleepcover_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Sleep Cover".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(sleepcover_label) as Box<dyn View>);

        let sleepcover_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let sleepcover_toggle = Button::new(
            sleepcover_rect,
            Event::Select(EntryId::ToggleSleepCover),
            if settings.sleep_cover { "On" } else { "Off" }.to_string(),
        );
        children.push(Box::new(sleepcover_toggle) as Box<dyn View>);

        y_pos += small_height;

        let suspend_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Auto Suspend (min)".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(suspend_label) as Box<dyn View>);

        let suspend_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let suspend_slider = Slider::new(
            suspend_rect,
            SliderId::AutoSuspend,
            settings.auto_suspend,
            5.0,
            60.0,
        );
        children.push(Box::new(suspend_slider) as Box<dyn View>);

        y_pos += small_height;

        let poweroff_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Auto Power Off (h)".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(poweroff_label) as Box<dyn View>);

        let poweroff_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let poweroff_slider = Slider::new(
            poweroff_rect,
            SliderId::AutoPowerOff,
            settings.auto_power_off,
            0.5,
            12.0,
        );
        children.push(Box::new(poweroff_slider) as Box<dyn View>);

        y_pos += small_height;

        let dualpage_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Auto Dual Page".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(dualpage_label) as Box<dyn View>);

        let dualpage_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let dualpage_toggle = Button::new(
            dualpage_rect,
            Event::Select(EntryId::ToggleDualPage),
            if settings.reader.auto_dual_page {
                "On"
            } else {
                "Off"
            }
            .to_string(),
        );
        children.push(Box::new(dualpage_toggle) as Box<dyn View>);

        y_pos += small_height;

        let finished_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Finished Action".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(finished_label) as Box<dyn View>);

        let finished_text = match settings.reader.finished {
            crate::settings::FinishedAction::Notify => "Notify",
            crate::settings::FinishedAction::Close => "Close",
            crate::settings::FinishedAction::GoToNext => "Go To Next",
        };
        let finished_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let finished_toggle = Button::new(
            finished_rect,
            Event::Select(EntryId::CycleFinishedAction),
            finished_text.to_string(),
        );
        children.push(Box::new(finished_toggle) as Box<dyn View>);

        y_pos += small_height;

        let lang_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Language".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(lang_label) as Box<dyn View>);

        let lang_text = if settings.language == "es" {
            "Español"
        } else {
            "English"
        };
        let lang_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let lang_toggle = Button::new(
            lang_rect,
            Event::Select(EntryId::CycleLanguage),
            lang_text.to_string(),
        );
        children.push(Box::new(lang_toggle) as Box<dyn View>);

        y_pos += small_height;

        let uifont_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "UI Font".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(uifont_label) as Box<dyn View>);

        let uifont_text = match settings.ui_font {
            crate::settings::UiFont::SansSerif => "Sans Serif",
            crate::settings::UiFont::Serif => "Serif",
        };
        let uifont_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let uifont_toggle = Button::new(
            uifont_rect,
            Event::Select(EntryId::CycleUiFont),
            uifont_text.to_string(),
        );
        children.push(Box::new(uifont_toggle) as Box<dyn View>);

        y_pos += small_height;

        let manga_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Manga Mode".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(manga_label) as Box<dyn View>);

        let manga_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let manga_toggle = Button::new(
            manga_rect,
            Event::Select(EntryId::ToggleMangaMode),
            if settings.reader.manga_mode {
                "On"
            } else {
                "Off"
            }
            .to_string(),
        );
        children.push(Box::new(manga_toggle) as Box<dyn View>);

        y_pos += small_height;

        let mupdf_search_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "MuPDF Search".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(mupdf_search_label) as Box<dyn View>);

        let mupdf_search_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let mupdf_search_toggle = Button::new(
            mupdf_search_rect,
            Event::Select(EntryId::ToggleMupdfSearch),
            if settings.reader.use_mupdf_search {
                "On"
            } else {
                "Off"
            }
            .to_string(),
        );
        children.push(Box::new(mupdf_search_toggle) as Box<dyn View>);

        y_pos += small_height;

        let showtime_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Show Time".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(showtime_label) as Box<dyn View>);

        let showtime_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let showtime_toggle = Button::new(
            showtime_rect,
            Event::Select(EntryId::ToggleShowTime),
            if settings.reader.show_time {
                "On"
            } else {
                "Off"
            }
            .to_string(),
        );
        children.push(Box::new(showtime_toggle) as Box<dyn View>);

        y_pos += small_height;

        let showbattery_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Show Battery".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(showbattery_label) as Box<dyn View>);

        let showbattery_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let showbattery_toggle = Button::new(
            showbattery_rect,
            Event::Select(EntryId::ToggleShowBattery),
            if settings.reader.show_battery {
                "On"
            } else {
                "Off"
            }
            .to_string(),
        );
        children.push(Box::new(showbattery_toggle) as Box<dyn View>);

        y_pos += small_height;

        let extstorage_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Ext. Storage".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(extstorage_label) as Box<dyn View>);

        let extstorage_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let extstorage_toggle = Button::new(
            extstorage_rect,
            Event::Select(EntryId::ToggleExternalStorage),
            if settings.external_storage.enabled {
                "On"
            } else {
                "Off"
            }
            .to_string(),
        );
        children.push(Box::new(extstorage_toggle) as Box<dyn View>);

        y_pos += small_height;

        let dither_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Dithering".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(dither_label) as Box<dyn View>);

        let dither_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let dither_toggle = Button::new(
            dither_rect,
            Event::Select(EntryId::ToggleDithered),
            if context.fb.dithered() { "On" } else { "Off" }.to_string(),
        );
        children.push(Box::new(dither_toggle) as Box<dyn View>);

        y_pos += small_height;

        let fastpt_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Fast Page Turn".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(fastpt_label) as Box<dyn View>);

        let fastpt_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let fastpt_toggle = Button::new(
            fastpt_rect,
            Event::Select(EntryId::ToggleFastPageTurn),
            if settings.reader.fast_page_turn {
                "On"
            } else {
                "Off"
            }
            .to_string(),
        );
        children.push(Box::new(fastpt_toggle) as Box<dyn View>);

        y_pos += small_height;

        let anim_label = Label::new(
            rect![
                rect.min.x + padding,
                y_pos,
                rect.min.x + max_label_width + padding,
                y_pos + small_height
            ],
            "Page Animation".to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(anim_label) as Box<dyn View>);

        let anim_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y_pos,
            rect.max.x - padding,
            y_pos + small_height
        ];
        let anim_text = match settings.reader.page_turn_animation {
            crate::settings::PageTurnAnimation::None => "None",
            crate::settings::PageTurnAnimation::Slide => "Slide",
            crate::settings::PageTurnAnimation::Fade => "Fade",
            crate::settings::PageTurnAnimation::Flip => "Flip",
        };
        let anim_toggle = Button::new(
            anim_rect,
            Event::Select(EntryId::CyclePageTurnAnimation),
            anim_text.to_string(),
        );
        children.push(Box::new(anim_toggle) as Box<dyn View>);

        y_pos += small_height;

        let button_height = x_height * 3;
        let button_width = window_width - 2 * padding;
        let save_rect = rect![
            rect.min.x + padding,
            y_pos,
            rect.min.x + button_width + padding,
            y_pos + button_height
        ];
        let save_button = Button::new(
            save_rect,
            Event::Select(EntryId::SaveSettings),
            LABEL_SAVE.to_string(),
        );
        children.push(Box::new(save_button) as Box<dyn View>);

        let total_height = y_pos + button_height + padding;
        let final_rect = rect![dx, dy, dx + window_width, dy + total_height];

        SettingsEditor {
            id,
            rect: final_rect,
            children,
            dirty: false,
        }
    }
}

impl View for SettingsEditor {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Select(EntryId::ToggleFrontlight) => {
                context.set_frontlight(!context.settings.frontlight);
                self.dirty = true;
                if let Some(btn) = self.children[3].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.frontlight {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        rq,
                    );
                }
                bus.push_back(Event::ToggleFrontlight);
                true
            }
            Event::Select(EntryId::ToggleWifi) => {
                context.settings.wifi = !context.settings.wifi;
                self.dirty = true;
                if let Some(btn) = self.children[5].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.wifi { "On" } else { "Off" }.to_string(),
                        rq,
                    );
                }
                bus.push_back(Event::Select(EntryId::ToggleWifi));
                true
            }
            Event::Select(EntryId::ToggleInverted) => {
                context.fb.toggle_inverted();
                context.settings.inverted = context.fb.inverted();
                self.dirty = true;
                if let Some(btn) = self.children[7].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.inverted {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        rq,
                    );
                }
                true
            }
            Event::Select(EntryId::ToggleSleepCover) => {
                context.settings.sleep_cover = !context.settings.sleep_cover;
                self.dirty = true;
                if let Some(btn) = self.children[9].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.sleep_cover {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        rq,
                    );
                }
                true
            }
            Event::Slider(SliderId::AutoSuspend, value, _) => {
                context.settings.auto_suspend = value;
                self.dirty = true;
                true
            }
            Event::Slider(SliderId::AutoPowerOff, value, _) => {
                context.settings.auto_power_off = value;
                self.dirty = true;
                true
            }
            Event::Select(EntryId::ToggleDualPage) => {
                context.settings.reader.auto_dual_page = !context.settings.reader.auto_dual_page;
                self.dirty = true;
                if let Some(btn) = self.children[15].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.reader.auto_dual_page {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        rq,
                    );
                }
                true
            }
            Event::Select(EntryId::CycleFinishedAction) => {
                use crate::settings::FinishedAction;
                context.settings.reader.finished = match context.settings.reader.finished {
                    FinishedAction::Notify => FinishedAction::Close,
                    FinishedAction::Close => FinishedAction::GoToNext,
                    FinishedAction::GoToNext => FinishedAction::Notify,
                };
                self.dirty = true;
                let new_text = match context.settings.reader.finished {
                    FinishedAction::Notify => "Notify",
                    FinishedAction::Close => "Close",
                    FinishedAction::GoToNext => "Go To Next",
                };
                if let Some(btn) = self.children[19].downcast_mut::<Button>() {
                    btn.update(new_text.to_string(), rq);
                }
                true
            }
            Event::Select(EntryId::CycleLanguage) => {
                use crate::i18n::Language;
                context.settings.language = if context.settings.language == "es" {
                    "en".to_string()
                } else {
                    "es".to_string()
                };
                self.dirty = true;
                if let Some(lang) = Language::from_code(&context.settings.language) {
                    crate::i18n::set_language(lang);
                }
                let new_text = if context.settings.language == "es" {
                    "Español"
                } else {
                    "English"
                };
                if let Some(btn) = self.children[19].downcast_mut::<Button>() {
                    btn.update(new_text.to_string(), rq);
                }
                true
            }
            Event::Select(EntryId::CycleUiFont) => {
                use crate::settings::UiFont;
                context.settings.ui_font = match context.settings.ui_font {
                    UiFont::SansSerif => UiFont::Serif,
                    UiFont::Serif => UiFont::SansSerif,
                };
                self.dirty = true;
                let new_text = match context.settings.ui_font {
                    UiFont::SansSerif => "Sans Serif",
                    UiFont::Serif => "Serif",
                };
                if let Some(btn) = self.children[21].downcast_mut::<Button>() {
                    btn.update(new_text.to_string(), rq);
                }
                true
            }
            Event::Select(EntryId::ToggleMangaMode) => {
                context.settings.reader.manga_mode = !context.settings.reader.manga_mode;
                self.dirty = true;
                if let Some(btn) = self.children[23].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.reader.manga_mode {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        rq,
                    );
                }
                true
            }
            Event::Select(EntryId::ToggleMupdfSearch) => {
                context.settings.reader.use_mupdf_search =
                    !context.settings.reader.use_mupdf_search;
                self.dirty = true;
                if let Some(btn) = self.children[25].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.reader.use_mupdf_search {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        rq,
                    );
                }
                true
            }
            Event::Select(EntryId::ToggleShowTime) => {
                context.settings.reader.show_time = !context.settings.reader.show_time;
                self.dirty = true;
                if let Some(btn) = self.children[27].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.reader.show_time {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        rq,
                    );
                }
                true
            }
            Event::Select(EntryId::ToggleShowBattery) => {
                context.settings.reader.show_battery = !context.settings.reader.show_battery;
                self.dirty = true;
                if let Some(btn) = self.children[29].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.reader.show_battery {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        rq,
                    );
                }
                true
            }
            Event::Select(EntryId::ToggleExternalStorage) => {
                context.settings.external_storage.enabled =
                    !context.settings.external_storage.enabled;
                self.dirty = true;
                if let Some(btn) = self.children[31].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.external_storage.enabled {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        rq,
                    );
                }
                true
            }
            Event::Select(EntryId::ToggleDithered) => {
                context.fb.toggle_dithered();
                if let Some(btn) = self.children[33].downcast_mut::<Button>() {
                    btn.update(
                        if context.fb.dithered() { "On" } else { "Off" }.to_string(),
                        rq,
                    );
                }
                true
            }
            Event::Select(EntryId::ToggleFastPageTurn) => {
                context.settings.reader.fast_page_turn = !context.settings.reader.fast_page_turn;
                self.dirty = true;
                if let Some(btn) = self.children[35].downcast_mut::<Button>() {
                    btn.update(
                        if context.settings.reader.fast_page_turn {
                            "On"
                        } else {
                            "Off"
                        }
                        .to_string(),
                        rq,
                    );
                }
                true
            }
            Event::Select(EntryId::CyclePageTurnAnimation) => {
                use crate::settings::PageTurnAnimation;
                context.settings.reader.page_turn_animation =
                    match context.settings.reader.page_turn_animation {
                        PageTurnAnimation::None => PageTurnAnimation::Slide,
                        PageTurnAnimation::Slide => PageTurnAnimation::Fade,
                        PageTurnAnimation::Fade => PageTurnAnimation::Flip,
                        PageTurnAnimation::Flip => PageTurnAnimation::None,
                    };
                self.dirty = true;
                let new_text = match context.settings.reader.page_turn_animation {
                    PageTurnAnimation::None => "None",
                    PageTurnAnimation::Slide => "Slide",
                    PageTurnAnimation::Fade => "Fade",
                    PageTurnAnimation::Flip => "Flip",
                };
                if let Some(btn) = self.children[37].downcast_mut::<Button>() {
                    btn.update(new_text.to_string(), rq);
                }
                true
            }
            Event::Select(EntryId::SaveSettings) => {
                if self.dirty {
                    if let Err(e) = save_settings(&context.settings) {
                        log_error!("Failed to save settings: {}", e);
                    }
                }
                bus.push_back(Event::Close(ViewId::SettingsEditor));
                true
            }
            Event::Close(ViewId::SettingsEditor) => {
                bus.push_back(Event::Close(ViewId::SettingsEditor));
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;
        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;
        let border_thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as u16;

        let corners = CornerSpec::Detailed {
            north_west: 0,
            north_east: border_radius - border_thickness as i32,
            south_east: 0,
            south_west: 0,
        };

        fb.draw_rounded_rectangle_with_border(
            &self.rect,
            &corners,
            &BorderSpec {
                thickness: border_thickness,
                color: BLACK,
            },
            &WHITE,
        );
    }

    fn is_background(&self) -> bool {
        true
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

    fn view_id(&self) -> Option<ViewId> {
        Some(ViewId::SettingsEditor)
    }
}

fn save_settings(settings: &Settings) -> Result<(), crate::anyhow::Error> {
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push(crate::settings::SETTINGS_PATH);
    crate::helpers::save_toml(settings, &path)
}
