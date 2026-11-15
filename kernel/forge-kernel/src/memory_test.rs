use super::*;

const TEST_FRAME_SIZE: usize = 4096;

#[test]
fn returns_distinct_aligned_pages_and_stays_exhausted() {
    let mut allocator = BootFrameAllocator::<TEST_FRAME_SIZE>::new(0x1000, 0x4000).unwrap();
    for address in [0x1000, 0x2000, 0x3000] {
        assert_eq!(allocator.allocate().unwrap().address(), address);
    }
    assert_eq!(allocator.remaining(), 0);
    assert_eq!(allocator.allocate(), None);
    assert_eq!(allocator.allocate(), None);
}

#[test]
fn rounds_inward_and_never_allocates_partial_pages() {
    let mut allocator = BootFrameAllocator::<TEST_FRAME_SIZE>::new(0x1001, 0x4001).unwrap();
    assert_eq!(allocator.remaining(), 2);
    assert_eq!(allocator.allocate().unwrap().address(), 0x2000);
    assert_eq!(allocator.allocate().unwrap().address(), 0x3000);
    assert_eq!(allocator.allocate(), None);
}

#[test]
fn accepts_empty_and_subpage_regions() {
    for (start, end) in [(0, 0), (4096, 4096), (1, 4095), (17, 17)] {
        let mut allocator = BootFrameAllocator::<TEST_FRAME_SIZE>::new(start, end).unwrap();
        assert_eq!(allocator.remaining(), 0);
        assert_eq!(allocator.allocate(), None);
    }
}

#[test]
fn rejects_reversed_and_overflowing_regions() {
    assert!(matches!(
        BootFrameAllocator::<TEST_FRAME_SIZE>::new(2, 1),
        Err(RegionError::Reversed)
    ));
    assert!(matches!(
        BootFrameAllocator::<TEST_FRAME_SIZE>::new(usize::MAX - 1, usize::MAX),
        Err(RegionError::AlignmentOverflow)
    ));
}

#[test]
fn handles_top_of_address_space_without_wraparound() {
    fn check<const SIZE: usize>() {
        let end = usize::MAX & !(SIZE - 1);
        let mut allocator = BootFrameAllocator::<SIZE>::new(end - SIZE, usize::MAX).unwrap();
        assert_eq!(allocator.allocate().unwrap().address(), end - SIZE);
        assert_eq!(allocator.allocate(), None);
    }
    check::<256>();
    check::<4096>();
    check::<16384>();
    check::<65536>();
}

#[test]
fn supports_platform_sizes_and_reports_frame_extent() {
    fn check<const SIZE: usize>() {
        // Partial first/last frames must be excluded at every configured size.
        let mut allocator = BootFrameAllocator::<SIZE>::new(SIZE + 1, 5 * SIZE + 1).unwrap();
        assert_eq!(allocator.remaining(), 3);
        for index in 2..5 {
            let frame = allocator.allocate().unwrap();
            assert_eq!(frame.address(), index * SIZE);
            assert_eq!(frame.size(), SIZE);
        }
        assert_eq!(allocator.remaining(), 0);
        assert_eq!(allocator.allocate(), None);
    }
    check::<256>();
    check::<4096>();
    check::<16384>();
    check::<65536>();
}

#[test]
fn rejects_zero_and_non_power_of_two_frame_sizes() {
    fn check<const SIZE: usize>() {
        assert!(matches!(
            BootFrameAllocator::<SIZE>::new(0, usize::MAX),
            Err(RegionError::InvalidFrameSize)
        ));
    }
    check::<0>();
    check::<3>();
    check::<6144>();
    check::<{ usize::MAX }>();
}
