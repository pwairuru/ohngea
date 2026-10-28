//! Provision template: escene — verbatim port of templates/provision/escene/base.j2.
//! Embedded via include_str! (single-binary prod, no file I/O).

pub const BASE: &str = include_str!("../../../../../templates/provision/escene/base.j2");
