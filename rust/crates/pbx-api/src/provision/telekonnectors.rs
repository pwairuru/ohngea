//! Provision template: telekonnectors — verbatim port of templates/provision/telekonnectors/base.j2.
//! Embedded via include_str! (single-binary prod, no file I/O).

pub const BASE: &str = include_str!("../../../../../templates/provision/telekonnectors/base.j2");
