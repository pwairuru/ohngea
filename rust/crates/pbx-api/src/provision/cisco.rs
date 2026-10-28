//! Provision template: cisco — verbatim port of templates/provision/cisco/base.j2.
//! Embedded via include_str! (single-binary prod, no file I/O).

pub const BASE: &str = include_str!("../../../../../templates/provision/cisco/base.j2");

pub const SPA122: &str = include_str!("../../../../../templates/provision/cisco/spa122.j2");
