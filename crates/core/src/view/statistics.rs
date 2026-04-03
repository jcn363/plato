use crate::color::{BLACK, WHITE};
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{halves, Rectangle};
use crate::settings::LibraryStatistics;
use crate::unit::scale_by_dpi;
use crate::view::filler::Filler;
use crate::view::label::Label;
use crate::view::top_bar::TopBar;
use crate::view::{Align, Bus, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{Id, ID_FEEDER};
use crate::view::{SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};
use std::time::Duration;

pub struct StatisticsView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    _statistics: LibraryStatistics,
}

impl StatisticsView {
    pub fn new(rect: Rectangle, rq: &mut RenderQueue, context: &mut Context) -> StatisticsView {
        let id = ID_FEEDER.next();
        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);

        let statistics = context.library.compute_statistics();

        let mut children = Vec::new();

        let top_bar = TopBar::new(
            rect![
                rect.min.x,
                rect.min.y,
                rect.max.x,
                rect.min.y + small_height - small_thickness
            ],
            Event::Back,
            "Statistics".to_string(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        let separator = Filler::new(
            rect![
                rect.min.x,
                rect.min.y + small_height - small_thickness,
                rect.max.x,
                rect.min.y + small_height + big_thickness
            ],
            BLACK,
        );
        children.push(Box::new(separator) as Box<dyn View>);

        let content_start = rect.min.y + small_height + big_thickness + thickness;
        let _content_height = rect.max.y - content_start;

        let stats_text = Self::format_statistics(&statistics);
        let stats_label = Label::new(
            rect![
                rect.min.x + thickness,
                content_start,
                rect.max.x - thickness,
                rect.max.y
            ],
            stats_text,
            Align::Center,
        );
        children.push(Box::new(stats_label) as Box<dyn View>);

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        StatisticsView {
            id,
            rect,
            children,
            _statistics: statistics,
        }
    }

    fn format_statistics(stats: &LibraryStatistics) -> String {
        let total_time = Duration::from_secs(stats.total_reading_time);
        let hours = total_time.as_secs() / 3600;
        let minutes = (total_time.as_secs() % 3600) / 60;

        format!(
            "Library Statistics\n\n\
            Total Books: {}\n\
            Finished: {}\n\
            Reading Time: {}h {}m\n\
            Current Streak: {} days\n\
            Longest Streak: {} days\n\
            Average Progress: {:.0}%",
            stats.total_books,
            stats.finished_books,
            hours,
            minutes,
            stats.current_streak,
            stats.longest_streak,
            stats.average_progress * 100.0
        )
    }
}

impl View for StatisticsView {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        for child in self.children_mut().iter_mut() {
            if child.handle_event(evt, hub, bus, rq, context) {
                return true;
            }
        }
        false
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        if let Some(r) = self.rect().intersection(&rect) {
            fb.draw_rectangle(&r, WHITE);
        }
        for child in self.children().iter() {
            child.render(fb, rect, fonts);
        }
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
