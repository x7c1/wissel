use gio::prelude::*;

use super::LaunchError;

/// Opens `urls` with the browser whose desktop file id is `id`.
///
/// GIO spawns the browser detached from wissel, so wissel may exit as soon as
/// this returns. `context` carries the activation token that lets the
/// browser's window come to the front; pass the GTK display's launch context
/// when one is available.
pub fn launch(
    id: &str,
    urls: &[String],
    context: &impl IsA<gio::AppLaunchContext>,
) -> Result<(), LaunchError> {
    let app = gio_unix::DesktopAppInfo::new(id).ok_or_else(|| LaunchError::UnknownId {
        id: id.to_owned(),
        urls: urls.to_vec(),
    })?;
    let uris: Vec<&str> = urls.iter().map(String::as_str).collect();
    app.launch_uris(&uris, Some(context))
        .map_err(|cause| LaunchError::Gio {
            browser: app.display_name().to_string(),
            urls: urls.to_vec(),
            cause,
        })
}
