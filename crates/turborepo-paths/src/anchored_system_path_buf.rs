use std::{
    borrow::Borrow,
    fmt,
    ops::Deref,
    path::{Path, PathBuf},
};

use camino::{Utf8Component, Utf8Components, Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::{AbsoluteSystemPath, AnchoredSystemPath, PathError, RelativeUnixPathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct AnchoredSystemPathBuf(pub(crate) Utf8PathBuf);

impl TryFrom<&str> for AnchoredSystemPathBuf {
    type Error = PathError;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        let path = Utf8Path::new(path);
        if path.is_absolute() {
            return Err(PathError::NotRelative(path.to_string()));
        }

        Ok(AnchoredSystemPathBuf(path.into()))
    }
}

impl Borrow<AnchoredSystemPath> for AnchoredSystemPathBuf {
    fn borrow(&self) -> &AnchoredSystemPath {
        unsafe { AnchoredSystemPath::new_unchecked(self.0.as_path()) }
    }
}

impl AsRef<AnchoredSystemPath> for AnchoredSystemPathBuf {
    fn as_ref(&self) -> &AnchoredSystemPath {
        self.borrow()
    }
}

impl Deref for AnchoredSystemPathBuf {
    type Target = AnchoredSystemPath;

    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl TryFrom<&Path> for AnchoredSystemPathBuf {
    type Error = PathError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let path = path
            .to_str()
            .ok_or_else(|| PathError::InvalidUnicode(path.to_string_lossy().to_string()))?;

        Self::try_from(path)
    }
}

impl fmt::Display for AnchoredSystemPathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// TODO: perhaps we ought to be converting to a unix path?
impl<'a> From<&'a AnchoredSystemPathBuf> for wax::CandidatePath<'a> {
    fn from(path: &'a AnchoredSystemPathBuf) -> Self {
        path.0.as_std_path().into()
    }
}

impl AnchoredSystemPathBuf {
    pub fn new(
        root: impl AsRef<AbsoluteSystemPath>,
        path: impl AsRef<AbsoluteSystemPath>,
    ) -> Result<Self, PathError> {
        let root = root.as_ref();
        let path = path.as_ref();
        let stripped_path = path
            .as_path()
            .strip_prefix(root.as_path())
            .map_err(|_| PathError::NotParent(root.to_string(), path.to_string()))?
            .into();

        Ok(AnchoredSystemPathBuf(stripped_path))
    }

    // Produces a path from start to end, which may include directory traversal
    // tokens. Given that both parameters are absolute, we _should_ always be
    // able to produce such a path. The exception is when crossing drive letters
    // on Windows, where no such path is possible. Since a repository is
    // expected to only reside on a single drive, this shouldn't be an issue.
    pub fn relative_path_between(start: &AbsoluteSystemPath, end: &AbsoluteSystemPath) -> Self {
        // Filter the implicit "RootDir" component that exists for unix paths.
        // For windows paths, we may want an assertion that we aren't crossing drives
        let these_components = start
            .components()
            .skip_while(|c| *c == Utf8Component::RootDir)
            .collect::<Vec<_>>();
        let other_components = end
            .components()
            .skip_while(|c| *c == Utf8Component::RootDir)
            .collect::<Vec<_>>();
        let prefix_len = these_components
            .iter()
            .zip(other_components.iter())
            .take_while(|(a, b)| a == b)
            .count();
        #[cfg(windows)]
        debug_assert!(
            prefix_len >= 1,
            "Cannot traverse drives between {} and {}",
            start,
            end
        );

        let traverse_count = these_components.len() - prefix_len;
        // For every remaining non-matching segment in self, add a directory traversal
        // Then, add every non-matching segment from other
        let path = std::iter::repeat(Utf8Component::ParentDir)
            .take(traverse_count)
            .chain(other_components.into_iter().skip(prefix_len))
            .collect::<Utf8PathBuf>();

        let path: Utf8PathBuf = path_clean::clean(path)
            .try_into()
            .expect("clean should preserve utf8");

        Self(path)
    }

