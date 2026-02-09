use crate::may_be_clap_bundle::may_be_clap_bundle;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct ClapFinder {
    paths: Vec<PathBuf>,
    follow_links: bool,
}

impl ClapFinder {
    pub fn new<P: Into<PathBuf>>(paths: impl IntoIterator<Item = P>) -> Self {
        Self {
            paths: paths.into_iter().map(P::into).collect(),
            follow_links: true,
        }
    }

    pub fn follow_links(mut self, yes: bool) -> Self {
        self.follow_links = yes;
        self
    }
}

impl IntoIterator for ClapFinder {
    type Item = PathBuf;
    type IntoIter = ClapFinderIter;

    fn into_iter(self) -> Self::IntoIter {
        ClapFinderIter {
            paths: self.paths.into_iter(),
            follow_links: self.follow_links,
            current_walkdir: None,
        }
    }
}

pub struct ClapFinderIter {
    follow_links: bool,
    paths: std::vec::IntoIter<PathBuf>,
    current_walkdir: Option<walkdir::IntoIter>,
}

impl Iterator for ClapFinderIter {
    type Item = PathBuf;

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
            if !may_be_clap_bundle(entry.path(), entry.file_type()) {
                continue;
            }

            // We may have a CLAP bundle.

            // On Windows / Linux / other UNIXes, this is simple, as CLAP bundles are always files.
            #[cfg(not(target_os = "macos"))]
            return Some(entry.into_path());

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

                return Some(executable_path);
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn get_executable_path_of_bundle(bundle_path: &std::path::Path) -> Option<PathBuf> {
    use objc2_foundation::{NSBundle, NSURL};

    let url = NSURL::from_directory_path(bundle_path)?;
    let bundle = NSBundle::bundleWithURL(&url)?;

    let executable_url = bundle.executableURL()?.absoluteURL()?;
    let executable_path = executable_url.to_file_path()?;

    Some(executable_path)
}
