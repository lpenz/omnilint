// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Helpers for analysing whole repositories.

use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use color_eyre::Result;
use tokio::process::Command;
use tokio_process_stream::{Item as ProcessItem, ProcessLineStream};
use tokio_stream::Stream;

/// Streams the paths of all the files tracked by git in the current
/// repository, obtained asynchronously with `git ls-files`.
pub fn git_ls_files() -> Result<impl Stream<Item = PathBuf> + Unpin> {
    let mut cmd = Command::new("git");
    cmd.arg("ls-files");
    let inner = ProcessLineStream::try_from(cmd)?;
    Ok(GitLsFiles { inner })
}

/// Stream of `git ls-files` output, one path per line.
struct GitLsFiles {
    inner: ProcessLineStream,
}

impl Stream for GitLsFiles {
    type Item = PathBuf;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            match ready!(Pin::new(&mut this.inner).poll_next(cx)) {
                Some(ProcessItem::Stdout(line)) => {
                    if !line.is_empty() {
                        return Poll::Ready(Some(PathBuf::from(line)));
                    }
                }
                Some(ProcessItem::Stderr(line)) => {
                    eprintln!("[git ls-files] stderr {}", line);
                }
                Some(ProcessItem::Done(status)) => match status {
                    Ok(status) if status.success() => {}
                    Ok(status) => eprintln!("[git ls-files] exited with {status}"),
                    Err(error) => eprintln!("[git ls-files] error: {error}"),
                },
                None => return Poll::Ready(None),
            }
        }
    }
}
