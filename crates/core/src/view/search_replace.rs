use crate::color::{foreground, text_normal, BLACK};
use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::filler::Filler;
use crate::view::input_field::InputField;
use crate::view::label::Label;
use crate::view::THICKNESS_MEDIUM;
use crate::view::{
    Align, Bus, EntryId, Event, Hub, Id, RenderData, RenderQueue, View, ViewId, ID_FEEDER,
};

pub struct SearchReplaceView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    search_text: String,
    replace_text: String,
    match_count: usize,
    current_match: usize,
    use_regex: bool,
    case_sensitive: bool,
    whole_word: bool,
    search_history: Vec<String>,
}

impl SearchReplaceView {
    pub fn new(
        rect: Rectangle,
        search_text: &str,
        replace_text: &str,
        context: &mut Context,
    ) -> SearchReplaceView {
        let id = ID_FEEDER.next();
        let (padding, row_height, label_width, input_x, title_padding, thickness) =
            Self::calculate_layout_metrics(&rect);
        let mut children = Vec::new();

        Self::add_title_label(&mut children, &rect, padding, title_padding);
        Self::add_search_fields(
            &mut children,
            &rect,
            padding,
            row_height,
            label_width,
            input_x,
            search_text,
            context,
        );
        Self::add_replace_fields(
            &mut children,
            &rect,
            padding,
            row_height,
            label_width,
            input_x,
            title_padding,
            replace_text,
            context,
        );
        Self::add_toggle_buttons(&mut children, &rect, padding, row_height, title_padding);
        Self::add_buttons(
            &mut children,
            &rect,
            padding,
            row_height,
            title_padding,
            thickness,
        );
        Self::add_separator(
            &mut children,
            &rect,
            padding,
            row_height,
            title_padding,
            thickness,
        );
        Self::add_status_label(
            &mut children,
            &rect,
            padding,
            row_height,
            title_padding,
            thickness,
        );

        SearchReplaceView {
            id,
            rect,
            children,
            search_text: search_text.to_string(),
            replace_text: replace_text.to_string(),
            match_count: 0,
            current_match: 0,
            use_regex: false,
            case_sensitive: false,
            whole_word: false,
            search_history: Vec::new(),
        }
    }

    fn calculate_layout_metrics(rect: &Rectangle) -> (i32, i32, i32, i32, i32, i32) {
        let dpi = crate::unit::get_device_dpi();
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let row_height = scale_by_dpi(36.0, dpi) as i32;
        let padding = scale_by_dpi(10.0, dpi) as i32;
        let label_width = scale_by_dpi(60.0, dpi) as i32;
        let input_x = rect.min.x + label_width + scale_by_dpi(5.0, dpi) as i32;
        let title_padding = scale_by_dpi(4.0, dpi) as i32;
        (
            padding,
            row_height,
            label_width,
            input_x,
            title_padding,
            thickness,
        )
    }

