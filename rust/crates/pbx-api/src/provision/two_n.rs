//! Provision template: 2n — verbatim port of templates/provision/2n/base.j2.
//! Embedded via include_str! (single-binary prod, no file I/O).

pub const BASE: &str = include_str!("../../../../../templates/provision/2n/base.j2");
