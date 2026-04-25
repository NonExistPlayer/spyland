/*
 *  spylandd — background daemon for continuous screen time tracking
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use anyhow::{Context, Result};
use log::{debug, info, warn};
use spyland_core::Clock;
use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::app::App;
use spyland_lib::{db::Db, ipc::IpcServer};

mod app;

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Starting spyland daemon...");

    let db_path = {
        let state_path = match env::var("XDG_STATE_HOME") {
            Ok(dir) => PathBuf::from(dir),
            Err(err) => {
                warn!("XDG_STATE_HOME is not set: {err}");
                let home = env::var("HOME").context("Home directory is not set")?;
                PathBuf::from(home).join(".local/state/")
            }
        }
        .join("spyland");

        debug!("State path: {state_path:?}");

        if !state_path.exists() {
            warn!("State path does not exist");
            fs::create_dir_all(&state_path).context("Failed to create state dir")?;
        }

        let filename = if cfg!(debug_assertions) {
            warn!("Running in DEBUG version! Using separate database file.");
            "sessions-debug.sqlite"
        } else {
            "sessions.sqlite"
        };

        format!("{}/{filename}", state_path.display())
    };

    let sock_path = {
        let runtime_dir = env::var("XDG_RUNTIME_DIR")?;

        debug!("Runtime path: {runtime_dir:?}");

        let filename = if cfg!(debug_assertions) {
            warn!("Running in DEBUG version! Using separate socket.");
            "spyland-debug.sock"
        } else {
            "spyland.sock"
        };

        let sock = format!("{runtime_dir}/{filename}");

        if fs::exists(&sock)? {
            warn!("Socket exists! Removing...");
            fs::remove_file(&sock)?;
        }

        sock
    };

    let app = App::new(
        Db::open(db_path, true).await?,
        IpcServer::new(sock_path.into())?,
        SystemClock {},
    )
    .await?;

    app.run().await
}

struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs()
    }
}
