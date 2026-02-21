use std::fmt::{Debug, Formatter};
use std::fs::FileType;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A builder to create an iterator that yields paths to possible CLAP dynamic library files.
///
/// This type implements [`IntoIterator`] so that it may be used as the subject of a `for` loop
/// directly.
///
/// This documentation refers only to "possible" CLAP files, which means the file paths yielded
/// by this type are not guaranteed to actually be CLAP files. It only seeks `.clap` files in the
/// standard (or given) locations and filters candidates using information provided by the file
/// system, but it never actually tries to open or load said files, both for performance and
/// safety reasons (due to arbitrary code execution).
///
/// This makes this type safe to use in any environment.
///
/// # Platform Compatibility Notes
///
/// On macOS, a CLAP files is not a dynamic library file, but a standard [macOS bundle] instead,
/// contrary to Windows and Linux CLAP bundles.
///
/// This type does not yield the path to the CLAP macOS bundle itself, but when it does find one,
/// it [opens] it and locates the path to its declared executable files, and yields that path instead.
///
/// This means, for example, that while this type might yield a path like
/// `/usr/lib/clap/Surge XT.clap` on Linux, it will yield a path like
/// `/Library/Audio/Plug-Ins/CLAP/Surge XT.clap/Contents/MacOS/Surge XT` on macOS.
///
/// This allows you to directly use the yielded paths with e.g. `clack-host` or `libloading`,
/// regardless of the platform.
///
/// # Example
///
/// ```
/// use clack_finder::ClapFinder;
///
/// for bundle_path in ClapFinder::from_standard_paths() {
///     println!("Found possible CLAP bundle at: {bundle_path:?}");
///     // Load the bundle using e.g. clack-host or libloading, etc.
/// }
///
/// ```
///
/// [macOS bundle]: https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/AboutBundles/AboutBundles.html
/// [opens]: https://developer.apple.com/documentation/foundation/bundle
#[derive(Clone)]
pub struct ClapFinder {
    paths: Vec<PathBuf>,
    follow_links: bool,
}

impl ClapFinder {
    /// Creates a new builder for an iterator that will search CLAP bundles in the directories with
    /// the provided paths.
    ///
    /// Note that the provided paths do not have to point to existing files or directories.
    /// If the provided directories do not exist or cannot be opened for any reason, they will be
    /// skipped.
    ///
    /// If you want to only seek standard CLAP locations, you can use
    /// [`ClapFinder::from_standard_paths`] instead.
    #[inline]
    pub fn new<P: Into<PathBuf>>(paths: impl IntoIterator<Item = P>) -> Self {
        Self {
            paths: paths.into_iter().map(P::into).collect(),
            follow_links: true,
        }
    }

    /// Creates a new builder for an iterator that will search CLAP bundles in the standard CLAP
    /// locations.
    ///
    /// The directory paths used are the ones provided by the
    /// [`standard_clap_paths`](crate::standard_clap_paths) function.
    #[inline]
    pub fn from_standard_paths() -> Self {
        Self {
            paths: crate::standard_clap_paths(),
            follow_links: true,
        }
    }

    /// Appends a given list of paths to the paths that will be searched by this [`ClapFinder`].
    #[inline]
    pub fn add_paths(mut self, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        self.extend(paths);
        self
    }

    /// Follow symbolic links. By default, this is *enabled*.
    #[inline]
    pub fn follow_links(mut self, yes: bool) -> Self {
        self.follow_links = yes;
        self
    }
}

impl IntoIterator for ClapFinder {
    type Item = PotentialClapFile;
    type IntoIter = ClapFinderIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ClapFinderIter {
            paths: self.paths.into_iter(),
            follow_links: self.follow_links,
            current_walkdir: None,
        }
    }
}

impl Extend<PathBuf> for ClapFinder {
    #[inline]
    fn extend<T: IntoIterator<Item = PathBuf>>(&mut self, iter: T) {
        self.paths.extend(iter);
    }
}

/// An iterator for searching CLAP bundle files.
///
/// The order of elements yielded by this iterator is unspecified.
/// If any encountered directory cannot be opened, then it is skipped.
///
/// See the [`ClapFinder`] documentation for more information.
pub struct ClapFinderIter {
    follow_links: bool,
    paths: std::vec::IntoIter<PathBuf>,
    current_walkdir: Option<walkdir::IntoIter>,
}