    pub fn from_raw(raw: impl AsRef<str>) -> Result<Self, PathError> {
        let system_path = raw.as_ref();
        Ok(Self(system_path.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn new_unchecked(path: impl Into<Utf8PathBuf>) -> Self {
        AnchoredSystemPathBuf(path.into())
    }

    // Takes in a path that has already been validated as anchored
    // via `check_name` in `turborepo-cache` and constructs an
    // `AnchoredSystemPathBuf` with no trailing slashes.
    pub fn from_validated_tar_path(path: &str) -> Self {
        let path = Utf8Path::new(path);
        // There's no easier way to remove trailing slashes in Rust
        // because `OsString`s don't allow for manipulation.
        let no_trailing_slash: Utf8PathBuf = path.components().collect();

        // We know this is indeed anchored because of `check_name`,
        // and it is indeed system because we just split and combined with the
        // system path separator above
        AnchoredSystemPathBuf::new_unchecked(no_trailing_slash)
    }

    pub fn as_path(&self) -> &Path {
        self.0.as_std_path()
    }

    pub fn to_unix(&self) -> Result<RelativeUnixPathBuf, PathError> {
        #[cfg(unix)]
        {
            return RelativeUnixPathBuf::new(self.0.as_str());
        }
        #[cfg(not(unix))]
        {
            use crate::IntoUnix;
            let unix_buf = self.0.as_path().into_unix();
            RelativeUnixPathBuf::new(unix_buf)
        }
    }

    pub fn components(&self) -> Utf8Components {
        self.0.components()
    }

    pub fn push(&mut self, path: impl AsRef<Utf8Path>) {
        self.0.push(path.as_ref());
    }
}

impl From<AnchoredSystemPathBuf> for PathBuf {
    fn from(path: AnchoredSystemPathBuf) -> PathBuf {
        path.0.into_std_path_buf()
    }
}

impl AsRef<Utf8Path> for AnchoredSystemPathBuf {
    fn as_ref(&self) -> &Utf8Path {
        self.0.as_ref()
    }
}

impl AsRef<Path> for AnchoredSystemPathBuf {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::{AbsoluteSystemPathBuf, AnchoredSystemPathBuf};

    #[test_case(&["a", "b", "c", "..", "c"], &["..", "c"] ; "re-entry self")]
    #[test_case(&["a", "b", "c", "d", "..", "d"], &["d"] ; "re-entry child")]
    // TODO reorder
    #[test_case(&["a"], &["..", ".."] ; "parent")]
    #[test_case(&["a", "b", "c"], &["."] ; "empty self")]
    #[test_case(&["a", "b", "d"], &["..", "d"] ; "sibling")]
    #[test_case(&["a", "b", "c", "d"], &["d"] ; "child")]
    #[test_case(&["e", "f"], &["..", "..", "..", "e", "f"] ; "ancestor sibling")]
    #[test_case(&[], &["..", "..", ".."] ; "root")]
    fn test_relative_path_to(input: &[&str], expected: &[&str]) {
        #[cfg(unix)]
        let root_token = "/";
        #[cfg(windows)]
        let root_token = "C:\\";

        let root = AbsoluteSystemPathBuf::new(
            [root_token, "a", "b", "c"].join(std::path::MAIN_SEPARATOR_STR),
        )
        .unwrap();
        let mut parts = vec![root_token];
        parts.extend_from_slice(input);
        let target = AbsoluteSystemPathBuf::new(parts.join(std::path::MAIN_SEPARATOR_STR)).unwrap();
        let expected =
            AnchoredSystemPathBuf::from_raw(expected.join(std::path::MAIN_SEPARATOR_STR)).unwrap();
        let result = AnchoredSystemPathBuf::relative_path_between(&root, &target);
        assert_eq!(result, expected);
    }

    #[test_case("./foo/bar/", "./foo/bar", ".\\foo\\bar" ; "with trailing slash")]
    #[test_case("foo/bar", "foo/bar", "foo\\bar" ; "no trailing slash")]
    #[test_case("", "", "" ; "empty")]
    fn test_from_validated_tar_path(
        path: &str,
        #[allow(unused_variables)] expected_unix_path: &str,
        #[allow(unused_variables)] expected_windows_path: &str,
    ) {
        let path = AnchoredSystemPathBuf::from_validated_tar_path(path);
        #[cfg(unix)]
        assert_eq!(path.as_str(), expected_unix_path);
        #[cfg(windows)]
        assert_eq!(path.as_str(), expected_windows_path);
    }
}
