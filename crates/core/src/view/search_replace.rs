use crate::color::{foreground, text_normal, BLACK};
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::font::Fonts;
use crate::framebuffer::Framebuffer;
use crate::geom::Rectangle;
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::filler::Filler;
use crate::view::input_field::InputField;
use crate::view::label::Label;
use crate::view::THICKNESS_MEDIUM;
use crate::view::{Align, Bus, EntryId, Event, Hub, Id, RenderQueue, View, ViewId, ID_FEEDER};

pub struct SearchReplaceView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    search_text: String,
    replace_text: String,
    match_count: usize,
    current_match: usize,
}

impl SearchReplaceView {
    pub fn new(
        rect: Rectangle,
        search_text: &str,
        replace_text: &str,
        context: &mut Context,
    ) -> SearchReplaceView {
        let id = ID_FEEDER.next();
        let mut children = Vec::new();
        let dpi = CURRENT_DEVICE.dpi;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let row_height = scale_by_dpi(36.0, dpi) as i32;
        let padding = scale_by_dpi(10.0, dpi) as i32;
        let label_width = scale_by_dpi(60.0, dpi) as i32;
        let input_x = rect.min.x + label_width + scale_by_dpi(5.0, dpi) as i32;
        let title_padding = scale_by_dpi(4.0, dpi) as i32;

        let title_label = Label::new(
            rect![
                rect.min.x + padding,
                rect.min.y + title_padding,
                rect.max.x - padding,
                rect.min.y + title_padding + row_height / 2
            ],
            "Search & Replace".to_string(),
            Align::Left(0),
        );
        children.push(Box::new(title_label) as Box<dyn View>);

        let search_label = Label::new(
            rect![
                rect.min.x + padding,
                rect.min.y + row_height,
                rect.min.x + label_width,
                rect.min.y + row_height + row_height
            ],
            "Find:".to_string(),
            Align::Left(0),
        );
        children.push(Box::new(search_label) as Box<dyn View>);

        let search_input = InputField::new(
            rect![
                input_x,
                rect.min.y + row_height,
                rect.max.x - padding,
                rect.min.y + row_height + row_height
            ],
            ViewId::EpubEditorSearchInput,
        )
        .border(true)
        .text(search_text, context)
        .placeholder("Search text...");
        children.push(Box::new(search_input) as Box<dyn View>);

        let replace_label = Label::new(
            rect![
                rect.min.x + padding,
                rect.min.y + 2 * row_height + title_padding,
                rect.min.x + label_width,
                rect.min.y + 3 * row_height + title_padding
            ],
            "Replace:".to_string(),
            Align::Left(0),
        );
        children.push(Box::new(replace_label) as Box<dyn View>);

        let replace_input = InputField::new(
            rect![
                input_x,
                rect.min.y + 2 * row_height + title_padding,
                rect.max.x - padding,
                rect.min.y + 3 * row_height + title_padding
            ],
            ViewId::EpubEditorReplaceInput,
        )
        .border(true)
        .text(replace_text, context)
        .placeholder("Replace with...");
        children.push(Box::new(replace_input) as Box<dyn View>);

        let btn_spacing = scale_by_dpi(8.0, dpi) as i32;
        let btn_y = rect.min.y + 3 * row_height + title_padding + btn_spacing;
        let btn_height = scale_by_dpi(32.0, dpi) as i32;
        let btn_width = (rect.width() as i32 - 2 * padding - 3 * thickness) / 4;

        let prev_btn = Button::new(
            rect![
                rect.min.x + padding,
                btn_y,
                rect.min.x + padding + btn_width,
                btn_y + btn_height
            ],
            Event::Select(EntryId::PrevMatch),
            "Prev".to_string(),
        );
        children.push(Box::new(prev_btn) as Box<dyn View>);

        let next_btn = Button::new(
            rect![
                rect.min.x + padding + btn_width + thickness,
                btn_y,
                rect.min.x + padding + 2 * btn_width + thickness,
                btn_y + btn_height
            ],
            Event::Select(EntryId::NextMatch),
            "Next".to_string(),
        );
        children.push(Box::new(next_btn) as Box<dyn View>);

        let replace_ch_btn = Button::new(
            rect![
                rect.min.x + padding + 2 * btn_width + 2 * thickness,
                btn_y,
                rect.min.x + padding + 3 * btn_width + 2 * thickness,
                btn_y + btn_height
            ],
            Event::Select(EntryId::ReplaceInChapter),
            "Replace".to_string(),
        );
        children.push(Box::new(replace_ch_btn) as Box<dyn View>);

        let close_btn = Button::new(
            rect![
                rect.min.x + padding + 3 * btn_width + 3 * thickness,
                btn_y,
                rect.max.x - padding,
                btn_y + btn_height
            ],
            Event::Select(EntryId::CloseSearchReplace),
            "Close".to_string(),
        );
        children.push(Box::new(close_btn) as Box<dyn View>);

        let sep_rect = rect![
            rect.min.x,
            btn_y + btn_height,
            rect.max.x,
            btn_y + btn_height + thickness
        ];
        let separator = Filler::new(sep_rect, BLACK);
        children.push(Box::new(separator) as Box<dyn View>);

        let status_padding = scale_by_dpi(6.0, dpi) as i32;
        let bottom_padding = scale_by_dpi(4.0, dpi) as i32;
        let status_label = Label::new(
            rect![
                rect.min.x + padding,
                btn_y + btn_height + status_padding,
                rect.max.x - padding,
                rect.max.y - bottom_padding
            ],
            "0 matches".to_string(),
            Align::Center,
        );
        children.push(Box::new(status_label) as Box<dyn View>);

        SearchReplaceView {
            id,
            rect,
            children,
            search_text: search_text.to_string(),
            replace_text: replace_text.to_string(),
            match_count: 0,
            current_match: 0,
        }
    }

