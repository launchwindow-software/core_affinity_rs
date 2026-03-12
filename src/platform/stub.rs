//! Fallback implementation for unsupported targets.

use super::CoreId;

/// Stub implementation for unsupported targets; always returns `None`.
pub fn get_core_ids() -> Option<Vec<CoreId>> {
    None
}

/// Stub implementation for unsupported targets; always returns `false`.
pub fn set_for_current(_core_id: CoreId) -> bool {
    false
}
