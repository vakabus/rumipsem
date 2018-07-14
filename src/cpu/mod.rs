//! `cpu` module houses all components, that are directly related to the actual CPU and not
//! the system running on it.

pub mod bitutils;
pub mod control;
pub mod event;
pub mod instructions;
pub mod registers;
pub mod watchdog;
pub mod instructions_constants;
pub mod float;
