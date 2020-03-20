// Copyright 2020 Google LLC
//
// Use of this source code is governed by an MIT-style license that can be found
// in the LICENSE file or at https://opensource.org/licenses/MIT.

//! An Fleetspeak client connector library.
//!
//! This library exposes a set of functions for writing client-side Fleetspeak
//! services. Each of these functions operates on a global connection object
//! that is lazily established. If this global connection cannot be established,
//! the library will panic (because without this connection Fleetspeak will shut
//! down the service anyway).
//!
//! Note that each service should send startup information upon its inception
//! and continue to heartbeat from time to time to notify the Fleetspeak client
//! that it did not get stuck.

mod connection;
mod error;

use std::fs::File;
use std::io::Result;
use std::sync::Mutex;

use lazy_static::lazy_static;

use self::connection::Connection;

/// Sends a heartbeat information to the standard Fleetspeak client.
///
/// All client services should heartbeat from time to time. Otherwise, from the
/// Fleetspeak perspective, the service is unresponsive and should be restarted.
///
/// The exact frequency of the required heartbeat is defined in the service
/// configuration file.
pub fn heartbeat() -> Result<()> {
    connected(|conn| conn.heartbeat())
}

/// Sends the startup information to the standard Fleetspeak client.
///
/// All clients are required to send this information on startup. If the client
/// does not receive this information quickly enough, the service will be
/// killed.
///
/// The `version` string should contain a self-reported version of the service.
/// This data is used primarily for statistics.
pub fn startup(version: &str) -> Result<()> {
    connected(|conn| conn.startup(version))
}

/// Sends the message to the Fleetspeak server through the standard client.
///
/// The message is sent to the server-side `service` and tagged with the `kind`
/// type. Note that this message type is rather irrelevant for Fleetspeak and
/// it is up to the service what to do with this information.
pub fn send<M>(service: &str, kind: &str, data: M) -> Result<()>
where
    M: prost::Message,
{
    connected(|conn| conn.send(service, kind, data))
}

/// Receives the message from the Fleetspeak server through the standard client.
///
/// This function will block until there is a message to be read from the input.
/// Errors are reported in case of any I/O failure or if the read message was
/// malformed (e.g. it cannot be parsed to the expected type).
pub fn receive<M>() -> Result<M>
where
    M: prost::Message + Default,
{
    connected(|conn| conn.receive())
}

/// Executes the given function with the standard client connection.
///
/// Note that the standard client connection object is guarded by a mutex. It
/// might happen that the mutex becomes poisoned and this call will panic in
/// result. This should not be a problem in practice, because mutex poisoning
/// is a result of one of the threads being aborted. In case of a such scenario,
/// it is likely the service needs to be restarted anyway.
fn connected<F, T>(f: F) -> Result<T>
where
    F: FnOnce(&mut Connection<File, File>) -> Result<T>
{
    let mut conn = CONNECTION.lock().expect("poisoned connection mutex");
    f(&mut conn)
}

lazy_static! {
    static ref CONNECTION: Mutex<Connection<File, File>> = {
        let input = open("FLEETSPEAK_COMMS_CHANNEL_INFD");
        let output = open("FLEETSPEAK_COMMS_CHANNEL_OUTFD");

        let conn = Connection::new(input, output).expect("handshake failure");
        Mutex::new(conn)
    };
}

/// Opens a file object pointed by an environment variable.
///
/// Note that this function will panic if the environment variable `var` is not
/// a valid file descriptor (in which case the library cannot be initialized and
/// the service is unlikely to work anyway).
fn open(var: &str) -> File {
    let fd = std::env::var(var)
        .expect(&format!("invalid variable `{}`", var))
        .parse()
        .expect(&format!("failed to parse file descriptor"));

    // TODO: Add support for Windows.
    unsafe {
        std::os::unix::io::FromRawFd::from_raw_fd(fd)
    }
}
