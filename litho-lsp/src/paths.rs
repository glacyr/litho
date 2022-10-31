use lsp_types::Url;

use std::path::{Component, Path, Prefix};

// This is copied from rust-analyzer.
pub fn url_from_path<P>(path: P) -> Result<Url, ()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let url = Url::from_file_path(path)?;

    match path.components().next() {
        Some(Component::Prefix(prefix))
            if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)) =>
        {
            // Need to lowercase driver letter.
        }
        _ => return Ok(url),
    }

    let driver_letter_range = {
        let parts = url.as_str().splitn(3, ':').collect::<Vec<_>>();
        let (scheme, drive_letter) = match parts.as_slice() {
            [scheme, drive_letter, _rest] => (scheme, drive_letter),
            _ => return Ok(url),
        };

        let start = scheme.len() + ':'.len_utf8();
        start..(start + drive_letter.len())
    };

    let mut url: String = url.into();
    url[driver_letter_range].make_ascii_lowercase();
    Url::parse(&url).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::url_from_path;

    #[cfg(target_os = "macos")]
    #[test]
    fn test_url_from_path_macos() {
        // Now try again with lowercase drive letter.
        let url = url_from_path("/Users/cutsoy/Documents/example.graphql").unwrap();
        assert_eq!(
            String::from(url),
            "file:///Users/cutsoy/Documents/example.graphql"
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_url_from_path_windows() {
        // Now try again with lowercase drive letter.
        let url = url_from_path("c:\\Users\\cutsoy\\Documents\\example.graphql").unwrap();
        assert_eq!(
            String::from(url),
            "file:///c:/Users/cutsoy/Documents/example.graphql"
        );

        // Now try again with uppercase drive letter.
        let url = url_from_path("C:\\Users\\cutsoy\\Documents\\example.graphql").unwrap();
        assert_eq!(
            String::from(url),
            "file:///c:/Users/cutsoy/Documents/example.graphql"
        );
    }
}
