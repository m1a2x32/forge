//! Early physical-address allocation. No heap, virtual memory, or target assumptions.

#[derive(Debug, PartialEq, Eq)]
pub enum RegionError {
    InvalidFrameSize,
    Reversed,
    AlignmentOverflow,
}

/// A numeric physical-page descriptor. It does not grant permission to dereference
/// memory. FRAME_SIZE is the allocation size in bytes, chosen by the platform at
/// build time and carried in the type so differently sized frames cannot be mixed
/// implicitly. The platform remains responsible for ownership and accessibility.
#[derive(Debug, PartialEq, Eq)]
pub struct PhysicalFrame<const FRAME_SIZE: usize> {
    address: usize,
}

impl<const FRAME_SIZE: usize> PhysicalFrame<FRAME_SIZE> {
    pub fn address(&self) -> usize {
        self.address
    }

    pub fn size(&self) -> usize {
        FRAME_SIZE
    }
}

/// Single-owner, monotonic allocator over a reserved half-open physical interval.
/// Rounds inward to full FRAME_SIZE-byte frames. No free operation: suitable for bootstrap
/// tables and structures whose lifetime spans the whole boot session. It cannot
/// discover RAM or exclude reservations; its caller must supply a free interval.
#[derive(Debug)]
pub struct BootFrameAllocator<const FRAME_SIZE: usize> {
    // Physical byte addresses, not allocated frames. At exhaustion the cursor can
    // equal the exclusive end (or exceed it for an interval with no full frames).
    next_frame_address: usize,
    region_end_address: usize,
}

impl<const FRAME_SIZE: usize> BootFrameAllocator<FRAME_SIZE> {
    /// Accept a reserved [region_start_address, region_end_address) interval.
    /// FRAME_SIZE must be a nonzero power of two. This checks allocator geometry;
    /// a future MMU layer must separately validate supported translation granules.
    pub fn new(
        region_start_address: usize,
        region_end_address: usize,
    ) -> Result<Self, RegionError> {
        if !FRAME_SIZE.is_power_of_two() {
            return Err(RegionError::InvalidFrameSize);
        }
        if region_start_address > region_end_address {
            return Err(RegionError::Reversed);
        }
        let next_frame_address = region_start_address
            .checked_add(FRAME_SIZE - 1)
            .ok_or(RegionError::AlignmentOverflow)?
            & !(FRAME_SIZE - 1);
        let region_end_address = region_end_address & !(FRAME_SIZE - 1);
        Ok(Self {
            next_frame_address,
            region_end_address,
        })
    }

    pub fn allocate(&mut self) -> Option<PhysicalFrame<FRAME_SIZE>> {
        if self.next_frame_address >= self.region_end_address {
            return None;
        }
        let address = self.next_frame_address;
        // Aligned bounds and cursor < exclusive end imply addition cannot overflow.
        self.next_frame_address += FRAME_SIZE;
        Some(PhysicalFrame { address })
    }

    pub fn remaining(&self) -> usize {
        self.region_end_address
            .saturating_sub(self.next_frame_address)
            / FRAME_SIZE
    }
}

#[cfg(test)]
#[path = "memory_test.rs"]
mod tests;
