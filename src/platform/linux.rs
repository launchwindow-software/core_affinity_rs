//! Linux and Android CPU affinity implementation.

use std::mem;

use libc::CPU_ISSET;
use libc::CPU_SET;
use libc::CPU_SETSIZE;
use libc::cpu_set_t;
use libc::sched_getaffinity;
use libc::sched_setaffinity;

use super::CoreId;

/// Returns core ids available to the current thread's Linux affinity mask.
pub fn get_core_ids() -> Option<Vec<CoreId>> {
    if let Some(full_set) = get_affinity_mask() {
        let cpu_set_size = cpu_set_size();
        let mut core_ids: Vec<CoreId> = Vec::new();

        for i in 0..cpu_set_size {
            if unsafe { CPU_ISSET(i, &full_set) } {
                core_ids.push(CoreId { id: i });
            }
        }

        Some(core_ids)
    } else {
        None
    }
}

/// Attempts to pin the current thread to the given Linux core id.
pub fn set_for_current(core_id: CoreId) -> bool {
    // Turn `core_id` into a `libc::cpu_set_t` with only
    // one core active.
    let mut set = new_cpu_set();

    unsafe { CPU_SET(core_id.id, &mut set) };

    // Set the current thread's core affinity.
    let res = unsafe {
        sched_setaffinity(
            0, // Defaults to current thread
            mem::size_of::<cpu_set_t>(),
            &raw const set,
        )
    };
    res == 0
}

fn get_affinity_mask() -> Option<cpu_set_t> {
    let mut set = new_cpu_set();

    // Try to get current core affinity mask.
    let result = unsafe {
        sched_getaffinity(
            0, // Defaults to current thread
            mem::size_of::<cpu_set_t>(),
            &raw mut set,
        )
    };

    if result == 0 { Some(set) } else { None }
}

fn new_cpu_set() -> cpu_set_t {
    unsafe { mem::zeroed::<cpu_set_t>() }
}

fn cpu_set_size() -> usize {
    #[cfg(target_os = "android")]
    {
        CPU_SETSIZE
    }

    #[cfg(not(target_os = "android"))]
    {
        usize::try_from(CPU_SETSIZE).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_get_affinity_mask() {
        match get_affinity_mask() {
            Some(_) => {}
            None => {
                panic!("linux::get_affinity_mask() returned None");
            }
        }
    }

    #[test]
    fn test_linux_get_core_ids() {
        match get_core_ids() {
            Some(set) => {
                assert_eq!(set.len(), num_cpus::get());
            }
            None => {
                panic!("linux::get_core_ids() returned None");
            }
        }
    }

    #[test]
    fn test_linux_set_for_current() {
        let ids = get_core_ids().unwrap();

        assert!(!ids.is_empty());

        let res = set_for_current(ids[0]);
        assert!(res);

        // Ensure that the system pinned the current thread
        // to the specified core.
        let mut core_mask = new_cpu_set();
        unsafe { CPU_SET(ids[0].id, &mut core_mask) };

        let new_mask = get_affinity_mask().unwrap();

        let mut is_equal = true;

        let cpu_set_size = cpu_set_size();
        for i in 0..cpu_set_size {
            let is_set1 = unsafe { CPU_ISSET(i, &core_mask) };
            let is_set2 = unsafe { CPU_ISSET(i, &new_mask) };

            if is_set1 != is_set2 {
                is_equal = false;
            }
        }

        assert!(is_equal);
    }
}
