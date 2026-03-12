//! Cross-platform CPU affinity helpers for Rust threads.
//!
//! ## Example
//!
//! This example shows how to create a thread for each available processor and pin each thread to its corresponding processor.
//!
//! ```
//! extern crate core_affinity;
//!
//! use std::thread;
//!
//! // Retrieve the IDs of all active CPU cores.
//! let core_ids = core_affinity::get_core_ids().unwrap();
//!
//! // Create a thread for each active CPU core.
//! let handles = core_ids.into_iter().map(|id| {
//!     thread::spawn(move || {
//!         // Pin this thread to a single CPU core.
//!         let res = core_affinity::set_for_current(id);
//!         if (res) {
//!             // Do more work after this.
//!         }
//!     })
//! }).collect::<Vec<_>>();
//!
//! for handle in handles.into_iter() {
//!     handle.join().unwrap();
//! }
//! ```

#[cfg(feature = "errors")]
/// Crate result type used by the optional `errors` feature.
pub type Result<T> = std::result::Result<T, AffinityError>;

#[cfg(feature = "errors")]
/// Error values returned by the feature-gated `try_*` APIs.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AffinityError {
    /// Core id discovery failed for the current runtime/platform context.
    #[error("failed to retrieve available core IDs")]
    GetCoreIdsFailed,
    /// Thread affinity could not be applied to the requested core id.
    #[error("failed to set affinity for core ID {core_id}")]
    SetForCurrentFailed { core_id: usize },
}

/// This function tries to retrieve information
/// on all the "cores" on which the current thread
/// is allowed to run.
#[must_use]
pub fn get_core_ids() -> Option<Vec<CoreId>> {
    platform::get_core_ids()
}

#[cfg(feature = "errors")]
/// Returns all core IDs available to the current thread/process affinity scope.
///
/// # Errors
///
/// Returns [`AffinityError::GetCoreIdsFailed`] when core IDs cannot be retrieved
/// for the current platform or process context.
pub fn try_get_core_ids() -> Result<Vec<CoreId>> {
    core_ids_to_result(get_core_ids())
}

/// This function tries to pin the current
/// thread to the specified core.
///
/// # Arguments
///
/// * `core_id` - ID of the core to pin
#[must_use]
pub fn set_for_current(core_id: CoreId) -> bool {
    platform::set_for_current(core_id)
}

#[cfg(feature = "errors")]
/// Pins the current thread to a specific core.
///
/// # Errors
///
/// Returns [`AffinityError::SetForCurrentFailed`] when setting thread affinity
/// fails for the supplied [`CoreId`].
pub fn try_set_for_current(core_id: CoreId) -> Result<()> {
    set_for_current_to_result(core_id, set_for_current(core_id))
}

#[cfg(feature = "errors")]
fn core_ids_to_result(core_ids: Option<Vec<CoreId>>) -> Result<Vec<CoreId>> {
    core_ids.ok_or(AffinityError::GetCoreIdsFailed)
}

#[cfg(feature = "errors")]
fn set_for_current_to_result(core_id: CoreId, success: bool) -> Result<()> {
    if success {
        Ok(())
    } else {
        Err(AffinityError::SetForCurrentFailed {
            core_id: core_id.id,
        })
    }
}

/// This represents a CPU core.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoreId {
    /// Zero-based logical core index understood by this crate.
    pub id: usize,
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[path = "platform/linux.rs"]
mod platform;

#[cfg(target_os = "windows")]
#[path = "platform/windows.rs"]
mod platform;

#[cfg(target_os = "macos")]
#[path = "platform/macos.rs"]
mod platform;

#[cfg(target_os = "freebsd")]
#[path = "platform/freebsd.rs"]
mod platform;

#[cfg(target_os = "netbsd")]
#[path = "platform/netbsd.rs"]
mod platform;

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "windows",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "netbsd"
)))]
#[path = "platform/stub.rs"]
mod platform;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_core_ids() {
        match get_core_ids() {
            Some(set) => {
                assert_eq!(set.len(), num_cpus::get());
            }
            None => {
                panic!("get_core_ids() returned None");
            }
        }
    }

    #[test]
    fn test_set_for_current() {
        let ids = get_core_ids().unwrap();
        assert!(!ids.is_empty());
        assert!(set_for_current(ids[0]));
    }

    #[cfg(feature = "errors")]
    #[test]
    fn test_core_ids_to_result_success() {
        let ids = vec![CoreId { id: 0 }];
        let res = core_ids_to_result(Some(ids.clone()));
        assert_eq!(res, Ok(ids));
    }

    #[cfg(feature = "errors")]
    #[test]
    fn test_core_ids_to_result_error() {
        let res = core_ids_to_result(None);
        assert_eq!(res, Err(AffinityError::GetCoreIdsFailed));
    }

    #[cfg(feature = "errors")]
    #[test]
    fn test_set_for_current_to_result_success() {
        let core_id = CoreId { id: 3 };
        let res = set_for_current_to_result(core_id, true);
        assert_eq!(res, Ok(()));
    }

    #[cfg(feature = "errors")]
    #[test]
    fn test_set_for_current_to_result_error() {
        let core_id = CoreId { id: 7 };
        let res = set_for_current_to_result(core_id, false);
        assert_eq!(res, Err(AffinityError::SetForCurrentFailed { core_id: 7 }));
    }
}
