//! Text extraction structures for PDFPurr

#![allow(dead_code)]

use super::types::{FzPoint, FzQuad, FzRect};
use pdfpurr::content::analysis::TextRun;

/// Text page wrapper for PDFPurr text runs
pub struct TextPage {
    runs: Vec<TextRun>,
}

impl TextPage {
    pub fn new(runs: Vec<TextRun>) -> Self {
        Self { runs }
    }

    pub fn blocks(&self) -> Vec<TextBlock> {
        // Convert TextRuns to TextBlocks for compatibility
        if self.runs.is_empty() {
            return Vec::new();
        }

        let mut blocks = Vec::new();
        let mut current_block_runs = Vec::new();
        let mut last_y = self.runs[0].y;

        for run in &self.runs {
            // Group runs by vertical position (lines)
            if (run.y - last_y).abs() > run.height && !current_block_runs.is_empty() {
                let bbox = bbox_from_runs(&current_block_runs);
                blocks.push(TextBlock {
                    runs: current_block_runs.clone(),
                    kind: 0, // Text block
                    bbox,
                });
                current_block_runs.clear();
            }
            current_block_runs.push(run.clone());
            last_y = run.y;
        }

        if !current_block_runs.is_empty() {
            let bbox = bbox_from_runs(&current_block_runs);
            blocks.push(TextBlock {
                runs: current_block_runs,
                kind: 0,
                bbox,
            });
        }

        blocks
    }

    pub fn chars(&self) -> usize {
        self.runs.iter().map(|r| r.text.chars().count()).sum()
    }
}

/// Text block wrapper
pub struct TextBlock {
    runs: Vec<TextRun>,
    kind: i32,
    bbox: FzRect,
}

impl TextBlock {
    pub fn new(runs: Vec<TextRun>, kind: i32, bbox: FzRect) -> Self {
        Self { runs, kind, bbox }
    }

    pub fn kind(&self) -> i32 {
        self.kind
    }

    pub fn bbox(&self) -> FzRect {
        self.bbox
    }

    pub fn lines(&self) -> Vec<TextLine> {
        // Convert runs to lines
        self.runs
            .chunks(10)
            .map(|chunk| TextLine {
                runs: chunk.to_vec(),
                bbox: bbox_from_runs(chunk),
            })
            .collect()
    }

    pub fn chars(&self) -> Vec<TextChar> {
        self.runs
            .iter()
            .flat_map(|run| {
                run.text.chars().enumerate().map(move |(i, c)| {
                    let char_x = run.x + (i as f64 * run.width / run.text.len() as f64);
                    TextChar {
                        char_code: c as u32 as i32,
                        quad: FzQuad {
                            ul: FzPoint {
                                x: char_x as f32,
                                y: run.y as f32,
                            },
                            ur: FzPoint {
                                x: (char_x + run.width / run.text.len() as f64) as f32,
                                y: run.y as f32,
                            },
                            ll: FzPoint {
                                x: char_x as f32,
                                y: (run.y + run.height) as f32,
                            },
                            lr: FzPoint {
                                x: (char_x + run.width / run.text.len() as f64) as f32,
                                y: (run.y + run.height) as f32,
                            },
                        },
                        origin: 0,
                    }
                })
            })
            .collect()
    }
}

/// Text line wrapper
pub struct TextLine {
    runs: Vec<TextRun>,
    bbox: FzRect,
}

impl TextLine {
    pub fn new(runs: Vec<TextRun>, bbox: FzRect) -> Self {
        Self { runs, bbox }
    }

    pub fn bbox(&self) -> FzRect {
        self.bbox
    }

    pub fn chars(&self) -> Vec<TextChar> {
        self.runs
            .iter()
            .flat_map(|run| {
                run.text.chars().enumerate().map(move |(i, c)| {
                    let char_x = run.x + (i as f64 * run.width / run.text.len() as f64);
                    TextChar {
                        char_code: c as u32 as i32,
                        quad: FzQuad {
                            ul: FzPoint {
                                x: char_x as f32,
                                y: run.y as f32,
                            },
                            ur: FzPoint {
                                x: (char_x + run.width / run.text.len() as f64) as f32,
                                y: run.y as f32,
                            },
                            ll: FzPoint {
                                x: char_x as f32,
                                y: (run.y + run.height) as f32,
                            },
                            lr: FzPoint {
                                x: (char_x + run.width / run.text.len() as f64) as f32,
                                y: (run.y + run.height) as f32,
                            },
                        },
                        origin: 0,
                    }
                })
            })
            .collect()
    }
}

/// Text character wrapper
pub struct TextChar {
    pub char_code: i32,
    pub quad: FzQuad,
    pub origin: i32,
}

/// Calculate bounding box from text runs
pub fn bbox_from_runs(runs: &[TextRun]) -> FzRect {
    if runs.is_empty() {
        return FzRect::default();
    }

    let min_x = runs.iter().map(|r| r.x).fold(f64::INFINITY, f64::min);
    let max_x = runs
        .iter()
        .map(|r| r.x + r.width)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = runs.iter().map(|r| r.y).fold(f64::INFINITY, f64::min);
    let max_y = runs
        .iter()
        .map(|r| r.y + r.height)
        .fold(f64::NEG_INFINITY, f64::max);

    FzRect {
        x0: min_x as f32,
        y0: min_y as f32,
        x1: max_x as f32,
        y1: max_y as f32,
    }
}