    fn add_title_label(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        padding: i32,
        title_padding: i32,
    ) {
        let title_label = Label::new(
            rect![
                rect.min.x + padding,
                rect.min.y + title_padding,
                rect.max.x - padding,
                rect.min.y
                    + title_padding
                    + scale_by_dpi(18.0, crate::unit::get_device_dpi()) as i32
            ],
            "Search & Replace".to_string(),
            Align::Left(0),
        );
        children.push(Box::new(title_label) as Box<dyn View>);
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Search fields builder requires many parameters (children, rect, dimensions, IDs, fonts) for comprehensive field construction"
    )]
    fn add_search_fields(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        padding: i32,
        row_height: i32,
        label_width: i32,
        input_x: i32,
        search_text: &str,
        context: &mut Context,
    ) {
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
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Replace fields builder requires many parameters (children, rect, dimensions, IDs, fonts) for comprehensive field construction"
    )]
    fn add_replace_fields(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        padding: i32,
        row_height: i32,
        label_width: i32,
        input_x: i32,
        title_padding: i32,
        replace_text: &str,
        context: &mut Context,
    ) {
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
    }

    fn add_toggle_buttons(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        padding: i32,
        row_height: i32,
        title_padding: i32,
    ) {
        let dpi = crate::unit::get_device_dpi();
        let toggle_y = rect.min.y + 3 * row_height + title_padding;
        let toggle_height = scale_by_dpi(28.0, dpi) as i32;
        let toggle_width = (rect.width() as i32 - 2 * padding) / 3;

        let regex_btn = Button::new(
            rect![
                rect.min.x + padding,
                toggle_y,
                rect.min.x + padding + toggle_width,
                toggle_y + toggle_height
            ],
            Event::Select(EntryId::ToggleRegex),
            "Regex".to_string(),
        );
        children.push(Box::new(regex_btn) as Box<dyn View>);

        let case_btn = Button::new(
            rect![
                rect.min.x + padding + toggle_width,
                toggle_y,
                rect.min.x + padding + 2 * toggle_width,
                toggle_y + toggle_height
            ],
            Event::Select(EntryId::ToggleCaseSensitive),
            "Case".to_string(),
        );
        children.push(Box::new(case_btn) as Box<dyn View>);

        let word_btn = Button::new(
            rect![
                rect.min.x + padding + 2 * toggle_width,
                toggle_y,
                rect.max.x - padding,
                toggle_y + toggle_height
            ],
            Event::Select(EntryId::ToggleWholeWord),
            "Whole".to_string(),
        );
        children.push(Box::new(word_btn) as Box<dyn View>);
    }

    fn add_buttons(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        padding: i32,
        row_height: i32,
        title_padding: i32,
        thickness: i32,
    ) {
        let dpi = crate::unit::get_device_dpi();
        let btn_spacing = scale_by_dpi(8.0, dpi) as i32;
        let btn_y = rect.min.y + 4 * row_height + title_padding + btn_spacing;
        let btn_height = scale_by_dpi(32.0, dpi) as i32;
        let btn_width = (rect.width() as i32 - 2 * padding - 4 * thickness) / 5;

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

        let replace_all_btn = Button::new(
            rect![
                rect.min.x + padding + 3 * btn_width + 3 * thickness,
                btn_y,
                rect.min.x + padding + 4 * btn_width + 3 * thickness,
                btn_y + btn_height
            ],
            Event::Select(EntryId::ReplaceInDocument),
            "All".to_string(),
        );
        children.push(Box::new(replace_all_btn) as Box<dyn View>);

        let close_btn = Button::new(
            rect![
                rect.min.x + padding + 4 * btn_width + 4 * thickness,
                btn_y,
                rect.max.x - padding,
                btn_y + btn_height
            ],
            Event::Select(EntryId::CloseSearchReplace),
            "Close".to_string(),
        );
        children.push(Box::new(close_btn) as Box<dyn View>);
    }

    fn add_separator(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        _padding: i32,
        row_height: i32,
        title_padding: i32,
        thickness: i32,
    ) {
        let _dpi = crate::unit::get_device_dpi();
        let sep_y = rect.min.y + 5 * row_height + title_padding;
        let sep_rect = rect![rect.min.x, sep_y, rect.max.x, sep_y + thickness];
        let separator = Filler::new(sep_rect, BLACK);
        children.push(Box::new(separator) as Box<dyn View>);
    }

    fn add_status_label(
        children: &mut Vec<Box<dyn View>>,
        rect: &Rectangle,
        padding: i32,
        row_height: i32,
        title_padding: i32,
        _thickness: i32,
    ) {
        let btn_spacing = scale_by_dpi(8.0, crate::unit::get_device_dpi()) as i32;
        let btn_height = scale_by_dpi(32.0, crate::unit::get_device_dpi()) as i32;
        let btn_y = rect.min.y + 4 * row_height + title_padding + btn_spacing;
        let status_padding = scale_by_dpi(6.0, crate::unit::get_device_dpi()) as i32;
        let bottom_padding = scale_by_dpi(4.0, crate::unit::get_device_dpi()) as i32;
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
    }

    pub fn update_matches(&mut self, count: usize, rq: &mut RenderQueue) {
        self.match_count = count;
        self.current_match = if count > 0 { 1 } else { 0 };
        if let Some(label) = self.children.iter().find(|c| c.is::<Label>()) {
            if let Some(_lbl) = label.downcast_ref::<Label>() {
                // Update label text
            }
        }
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    pub fn toggle_regex(&mut self) {
        self.use_regex = !self.use_regex;
    }

    pub fn toggle_case_sensitive(&mut self) {
        self.case_sensitive = !self.case_sensitive;
    }

    pub fn toggle_whole_word(&mut self) {
        self.whole_word = !self.whole_word;
    }

    pub fn get_search_options(&self) -> (bool, bool, bool) {
        (self.use_regex, self.case_sensitive, self.whole_word)
    }

    pub fn add_to_search_history(&mut self, text: &str) {
        if !text.is_empty() {
            self.search_history.retain(|s| s != text);
            self.search_history.insert(0, text.to_string());
            if self.search_history.len() > 10 {
                self.search_history.truncate(10);
            }
        }
    }

    pub fn get_search_history(&self) -> &[String] {
        &self.search_history
    }

    pub fn set_search_text(&mut self, text: &str, rq: &mut RenderQueue, context: &mut Context) {
        if text.is_empty() {
            return;
        }
        self.search_text = text.to_string();
        self.add_to_search_history(text);
        if let Some(input) = self.children[2].downcast_mut::<InputField>() {
            input.set_text(text, true, rq, context);
        }
    }

    pub fn set_replace_text(&mut self, text: &str, rq: &mut RenderQueue, context: &mut Context) {
        if text.is_empty() {
            return;
        }
        self.replace_text = text.to_string();
        if let Some(input) = self.children[4].downcast_mut::<InputField>() {
            input.set_text(text, true, rq, context);
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
        let dpi = crate::unit::get_device_dpi();
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
