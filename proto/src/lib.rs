// Copyright 2020 Google LLC
//
// Use of this source code is governed by an MIT-style license that can be found
// in the LICENSE file or at https://opensource.org/licenses/MIT.

pub mod common {
    include!(concat!(env!("OUT_DIR"), "/fleetspeak.rs"));
}

pub mod channel {
    include!(concat!(env!("OUT_DIR"), "/fleetspeak.channel.rs"));
}