impl Iterator for ClapFinderIter {
    type Item = PotentialClapFile;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let walkdir = match &mut self.current_walkdir {
                // Get the current Walkdir iterator, if it exists
                Some(walkdir) => walkdir,
                // Pop a new path to use, return None if we're out of paths
                None => {
                    let next_path = self.paths.next()?;

                    let walkdir = WalkDir::new(next_path)
                        .follow_links(self.follow_links)
                        .into_iter();

                    self.current_walkdir.insert(walkdir)
                }
            };

            // If this walkdir is empty, get rid of it and prepare go to next iteration.
            let Some(entry) = walkdir.next() else {
                self.current_walkdir = None;
                continue;
            };

            // Filter out failed entries.
            let Ok(entry) = entry else {
                continue;
            };

            // If there is no way the current file can be a CLAP bundle, filter the entry out
            // and keep going or descending.
            if !may_be_clap_file(entry.path(), entry.file_type()) {
                continue;
            }

            // We may have a CLAP bundle.

            // On Windows / Linux / other UNIXes, this is simple, as CLAP bundles are always files.
            #[cfg(not(target_os = "macos"))]
            return Some(PotentialClapFile {
                bundle_path: entry.into_path(),
            });

            #[cfg(target_os = "macos")]
            {
                // Extract its executable path (for macOS).
                let Some(executable_path) = get_executable_path_of_bundle(entry.path()) else {
                    // Do not skip current dir here: if this failed, it means the directory is not actually
                    // a bundle (or at least not a CLAP one), so we may need to descend into it.
                    continue;
                };

                // Do not descend into the current directory.
                // This directory is actually a bundle, so there is nothing for us there.
                walkdir.skip_current_dir();

                return Some(PotentialClapFile {
                    executable_path: Some(executable_path),
                    bundle_path: entry.into_path(),
                });
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn get_executable_path_of_bundle(bundle_path: &Path) -> Option<PathBuf> {
    use objc2_foundation::{NSBundle, NSURL};

    let url = NSURL::from_directory_path(bundle_path)?;
    let bundle = NSBundle::bundleWithURL(&url)?;

    let executable_url = bundle.executableURL()?.absoluteURL()?;
    let executable_path = executable_url.to_file_path()?;

    Some(executable_path)
}

fn may_be_clap_file(path: &Path, file_type: FileType) -> bool {
    #[cfg(target_os = "macos")]
    if file_type.is_file() {
        return false;
    }

    #[cfg(not(target_os = "macos"))]
    if file_type.is_dir() {
        return false;
    }

    let Some(ext) = path.extension() else {
        return false;
    };

    ext == "clap"
}

/// Information about a potential CLAP plugin file.
///
/// Use this to retrieve a CLAP file's [`bundle_path`](Self::bundle_path) and
/// [`executable_path`](Self::executable_path).
#[derive(Clone)]
pub struct PotentialClapFile {
    bundle_path: PathBuf,
    #[cfg(target_os = "macos")]
    executable_path: Option<PathBuf>,
}

impl PotentialClapFile {
    /// Returns the path of the CLAP file.
    ///
    /// This is the path you want to pass to a CLAP entry's `init` function.
    #[inline]
    pub fn bundle_path(&self) -> &Path {
        &self.bundle_path
    }

    /// Returns the path of the CLAP file, consuming this [`PotentialClapFile`].
    #[inline]
    pub fn into_bundle_path(self) -> PathBuf {
        self.bundle_path
    }

    /// Returns the path to the executable dynamic library file of the CLAP file.
    ///
    /// On most platforms this is the same file as the CLAP file itself, but on macOS it is
    /// the executable file of the standard bundle instead.
    #[inline]
    pub fn executable_path(&self) -> &Path {
        #[cfg(target_os = "macos")]
        {
            self.executable_path.as_ref().unwrap_or(&self.bundle_path)
        }

        #[cfg(not(target_os = "macos"))]
        {
            &self.bundle_path
        }
    }
}

impl Debug for PotentialClapFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(target_os = "macos")]
        {
            f.debug_struct("ClapFile")
                .field("bundle_path", &self.bundle_path)
                .field("executable_path", &self.executable_path)
                .finish()
        }
        #[cfg(not(target_os = "macos"))]
        {
            f.debug_tuple("ClapFile").field(&self.bundle_path).finish()
        }
    }
}
