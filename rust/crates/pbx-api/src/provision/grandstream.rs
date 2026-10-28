//! Provision template: grandstream — verbatim port of templates/provision/grandstream/base.j2.
//! Embedded via include_str! (single-binary prod, no file I/O).

pub const BASE: &str = include_str!("../../../../../templates/provision/grandstream/base.j2");

pub const GXP2170: &str = include_str!("../../../../../templates/provision/grandstream/gxp2170.j2");
