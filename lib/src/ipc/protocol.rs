/*
 *  spyland-lib — public library API for accessing spyland
 *  part of the spyland project
 *  Copyright (C) 2026 Ilya Korobov (NonExistPlayer)
 *  SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use anyhow::Result;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Request {
    Ping,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Response {
    Pong,
}

pub fn send<T: Serialize>(stream: &UnixStream, serializable: T) -> Result<()> {
    let json = serde_json::to_string(&serializable)?;
    let mut writer = stream;
    writeln!(writer, "{json}")?;
    Ok(())
}

pub fn read<T: DeserializeOwned>(stream: &UnixStream) -> Result<T> {
    let mut json = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut json)?;

    let deserializable: T = serde_json::from_str(&json)?;

    Ok(deserializable)
}
