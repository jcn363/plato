use crate::color::{BLACK, WHITE};
use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{halves, Rectangle};
use crate::metadata::ReadingStatistics;
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
    _reading_stats: ReadingStatistics,
}

impl StatisticsView {
    pub fn new(rect: Rectangle, rq: &mut RenderQueue, context: &mut Context) -> StatisticsView {
        let id = ID_FEEDER.next();
        let (small_height, thickness, small_thickness, big_thickness) =
            Self::calculate_layout_params();
        let statistics = context.library.compute_statistics();
        let reading_stats = ReadingStatistics::new();

        let mut children = Vec::new();

        Self::add_top_bar(&mut children, rect, small_height, small_thickness, context);
        Self::add_separator(
            &mut children,
            rect,
            small_height,
            small_thickness,
            big_thickness,
        );
        Self::add_stats_label(
            &mut children,
            rect,
            thickness,
            small_height,
            big_thickness,
            &statistics,
            &reading_stats,
        );

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        StatisticsView {
            id,
            rect,
            children,
            _statistics: statistics,
            _reading_stats: reading_stats,
        }
    }

    fn calculate_layout_params() -> (i32, i32, i32, i32) {
        let dpi = crate::unit::get_device_dpi();
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);
        (small_height, thickness, small_thickness, big_thickness)
    }

    fn add_top_bar(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        small_height: i32,
        small_thickness: i32,
        context: &mut Context,
    ) {
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
    }

    fn add_separator(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        small_height: i32,
        small_thickness: i32,
        big_thickness: i32,
    ) {
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
    }

    fn add_stats_label(
        children: &mut Vec<Box<dyn View>>,
        rect: Rectangle,
        thickness: i32,
        small_height: i32,
        big_thickness: i32,
        statistics: &LibraryStatistics,
        reading_stats: &ReadingStatistics,
    ) {
        let content_start = rect.min.y + small_height + big_thickness + thickness;
        let stats_text = Self::format_statistics(statistics, reading_stats);
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
    }

    fn format_statistics(stats: &LibraryStatistics, reading_stats: &ReadingStatistics) -> String {
        let total_time = Duration::from_secs(stats.total_reading_time);
        let hours = total_time.as_secs() / 3600;
        let minutes = (total_time.as_secs() % 3600) / 60;

        let ppm = reading_stats.pages_per_minute();
        let wpm = reading_stats.words_per_minute();
        let reading_time = Duration::from_secs(reading_stats.total_reading_time_seconds);
        let reading_hours = reading_time.as_secs() / 3600;
        let reading_minutes = (reading_time.as_secs() % 3600) / 60;

        format!(
            "Library Statistics\n\n\
            Total Books: {}\n\
            Finished: {}\n\
            Reading Time: {}h {}m\n\
            Current Streak: {} days\n\
            Longest Streak: {} days\n\
            Average Progress: {:.0}%\n\n\
            Reading Speed\n\n\
            Pages/Minute: {:.1}\n\
            Words/Minute: {:.0}\n\
            Total Reading Time: {}h {}m\n\
            Reading Streak: {} days",
            stats.total_books,
            stats.finished_books,
            hours,
            minutes,
            stats.current_streak,
            stats.longest_streak,
            stats.average_progress * 100.0,
            ppm,
            wpm,
            reading_hours,
            reading_minutes,
            reading_stats.reading_streak_days
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
