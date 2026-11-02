//! Biblioteca do binário `vostok`, exposta também para os testes de integração.

pub mod aval;
pub mod cli;
pub mod commands;
pub mod embedded;
pub mod harness;
pub mod manifest;
pub mod models;
#[cfg(feature = "painel")]
pub mod painel;
pub mod routing;
pub mod support;
