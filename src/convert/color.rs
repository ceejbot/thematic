//! Hex color helper that normalizes the leading `#`.
//!
//! Zed theme JSON requires colors in `#rrggbb` / `#rrggbbaa` form. When we
//! synthesize a translucent color by appending an alpha suffix, the source
//! color already carries a leading `#`; appending naively to the trimmed form
//! would drop the `#` and produce a value Zed silently rejects. `HexColor` owns
//! that normalization in one place so the bug cannot recur at individual call
//! sites.

/// A hex color string, normalized to carry exactly one leading `#`.
#[derive(Debug, Clone)]
pub(crate) struct HexColor(String);

impl HexColor {
    /// Wrap a hex color, normalizing to a single leading `#`
    /// (idempotent whether or not the input already has one).
    pub(crate) fn new(s: &str) -> Self {
        HexColor(format!("#{}", s.trim_start_matches('#')))
    }

    /// Append a 2-digit alpha suffix, preserving the leading `#`.
    /// Returns a plain `String` to drop into existing `Option<String>` fields.
    pub(crate) fn with_alpha(&self, alpha: &str) -> String {
        format!("{}{alpha}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_alpha_keeps_single_hash() {
        assert_eq!(HexColor::new("#1a1a1a").with_alpha("66"), "#1a1a1a66");
        // normalizes a missing '#' rather than emitting a hash-less color
        assert_eq!(HexColor::new("1a1a1a").with_alpha("66"), "#1a1a1a66");
    }
}
