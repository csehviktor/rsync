use std::path::Path;

use crate::error::*;

pub fn collect_files(root: &Path, exclude_fn: impl Fn(&str) -> bool) -> RsyncResult<Vec<String>> {
    let mut files = Vec::new();
    visit(root, root, &mut files, &exclude_fn)?;
    files.sort();
    Ok(files)
}

fn visit(
    root: &Path,
    dir: &Path,
    files: &mut Vec<String>,
    exclude_fn: &impl Fn(&str) -> bool,
) -> RsyncResult<()> {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();

        let path = entry.path();
        let file_name = entry.file_name();

        let key = key_for(root, &path);

        if file_name.into_encoded_bytes().first() == Some(&b'.') || exclude_fn(&key) {
            continue;
        }

        let file_type = entry.file_type().unwrap();

        if file_type.is_dir() {
            visit(root, &path, files, exclude_fn)?;
        } else if file_type.is_file() {
            files.push(key);
        }
    }

    Ok(())
}

#[inline]
fn key_for(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap().to_string_lossy().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk() {
        let root = std::env::temp_dir().join(format!("rsync-walk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);

        std::fs::create_dir_all(root.join("nested/empty")).unwrap();
        std::fs::write(root.join(".hidden"), "\0").unwrap();
        std::fs::write(root.join("b"), "\0").unwrap();
        std::fs::write(root.join("a"), "\0").unwrap();
        std::fs::write(root.join("_"), "\0").unwrap();
        std::fs::write(root.join("nested/z"), "\0").unwrap();
        std::fs::write(root.join("nested/a"), "\0").unwrap();
        std::fs::write(root.join("nested/.hidden"), "\0").unwrap();

        let keys = collect_files(&root, |_| false).unwrap();

        assert_eq!(
            keys,
            vec![
                "_".to_string(),
                "a".to_string(),
                "b".to_string(),
                "nested/a".to_string(),
                "nested/z".to_string()
            ]
        );

        std::fs::remove_dir_all(&root).unwrap();
    }
}
