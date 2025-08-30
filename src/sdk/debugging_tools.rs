//! Debugging Tools for SIS-OS Applications
//! Development and debugging utilities

#![no_std]

use alloc::string::String;
use alloc::vec::Vec;
use super::{SDKConfiguration, ProjectId, DebugConfiguration, DebugSession};

pub struct DebuggingTools;

impl DebuggingTools {
    pub fn new() -> Self { Self }
    
    pub fn initialize(&mut self, config: &SDKConfiguration) -> Result<(), DebugError> {
        Ok(())
    }

    fn str_to_string(s: &str) -> String {
        let mut string = String::new();
        string.push_str(s);
        string
    }
    
    pub fn start_debug_session(&mut self, project_id: ProjectId, config: DebugConfiguration) 
        -> Result<DebugSession, DebugError> {
        Ok(DebugSession {
            session_id: Self::str_to_string("debug-001"),
            project_id,
            status: super::DebugStatus::Running,
            breakpoints: Vec::new(),
            call_stack: Vec::new(),
        })
    }
}

#[derive(Debug)]
pub enum DebugError {
    SessionFailed,
}

impl From<DebugError> for super::SDKError {
    fn from(error: DebugError) -> Self {
        super::SDKError::DebugError(error)
    }
}