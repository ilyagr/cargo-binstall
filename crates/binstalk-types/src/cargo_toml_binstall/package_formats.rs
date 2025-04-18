use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

/// Binary format enumeration
#[derive(
    Debug, Display, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, EnumString, EnumIter,
)]
#[serde(rename_all = "snake_case")]
#[strum(ascii_case_insensitive)]
pub enum PkgFmt {
    /// Download format is TAR (uncompressed)
    Tar,
    /// Download format is TAR + Bzip2
    Tbz2,
    /// Download format is TGZ (TAR + GZip)
    Tgz,
    /// Download format is TAR + XZ
    Txz,
    /// Download format is TAR + Zstd
    Tzstd,
    /// Download format is Zip
    Zip,
    /// Download format is raw / binary
    Bin,
    /// Download format is Bzip2 (uncompressed)
    Bz2,
}

impl Default for PkgFmt {
    fn default() -> Self {
        Self::Tgz
    }
}

impl PkgFmt {
    /// If self is one of the tar based formats, return Some.
    pub fn decompose(self) -> PkgFmtDecomposed {
        match self {
            PkgFmt::Tar => PkgFmtDecomposed::Tar(TarBasedFmt::Tar),
            PkgFmt::Tbz2 => PkgFmtDecomposed::Tar(TarBasedFmt::Tbz2),
            PkgFmt::Tgz => PkgFmtDecomposed::Tar(TarBasedFmt::Tgz),
            PkgFmt::Txz => PkgFmtDecomposed::Tar(TarBasedFmt::Txz),
            PkgFmt::Tzstd => PkgFmtDecomposed::Tar(TarBasedFmt::Tzstd),
            PkgFmt::Bin => PkgFmtDecomposed::Bin,
            PkgFmt::Zip => PkgFmtDecomposed::Zip,
            PkgFmt::Bz2 => PkgFmtDecomposed::Bz2,
        }
    }

    /// List of possible file extensions for the format
    /// (with prefix `.`).
    ///
    /// * `is_windows` - if true and `self == PkgFmt::Bin`, then it will return
    ///   `.exe` in additional to other bin extension names.
    pub fn extensions(self, is_windows: bool) -> &'static [&'static str] {
        match self {
            PkgFmt::Tar => &[".tar"],
            PkgFmt::Tbz2 => &[".tbz2", ".tar.bz2"],
            PkgFmt::Tgz => &[".tgz", ".tar.gz"],
            PkgFmt::Txz => &[".txz", ".tar.xz"],
            PkgFmt::Tzstd => &[".tzstd", ".tzst", ".tar.zst"],
            PkgFmt::Bin => {
                if is_windows {
                    &[".bin", "", ".exe"]
                } else {
                    &[".bin", ""]
                }
            }
            PkgFmt::Zip => &[".zip"],
            PkgFmt::Bz2 => &[".bz2"],
        }
    }

    /// Given the pkg-url template, guess the possible pkg-fmt.
    pub fn guess_pkg_format(pkg_url: &str) -> Option<Self> {
        let mut it = pkg_url.rsplitn(3, '.');
        let last = it.next()?;
        let second_last = it.next();

        let guess = match last {
            "tar" => Some(PkgFmt::Tar),

            "tbz2" => Some(PkgFmt::Tbz2),
            "bz2" => match second_last {
                Some("tar") => Some(PkgFmt::Tbz2),
                _ => Some(PkgFmt::Bz2), // Plain .bz2
            },

            "tgz" => Some(PkgFmt::Tgz),
            "gz" if second_last == Some("tar") => Some(PkgFmt::Tgz),

            "txz" => Some(PkgFmt::Txz),
            "xz" if second_last == Some("tar") => Some(PkgFmt::Txz),

            "tzstd" | "tzst" => Some(PkgFmt::Tzstd),
            "zst" if second_last == Some("tar") => Some(PkgFmt::Tzstd),

            "exe" | "bin" => Some(PkgFmt::Bin),
            "zip" => Some(PkgFmt::Zip),

            _ => None,
        };

        // Ensure we consumed the expected number of parts for tar formats
        // or exactly one part for non-tar formats.
        match guess {
            Some(PkgFmt::Tbz2 | PkgFmt::Tgz | PkgFmt::Txz | PkgFmt::Tzstd)
                if last != "tbz2"
                    && last != "tgz"
                    && last != "txz"
                    && last != "tzstd"
                    && last != "tzst" =>
            {
                // Requires .tar.<ext>
                if second_last == Some("tar") && it.next().is_some() {
                    guess
                } else {
                    None
                }
            }
            Some(PkgFmt::Tar | PkgFmt::Bz2 | PkgFmt::Bin | PkgFmt::Zip) => {
                // Requires only one extension part (or specific multi-part like .tar.bz2 handled above)
                if second_last.is_none() || (last == "bz2" && second_last != Some("tar")) {
                    guess
                } else {
                    None
                }
            }
            _ => {
                // Handles cases like .tbz2, .tgz, .txz, .tzstd, .tzst directly
                if it.next().is_some() {
                    guess
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PkgFmtDecomposed {
    Tar(TarBasedFmt),
    Bin,
    Zip,
    Bz2,
}

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq)]
pub enum TarBasedFmt {
    /// Download format is TAR (uncompressed)
    Tar,
    /// Download format is TAR + Bzip2
    Tbz2,
    /// Download format is TGZ (TAR + GZip)
    Tgz,
    /// Download format is TAR + XZ
    Txz,
    /// Download format is TAR + Zstd
    Tzstd,
}

impl From<TarBasedFmt> for PkgFmt {
    fn from(fmt: TarBasedFmt) -> Self {
        match fmt {
            TarBasedFmt::Tar => PkgFmt::Tar,
            TarBasedFmt::Tbz2 => PkgFmt::Tbz2,
            TarBasedFmt::Tgz => PkgFmt::Tgz,
            TarBasedFmt::Txz => PkgFmt::Txz,
            TarBasedFmt::Tzstd => PkgFmt::Tzstd,
        }
    }
}
