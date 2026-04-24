#![no_std]
#![doc(html_no_source)]

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
mod sys;

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
mod macros;

pub use macros::*;
pub use sys::*;
