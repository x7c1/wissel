/// A browser installed on the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Browser {
    /// The desktop file id, for example `firefox_firefox.desktop`.
    pub id: String,
    /// The display name.
    pub name: String,
    /// The icon in string form: a theme icon name or a file path.
    pub icon: Option<String>,
}
