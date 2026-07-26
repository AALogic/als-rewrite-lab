use crate::parse_als_path;

pub(crate) fn filename_from_candidates(
    path: Option<&str>,
    relative_path: Option<&str>,
) -> Option<String> {
    first_non_empty(path, relative_path).and_then(|value| parse_als_path(value).filename)
}

pub(crate) fn extension_from_candidates(
    path: Option<&str>,
    relative_path: Option<&str>,
) -> Option<String> {
    first_non_empty(path, relative_path).and_then(|value| parse_als_path(value).extension)
}

fn first_non_empty<'a>(path: Option<&'a str>, relative_path: Option<&'a str>) -> Option<&'a str> {
    path.filter(|value| !value.trim().is_empty())
        .or_else(|| relative_path.filter(|value| !value.trim().is_empty()))
}
