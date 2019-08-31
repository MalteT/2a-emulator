//! Basic UTF8/ASCII displaying utils.

/// Simple trait to display things either as
/// UTF8 or as ASCII.
pub trait Display {
    /// Create a valid UTF8 string from self.
    fn display_utf8(&self) -> String;
    /// Create a valid ASCII only string from self.
    fn display_ascii(&self) -> String;

    /// Defaults to display_ascii unless the `utf8` feature is used.
    fn display(&self) -> String {
        #[cfg(feature = "utf8")]
        {
            self.display_utf8()
        }

        #[cfg(not(feature = "utf8"))]
        {
            self.display_ascii()
        }
    }
}

impl Display for u8 {
    fn display_ascii(&self) -> String {
        format!("{:>08b}", self)
    }
    fn display_utf8(&self) -> String {
        format!("{:>08b}", self)
            .replace('0', "○")
            .replace('1', "●")
    }
}
