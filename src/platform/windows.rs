//! Windows CPU affinity implementation.

use std::convert::TryFrom;
use std::ptr;

use windows_sys::Win32::System::SystemInformation::GROUP_AFFINITY;
use windows_sys::Win32::System::Threading::GetActiveProcessorCount;
use windows_sys::Win32::System::Threading::GetActiveProcessorGroupCount;
use windows_sys::Win32::System::Threading::GetCurrentThread;
use windows_sys::Win32::System::Threading::SetThreadAffinityMask;
use windows_sys::Win32::System::Threading::SetThreadGroupAffinity;

use super::CoreId;

#[allow(clippy::unnecessary_wraps)]
/// Returns core ids visible to the current process on Windows.
pub fn get_core_ids() -> Option<Vec<CoreId>> {
    let core_count: u32 = if let Some(group_list) = unsafe { get_group_list() } {
        let count = group_list
            .iter()
            .copied()
            .map(|group| unsafe { GetActiveProcessorCount(group) })
            .sum();
        if count > 0 {
            count
        } else {
            fallback_core_count_u32()
        }
    } else {
        fallback_core_count_u32()
    };

    let core_ids: Vec<CoreId> = (0..core_count).map(|n| CoreId { id: n as usize }).collect();

    Some(core_ids)
}

fn fallback_core_count_u32() -> u32 {
    u32::try_from(num_cpus::get()).unwrap_or(u32::MAX)
}

/// Attempts to pin the current thread to the provided Windows core id.
pub fn set_for_current(core_id: CoreId) -> bool {
    if let Some(group_list) = unsafe { get_group_list() } {
        let group_counts: Vec<(u16, usize)> = group_list
            .iter()
            .copied()
            .map(|group| (group, unsafe { GetActiveProcessorCount(group) } as usize))
            .collect();

        if let Some((group, local_index)) = resolve_group_and_local_index(core_id.id, &group_counts)
        {
            let affinity = GROUP_AFFINITY {
                Mask: 1 << local_index,
                Group: group,
                ..Default::default()
            };
            return unsafe {
                SetThreadGroupAffinity(GetCurrentThread(), &raw const affinity, ptr::null_mut())
                    != 0
            };
        }
    }

    // Fallback for environments where processor groups are unavailable.
    if core_id.id < usize::BITS as usize {
        let mask = 1usize << core_id.id;
        return unsafe { SetThreadAffinityMask(GetCurrentThread(), mask) != 0 };
    }

    false
}

fn resolve_group_and_local_index(
    global_core_id: usize,
    group_counts: &[(u16, usize)],
) -> Option<(u16, usize)> {
    let mut remaining = global_core_id;
    for (group, count) in group_counts.iter().copied() {
        if remaining < count {
            return Some((group, remaining));
        }
        remaining = remaining.saturating_sub(count);
    }
    None
}

unsafe fn get_group_list() -> Option<Vec<u16>> {
    unsafe {
        let group_count = GetActiveProcessorGroupCount();
        if group_count == 0 {
            return None;
        }

        let group_list: Vec<u16> = (0..group_count).collect();
        Some(group_list)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_windows_get_core_ids() {
        match get_core_ids() {
            Some(set) => {
                assert_eq!(set.len(), num_cpus::get());
            }
            None => {
                panic!("windows::get_core_ids() returned None");
            }
        }
    }

    #[test]
    fn test_windows_set_for_current() {
        let ids = get_core_ids().unwrap();

        assert!(!ids.is_empty());

        assert!(set_for_current(ids[0]));
    }

    #[test]
    fn test_resolve_group_and_local_index_single_group() {
        let groups = vec![(0u16, 8usize)];
        assert_eq!(resolve_group_and_local_index(0, &groups), Some((0, 0)));
        assert_eq!(resolve_group_and_local_index(7, &groups), Some((0, 7)));
        assert_eq!(resolve_group_and_local_index(8, &groups), None);
    }

    #[test]
    fn test_resolve_group_and_local_index_multi_group() {
        let groups = vec![(0u16, 4usize), (1u16, 4usize), (2u16, 2usize)];
        assert_eq!(resolve_group_and_local_index(0, &groups), Some((0, 0)));
        assert_eq!(resolve_group_and_local_index(3, &groups), Some((0, 3)));
        assert_eq!(resolve_group_and_local_index(4, &groups), Some((1, 0)));
        assert_eq!(resolve_group_and_local_index(7, &groups), Some((1, 3)));
        assert_eq!(resolve_group_and_local_index(8, &groups), Some((2, 0)));
        assert_eq!(resolve_group_and_local_index(9, &groups), Some((2, 1)));
        assert_eq!(resolve_group_and_local_index(10, &groups), None);
    }
}
