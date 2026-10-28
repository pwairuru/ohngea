//! Provision template: yealink — verbatim port of templates/provision/yealink/base.j2.
//! Embedded via include_str! (single-binary prod, no file I/O).

pub const BASE: &str = include_str!("../../../../../templates/provision/yealink/base.j2");

pub const T29G: &str = include_str!("../../../../../templates/provision/yealink/t29g.j2");
pub const T46S: &str = include_str!("../../../../../templates/provision/yealink/t46s.j2");
