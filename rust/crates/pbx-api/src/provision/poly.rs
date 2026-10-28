//! Provision template: poly — verbatim port of templates/provision/poly/base.j2.
//! Embedded via include_str! (single-binary prod, no file I/O).

pub const BASE: &str = include_str!("../../../../../templates/provision/poly/base.j2");

pub const VVX250: &str = include_str!("../../../../../templates/provision/poly/vvx250.j2");
pub const VVX450: &str = include_str!("../../../../../templates/provision/poly/vvx450.j2");
