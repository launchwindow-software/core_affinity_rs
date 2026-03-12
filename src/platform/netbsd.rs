//! NetBSD CPU affinity implementation.

use libc::_cpuset_create;
use libc::_cpuset_destroy;
use libc::_cpuset_isset;
use libc::_cpuset_set;
use libc::_cpuset_size;
use libc::cpuset_t;
use libc::pthread_getaffinity_np;
use libc::pthread_self;
use libc::pthread_setaffinity_np;

use super::CoreId;

/// Returns core ids available to the current thread on NetBSD.
pub fn get_core_ids() -> Option<Vec<CoreId>> {
    if let Some(full_set) = get_affinity_mask() {
        let mut core_ids: Vec<CoreId> = Vec::new();

        let num_cpus = num_cpus::get();
        for i in 0..num_cpus {
            if unsafe { _cpuset_isset(i as u64, full_set) } >= 0 {
                core_ids.push(CoreId { id: i });
            }
        }
        unsafe { _cpuset_destroy(full_set) };
        Some(core_ids)
    } else {
        None
    }
}

/// Attempts to pin the current thread to the given NetBSD core id.
pub fn set_for_current(core_id: CoreId) -> bool {
    let set = unsafe { _cpuset_create() };
    unsafe { _cpuset_set(core_id.id as u64, set) };

    let result = unsafe { pthread_setaffinity_np(pthread_self(), _cpuset_size(set), set) };
    unsafe { _cpuset_destroy(set) };

    result == 0
}

fn get_affinity_mask() -> Option<*mut cpuset_t> {
    let set = unsafe { _cpuset_create() };

    match unsafe { pthread_getaffinity_np(pthread_self(), _cpuset_size(set), set) } {
        0 => Some(set),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netbsd_get_affinity_mask() {
        match get_affinity_mask() {
            Some(set) => unsafe {
                _cpuset_destroy(set);
            },
            None => {
                panic!("netbsd::get_affinity_mask() returned None");
            }
        }
    }

    #[test]
    fn test_netbsd_get_core_ids() {
        match get_core_ids() {
            Some(set) => {
                assert_eq!(set.len(), num_cpus::get());
            }
            None => {
                panic!("netbsd::get_core_ids() returned None");
            }
        }
    }

    #[test]
    fn test_netbsd_set_for_current() {
        let ids = get_core_ids().unwrap();

        assert!(!ids.is_empty());

        let ci = ids[ids.len() - 1]; // use the last reported core
        let res = set_for_current(ci);
        assert!(res);

        // Ensure that the system pinned the current thread
        // to the specified core.
        let new_mask = get_affinity_mask().unwrap();
        assert!(unsafe { _cpuset_isset(ci.id as u64, new_mask) > 0 });
        let num_cpus = num_cpus::get();
        for i in 0..num_cpus {
            if i != ci.id {
                assert_eq!(0, unsafe { _cpuset_isset(i as u64, new_mask) });
            }
        }
        unsafe { _cpuset_destroy(new_mask) };
    }
}
