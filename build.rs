// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::Bash;
use clap_complete::shells::Fish;
use clap_complete::shells::Zsh;
use color_eyre::{Result, eyre::eyre};
use man::prelude::*;
use std::env;
use std::error::Error;
use std::fs;
use std::path;

include!("src/cli.rs");

fn generate_man_page<P: AsRef<path::Path>>(
    cmd: &clap::Command,
    name: &str,
    outdir: P,
) -> Result<()> {
    let outdir = outdir.as_ref();
    let man_path = outdir.join(format!("{}.1", name));
    let manpage: Manual = clap2man::Manual::try_from(cmd)
        .map_err(|e| eyre!(e))?
        .into();
    std::fs::write(man_path, manpage.render())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    let cmd = Cli::command();
    let name = cmd
        .get_display_name()
        .unwrap_or_else(|| cmd.get_name())
        .to_owned();
    let mut outdir =
        path::PathBuf::from(env::var_os("OUT_DIR").ok_or_else(|| eyre!("error getting OUT_DIR"))?);
    fs::create_dir_all(&outdir)?;
    generate_man_page(&cmd, &name, &outdir)?;
    // build/romers-*/out
    outdir.pop();
    // build/romers-*
    outdir.pop();
    // build
    outdir.pop();
    // .
    // (either target/release or target/build)
    generate_man_page(&cmd, &name, &outdir)?;
    // Generate shell completions:
    generate_to(Bash, &mut cmd.clone(), &name, &outdir)?;
    generate_to(Fish, &mut cmd.clone(), &name, &outdir)?;
    generate_to(Zsh, &mut cmd.clone(), &name, &outdir)?;
    Ok(())
}
