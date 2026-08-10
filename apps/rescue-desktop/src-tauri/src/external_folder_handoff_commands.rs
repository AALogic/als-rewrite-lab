use crate::external_folder_handoff::{
    error, opened, validate_request, ExternalProviderOpenError, ExternalProviderOpenRequest,
    ExternalProviderOpenResult,
};

const PROVIDER_URL: &str = "https://wetransfer.com/";

#[tauri::command(rename_all = "snake_case")]
pub(crate) fn open_external_provider(
    request: ExternalProviderOpenRequest,
) -> Result<ExternalProviderOpenResult, ExternalProviderOpenError> {
    validate_request(&request)?;
    open_provider_url().map_err(|_| {
        error(
            "PROVIDER_OPEN_FAILED",
            "open_provider",
            "The default browser could not open the provider.",
        )
    })?;
    Ok(opened(&request))
}

#[cfg(target_os = "macos")]
fn open_provider_url() -> Result<(), ()> {
    use objc2_app_kit::{NSWorkspace, NSWorkspaceOpenConfiguration};
    use objc2_foundation::{NSString, NSURL};

    let value = NSString::from_str(PROVIDER_URL);
    let url = NSURL::URLWithString(&value).ok_or(())?;
    let workspace = NSWorkspace::sharedWorkspace();
    if workspace.URLForApplicationToOpenURL(&url).is_none() {
        return Err(());
    }
    let configuration = NSWorkspaceOpenConfiguration::configuration();
    configuration.setActivates(true);
    workspace.openURL_configuration_completionHandler(&url, &configuration, None);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn open_provider_url() -> Result<(), ()> {
    tauri_plugin_opener::open_url(PROVIDER_URL, None::<&str>).map_err(|_| ())
}
