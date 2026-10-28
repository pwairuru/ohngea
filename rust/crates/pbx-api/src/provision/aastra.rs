//! Provision template: aastra — verbatim port of templates/provision/aastra/base.j2.
//! Embedded via include_str! (single-binary prod, no file I/O).

pub const BASE: &str = include_str!("../../../../../templates/provision/aastra/base.j2");
