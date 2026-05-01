//! AI Settings View for Plato
//!
//! Provides UI for configuring AI features on desktop platforms.

use crate::context::Context;
use crate::geom::Rectangle;
use crate::settings::Settings;
use crate::view::button::Button;
use crate::view::label::Label;
use crate::view::{Align, Bus, EntryId, Event, RenderQueue, View};

pub const CHILD_COUNT: usize = 8;

const PROVIDERS: &[&str] = &["ollama", "openai", "claude"];
const MODELS: &[&str] = &["phi3:mini", "gpt-4", "claude-3"];

pub fn build_rows(
    rect: &Rectangle,
    y_pos: i32,
    small_height: i32,
    padding: i32,
    max_label_width: i32,
    settings: &Settings,
) -> (Vec<Box<dyn View>>, i32) {
    let mut children = Vec::new();
    let mut y = y_pos;

    // AI Enable toggle
    let enable_label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "AI Features".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(enable_label) as Box<dyn View>);

    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::ToggleAiFeature),
        if settings.ai.enabled {
            "On".to_string()
        } else {
            "Off".to_string()
        },
    );
    children.push(Box::new(toggle) as Box<dyn View>);

    y += small_height;

    // Provider selection
    let provider_label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Provider".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(provider_label) as Box<dyn View>);

    let provider_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let provider_btn = Button::new(
        provider_rect,
        Event::Select(EntryId::ToggleAiProvider),
        settings.ai.provider.clone(),
    );
    children.push(Box::new(provider_btn) as Box<dyn View>);

    y += small_height;

    // Model selection
    let model_label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Model".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(model_label) as Box<dyn View>);

    let model_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let model_btn = Button::new(
        model_rect,
        Event::Select(EntryId::ToggleAiModel),
        settings.ai.model.clone(),
    );
    children.push(Box::new(model_btn) as Box<dyn View>);

    y += small_height;

    // Endpoint display
    let endpoint_label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Endpoint".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(endpoint_label) as Box<dyn View>);

    let endpoint_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let endpoint_text = settings
        .ai
        .endpoint
        .clone()
        .unwrap_or_else(|| "localhost:11434".to_string());
    let endpoint_btn = Button::new(
        endpoint_rect,
        Event::Select(EntryId::ToggleAiEndpoint),
        endpoint_text,
    );
    children.push(Box::new(endpoint_btn) as Box<dyn View>);

    y += small_height;

    // API Key display
    let api_key_label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "API Key".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(api_key_label) as Box<dyn View>);

    let api_key_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let api_key_text = if settings.ai.api_key.is_some() {
        "***".to_string()
    } else {
        "Set API Key".to_string()
    };
    let api_key_btn = Button::new(
        api_key_rect,
        Event::Select(EntryId::ToggleAiApiKey),
        api_key_text,
    );
    children.push(Box::new(api_key_btn) as Box<dyn View>);

    (children, y)
}

pub fn handle_event(
    evt: &Event,
    children: &mut [Box<dyn View>],
    offset: usize,
    _bus: &mut Bus,
    _rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    match evt {
        Event::Select(EntryId::ToggleAiFeature) => {
            context.settings.ai.enabled = !context.settings.ai.enabled;
            if let Some(btn) = children[offset + 1].downcast_mut::<Button>() {
                let txt = if context.settings.ai.enabled {
                    "On"
                } else {
                    "Off"
                };
                btn.set_text(txt);
            }
            true
        }
        Event::Select(EntryId::ToggleAiProvider) => {
            let current = &context.settings.ai.provider;
            let idx = PROVIDERS.iter().position(|p| *p == current).unwrap_or(0);
            let next_idx = (idx + 1) % PROVIDERS.len();
            context.settings.ai.provider = PROVIDERS[next_idx].to_string();
            if let Some(btn) = children[offset + 3].downcast_mut::<Button>() {
                btn.set_text(&context.settings.ai.provider);
            }
            true
        }
        Event::Select(EntryId::ToggleAiModel) => {
            let current = &context.settings.ai.model;
            let idx = MODELS.iter().position(|m| *m == current).unwrap_or(0);
            let next_idx = (idx + 1) % MODELS.len();
            context.settings.ai.model = MODELS[next_idx].to_string();
            if let Some(btn) = children[offset + 5].downcast_mut::<Button>() {
                btn.set_text(&context.settings.ai.model);
            }
            true
        }
        Event::Select(EntryId::ToggleAiEndpoint) => {
            // For now, cycle through common endpoints
            let endpoints = [
                "http://localhost:11434",
                "https://api.openai.com",
                "https://api.anthropic.com",
            ];
            let current = context
                .settings
                .ai
                .endpoint
                .as_deref()
                .unwrap_or("http://localhost:11434");
            let idx = endpoints.iter().position(|e| *e == current).unwrap_or(0);
            let next_idx = (idx + 1) % endpoints.len();
            context.settings.ai.endpoint = Some(endpoints[next_idx].to_string());
            if let Some(btn) = children[offset + 7].downcast_mut::<Button>() {
                let text = context.settings.ai.endpoint.clone().unwrap_or_default();
                btn.set_text(&text);
            }
            true
        }
        Event::Select(EntryId::ToggleAiApiKey) => {
            // Cycle through demo API keys (in production, this would open a secure input dialog)
            let demo_keys = [
                None,
                Some("sk-demo-openai".to_string()),
                Some("sk-demo-claude".to_string()),
            ];
            let current_has_key = context.settings.ai.api_key.is_some();
            let next_idx = if current_has_key { 0 } else { 1 };
            context.settings.ai.api_key = demo_keys[next_idx].clone();
            let text = if context.settings.ai.api_key.is_some() {
                "***".to_string()
            } else {
                "Set API Key".to_string()
            };
            if let Some(btn) = children[offset + 9].downcast_mut::<Button>() {
                btn.set_text(&text);
            }
            true
        }
        _ => false,
    }
}
