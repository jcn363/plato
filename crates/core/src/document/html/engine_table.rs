use super::engine::{Engine, ResourceFetcher};
use super::layout::{
    collapse_margins, ChildArtifact, DrawCommand, DrawState, ImageCommand, LoopContext, RootData,
    StyleData, TextCommand,
};
use super::parse::parse_display;
use super::style::specified_values;
use crate::unit::pt_to_px;
use anyhow::Error;

impl Engine {
    pub(super) fn compute_column_widths(
        &mut self,
        node: super::dom::NodeRef,
        parent_style: &StyleData,
        loop_context: &LoopContext,
        stylesheet: &super::style::StyleSheet,
        root_data: &RootData,
        resource_fetcher: &mut dyn ResourceFetcher,
        draw_state: &mut DrawState,
    ) {
        if node.tag_name() == Some("tr") {
            let mut index = 0;
            for child in node.children().filter(|c| c.is_element()) {
                let colspan = child
                    .attribute("colspan")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1);
                let mut display_list = vec![Vec::new()];
                let artifact = self.build_display_list(
                    child,
                    parent_style,
                    loop_context,
                    stylesheet,
                    root_data,
                    resource_fetcher,
                    draw_state,
                    &mut display_list,
                );
                let horiz_padding =
                    artifact.sibling_style.padding.left + artifact.sibling_style.padding.right;
                let min_width = display_list
                    .into_iter()
                    .flatten()
                    .filter_map(|dc| match dc {
                        DrawCommand::Text(TextCommand { rect, .. }) => {
                            Some(rect.width() as i32 + horiz_padding)
                        }
                        DrawCommand::Image(ImageCommand { rect, .. }) => Some(
                            (rect.width() as i32)
                                .min(pt_to_px(parent_style.font_size, self.dpi).round().max(1.0)
                                    as i32)
                                + horiz_padding,
                        ),
                        _ => None,
                    })
                    .max()
                    .unwrap_or(0);
                let max_width = artifact
                    .rects
                    .into_iter()
                    .filter_map(|v| v.map(|r| r.width() as i32 + horiz_padding))
                    .max()
                    .unwrap_or(0);
                if colspan == 1 {
                    if let Some(cw) = draw_state.min_column_widths.get_mut(index) {
                        *cw = (*cw).max(min_width);
                    } else {
                        draw_state.min_column_widths.push(min_width);
                    }
                    if let Some(cw) = draw_state.max_column_widths.get_mut(index) {
                        *cw = (*cw).max(max_width);
                    } else {
                        draw_state.max_column_widths.push(max_width);
                    }
                }

                index += colspan;
            }
        } else {
            for child in node.children().filter(|c| c.is_element()) {
                self.compute_column_widths(
                    child,
                    parent_style,
                    loop_context,
                    stylesheet,
                    root_data,
                    resource_fetcher,
                    draw_state,
                );
            }
        }
    }
}
