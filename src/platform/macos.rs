//! macOS CPU affinity implementation.

use std::mem;

use libc::c_int;
use libc::c_uint;
use libc::pthread_self;

use super::CoreId;

type KernReturnT = c_int;
type IntegerT = c_int;
type NaturalT = c_uint;
type ThreadT = c_uint;
type ThreadPolicyFlavorT = NaturalT;
type MachMsgTypeNumberT = NaturalT;

#[repr(C)]
struct ThreadAffinityPolicyDataT {
    affinity_tag: IntegerT,
}

type ThreadPolicyT = *mut ThreadAffinityPolicyDataT;

const THREAD_AFFINITY_POLICY: ThreadPolicyFlavorT = 4;

unsafe extern "C" {
    fn thread_policy_set(
        thread: ThreadT,
        flavor: ThreadPolicyFlavorT,
        policy_info: ThreadPolicyT,
        count: MachMsgTypeNumberT,
    ) -> KernReturnT;
}

/// Returns core ids available on macOS.
pub fn get_core_ids() -> Option<Vec<CoreId>> {
    Some(
        (0..num_cpus::get())
            .map(|n| CoreId { id: n as usize })
            .collect::<Vec<_>>(),
    )
}

/// Attempts to pin the current thread to the provided macOS core id.
pub fn set_for_current(core_id: CoreId) -> bool {
    let thread_affinity_policy_count: MachMsgTypeNumberT =
        mem::size_of::<ThreadAffinityPolicyDataT>() as MachMsgTypeNumberT
            / mem::size_of::<IntegerT>() as MachMsgTypeNumberT;

    let mut info = ThreadAffinityPolicyDataT {
        affinity_tag: core_id.id as IntegerT,
    };

    let res = unsafe {
        thread_policy_set(
            pthread_self() as ThreadT,
            THREAD_AFFINITY_POLICY,
            &mut info as ThreadPolicyT,
            thread_affinity_policy_count,
        )
    };
    res == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_get_core_ids() {
        match get_core_ids() {
            Some(set) => {
                assert_eq!(set.len(), num_cpus::get());
            }
            None => {
                panic!("macos::get_core_ids() returned None");
            }
        }
    }

    #[test]
    fn test_macos_set_for_current() {
        let ids = get_core_ids().unwrap();
        assert!(!ids.is_empty());
        assert!(set_for_current(ids[0]))
    }
}
