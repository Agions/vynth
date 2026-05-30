//! Syncode — AI Pair Programming Terminal (library crate)
//!
//! Re-exports all modules for both binary and integration test use.

#![allow(dead_code, unused_imports, unused_variables)]

pub mod app;
pub mod error;
pub mod config;
pub mod tui;
pub mod session;
pub mod agent;
pub mod llm;
pub mod tools;
pub mod skills;
pub mod mcp;
pub mod sandbox;
