use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};
use gtk4::gdk::Display;

pub fn text_wl_copy_args() -> Vec<&'static str> {
    vec!["--type", "text/plain;charset=utf-8"]
}

pub fn gif_wl_copy_args() -> Vec<&'static str> {
    vec!["--type", "image/gif"]
}

pub async fn copy_gif_bytes(_display: &Display, url: &str) -> Result<()> {
    let bytes = reqwest::get(url).await?.error_for_status()?.bytes().await?;
    pipe_to_wl_copy(&gif_wl_copy_args(), bytes.as_ref())
}

pub fn copy_text(_display: &Display, value: &str) -> Result<()> {
    pipe_to_wl_copy(&text_wl_copy_args(), value.as_bytes())
}

fn pipe_to_wl_copy(args: &[&str], input: &[u8]) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("failed to open wl-copy stdin"))?;
        stdin.write_all(input)?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!("wl-copy failed with status {status}"));
    }

    Ok(())
}
