pub(crate) trait StringCleanup {
    fn string_cleanup(self) -> String;
}

impl StringCleanup for String {
    fn string_cleanup(self) -> String {
        self.trim_end_matches("\0").trim().to_string()
    }
}

impl StringCleanup for &str {
    fn string_cleanup(self) -> String {
        self.trim_end_matches("\0").trim().to_string()
    }
}
