//! HTML Engine Helper Functions
//!
//! Provides utility functions for HTML document processing.

/// Check if a tag is a MathML tag
pub(super) fn is_math_tag(tag: &str) -> bool {
    matches!(
        tag,
        "math"
            | "mrow"
            | "mi"
            | "mo"
            | "mn"
            | "ms"
            | "mspace"
            | "mtext"
            | "mfrac"
            | "msqrt"
            | "mroot"
            | "mstyle"
            | "merror"
            | "mpadded"
            | "mphantom"
            | "mfenced"
            | "menclose"
            | "msub"
            | "msup"
            | "msubsup"
            | "munder"
            | "mover"
            | "munderover"
            | "mmultiscripts"
            | "mtable"
            | "mtr"
            | "mtd"
            | "mlabeledtr"
            | "maction"
    )
}
