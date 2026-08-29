// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Structured errors reported by the `run_*` functions.

/// The reason a run exited non-zero.
///
/// Findings and missing-linter messages are already printed to stderr as they
/// are discovered; this type only carries the reason for the non-zero exit.
#[derive(Debug, thiserror::Error)]
pub(crate) enum OmnilintError {
    /// One or more lint findings were emitted during the run.
    #[error("lint findings were emitted")]
    Findings,
    /// The inventory found one or more required linters that are unavailable.
    #[error("required linter(s) not found")]
    MissingRequiredLinters,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn findings_display() {
        assert_eq!(
            OmnilintError::Findings.to_string(),
            "lint findings were emitted"
        );
    }

    #[test]
    fn missing_required_linters_display() {
        assert_eq!(
            OmnilintError::MissingRequiredLinters.to_string(),
            "required linter(s) not found"
        );
    }
}