    pub fn update_matches(&mut self, count: usize, rq: &mut RenderQueue) {
        self.match_count = count;
        self.current_match = if count > 0 { 1 } else { 0 };
        self.update_status(rq);
    }

    pub fn set_search_text(&mut self, text: &str, rq: &mut RenderQueue, context: &mut Context) {
        self.search_text = text.to_string();
        if let Some(input) = self.children[2].downcast_mut::<InputField>() {
            input.set_text(text, true, rq, context);
        }
    }

    pub fn set_replace_text(&mut self, text: &str, rq: &mut RenderQueue, context: &mut Context) {
        self.replace_text = text.to_string();
        if let Some(input) = self.children[4].downcast_mut::<InputField>() {
            input.set_text(text, true, rq, context);
        }
    }

    fn update_status(&mut self, rq: &mut RenderQueue) {
        let status = if self.match_count == 0 {
            "No matches".to_string()
        } else if self.match_count == 1 {
            "1 match".to_string()
        } else {
            format!("{} matches", self.match_count)
        };
        if let Some(label) = self.children[10].downcast_mut::<Label>() {
            label.update(&status, rq);
        }
    }

    pub fn get_search_text(&self) -> &str {
        &self.search_text
    }

    pub fn get_replace_text(&self) -> &str {
        &self.replace_text
    }
}

impl View for SearchReplaceView {
    fn handle_event(
        &mut self,
        evt: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match evt {
            Event::Submit(ViewId::EpubEditorSearchInput, text) => {
                self.search_text = text.clone();
                bus.push_back(Event::SearchReplace);
                true
            }
            Event::Submit(ViewId::EpubEditorReplaceInput, text) => {
                self.replace_text = text.clone();
                true
            }
            Event::Select(EntryId::CloseSearchReplace) => {
                bus.push_back(Event::Close(ViewId::EpubEditor));
                true
            }
            _ => {
                for child in self.children_mut().iter_mut() {
                    if child.handle_event(evt, _hub, bus, rq, context) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let color = text_normal(theme::is_dark_mode());

        fb.draw_rectangle(&self.rect, color[0]);
        fb.draw_rectangle_outline(
            &self.rect,
            &crate::geom::BorderSpec {
                thickness: thickness as u16,
                color: foreground(theme::is_dark_mode()),
            },
        );

        for child in self.children().iter() {
            child.render(fb, rect, fonts);
        }
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
}
