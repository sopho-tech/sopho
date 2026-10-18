use crate::common::errors::BaseUrlError;
use std::borrow::Cow;

pub fn validated_base_url(
    setting_name: &str,
    base_url: Option<Cow<'static, str>>,
) -> Result<Option<Cow<'static, str>>, BaseUrlError> {
    let Some(base_url) = base_url else {
        return Ok(None);
    };
    let parsed_base_url: http::Uri =
        base_url
            .parse()
            .map_err(|source| BaseUrlError::Unparsable {
                setting_name: setting_name.to_string(),
                base_url: base_url.to_string(),
                source,
            })?;
    let uses_http_scheme = matches!(parsed_base_url.scheme_str(), Some("http" | "https"));
    if !uses_http_scheme || parsed_base_url.authority().is_none() {
        return Err(BaseUrlError::NotHttp {
            setting_name: setting_name.to_string(),
            base_url: base_url.to_string(),
        });
    }
    Ok(Some(base_url))
}
