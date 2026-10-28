//! Provision template: zoiper — verbatim port of templates/provision/zoiper/base.j2.
//! Embedded via include_str! (single-binary prod, no file I/O).

pub const BASE: &str = include_str!("../../../../../templates/provision/zoiper/base.j2");
