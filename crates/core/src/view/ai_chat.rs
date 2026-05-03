//! AI Chat View for Plato Reader
//!
//! Provides an AI chat sidebar in the reader for asking questions about the current document.
//! The actual AI processing is done via LLM providers from the `plato-ai` crate.

use super::input_field::InputField;
use super::top_bar::TopBar;
use super::{Bus, Event, Hub, Id, RenderData, RenderQueue, View, ViewId, ID_FEEDER};
use crate::color::{background, text_inverted_hard};
use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::SMALL_BAR_HEIGHT;

use plato_ai::traits::LLMProvider;
use plato_ai::AiContext;

pub trait AiProcessor: Send + Sync {
    fn process(&self, text: String) -> String;
    fn clone_box(&self) -> Box<dyn AiProcessor>;
}

pub struct LlmProcessor {
    provider: Box<dyn LLMProvider>,
    context: AiContext,
}

impl LlmProcessor {
    pub fn new(
        provider: Box<dyn LLMProvider>,
        document_path: String,
        current_page: usize,
        total_pages: usize,
    ) -> Self {
        Self {
            provider,
            context: AiContext::new(document_path, current_page, total_pages),
        }
    }
}

impl AiProcessor for LlmProcessor {
    fn process(&self, text: String) -> String {
        match self.provider.generate(&text, &self.context) {
            Ok(response) => response.content,
            Err(e) => format!("Error: {}", e),
        }
    }

    fn clone_box(&self) -> Box<dyn AiProcessor> {
        let config = plato_ai::traits::ProviderConfig::default();
        Box::new(LlmProcessor {
            provider: Box::new(plato_ai::providers::MockProvider::new(config)),
            context: self.context.clone(),
        })
    }
}

#[derive(Clone)]
pub struct EchoProcessor;

impl AiProcessor for EchoProcessor {
    fn process(&self, text: String) -> String {
        format!("Echo: {}", text)
    }
    fn clone_box(&self) -> Box<dyn AiProcessor> {
        Box::new(self.clone())
    }
}

pub struct AiChatView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    messages: Vec<(bool, String)>,
    is_processing: bool,
    status: String,
    processor: Box<dyn AiProcessor>,
}

impl AiChatView {
    pub fn new(
        rect: Rectangle,
        context: &mut Context,
        processor: Box<dyn AiProcessor>,
    ) -> AiChatView {
        let id = ID_FEEDER.next();
        let dpi = crate::unit::get_device_dpi();
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;

        let mut children: Vec<Box<dyn View>> = Vec::new();

        let top_bar = TopBar::new(
            rect![
                rect.min.x,
                rect.min.y,
                rect.max.x,
                rect.min.y + small_height
            ],
            Event::Close(ViewId::AiChat),
            "AI Assistant".to_string(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        let input_field_height = small_height;
        let input_field = InputField::new(
            rect![
                rect.min.x,
                rect.max.y - input_field_height,
                rect.max.x,
                rect.max.y
            ],
            ViewId::AiChat,
        )
        .placeholder("Ask a question...");
        children.push(Box::new(input_field) as Box<dyn View>);

        AiChatView {
            id,
            rect,
            children,
            messages: Vec::new(),
            is_processing: false,
            status: String::new(),
            processor,
        }
    }
}

impl View for AiChatView {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match evt {
            Event::Submit(ViewId::AiChat, text) => {
                let text = text.clone();
                if !self.is_processing && !text.is_empty() {
                    self.messages.push((true, text.clone()));
                    self.is_processing = true;
                    self.status = "Thinking...".to_string();

                    let processor = self.processor.clone_box();
                    let hub_cloned = hub.clone();
                    std::thread::spawn(move || {
                        let response = processor.process(text);
                        hub_cloned.send(Event::AiResponse(response)).ok();
                    });

                    if let Some(input_field) = self.children[1].downcast_mut::<InputField>() {
                        input_field.set_text("", true, rq, context);
                    }

                    rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                }
                true
            }
            Event::AiResponse(text) => {
                self.messages.push((false, text.clone()));
                self.is_processing = false;
                self.status = String::new();
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::AiError(err) => {
                self.is_processing = false;
                self.status = format!("Error: {}", err);
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            _ => {
                for child in self.children.iter_mut() {
                    if child.handle_event(evt, hub, bus, rq, context) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut Fonts) {
        let is_dark = theme::is_dark_mode();
        fb.draw_rectangle(&self.rect, background(is_dark));

        // Render messages (simple layout for now)
        let dpi = crate::unit::get_device_dpi();
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let mut y = self.rect.min.y + small_height + 10;

        let scheme = text_inverted_hard(is_dark);
        let font = &mut fonts.sans_serif.regular;

        for (is_user, text) in self.messages.iter().rev().take(5).rev() {
            let label = if *is_user { "You: " } else { "AI: " };
            let full_text = format!("{}{}", label, text);
            // Very basic wrapping/truncation for now
            let display_text = if full_text.len() > 40 {
                format!("{}...", &full_text[..37])
            } else {
                full_text
            };

            let plan = font.plan(&display_text, None, None);
            font.render(fb, scheme[1], &plan, pt!(self.rect.min.x + 10, y));
            y += 30;
        }

        if !self.status.is_empty() {
            let plan = font.plan(&self.status, None, None);
            font.render(fb, scheme[1], &plan, pt!(self.rect.min.x + 10, y));
        }

        for child in self.children.iter() {
            child.render(fb, *child.rect(), fonts);
        }
    }

    fn id(&self) -> Id {
        self.id
    }

    fn view_id(&self) -> Option<ViewId> {
        Some(ViewId::AiChat)
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
}
