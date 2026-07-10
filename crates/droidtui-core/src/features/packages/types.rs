//! Package domain types.

/// Filter options for `pm list packages`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageFilter {
    All,
    /// Third-party (user) packages only (`-3`).
    User,
    /// System packages only (`-s`).
    System,
    /// Enabled packages only (`-e`).
    Enabled,
    /// Disabled packages only (`-d`).
    Disabled,
}

impl PackageFilter {
    /// The `pm list packages` argument for this filter.
    pub fn arg(&self) -> &'static str {
        match self {
            PackageFilter::All => "",
            PackageFilter::User => " -3",
            PackageFilter::System => " -s",
            PackageFilter::Enabled => " -e",
            PackageFilter::Disabled => " -d",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_args() {
        assert_eq!(PackageFilter::All.arg(), "");
        assert_eq!(PackageFilter::User.arg(), " -3");
        assert_eq!(PackageFilter::System.arg(), " -s");
        assert_eq!(PackageFilter::Enabled.arg(), " -e");
        assert_eq!(PackageFilter::Disabled.arg(), " -d");
    }
}
