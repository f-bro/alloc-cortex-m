//! A heap allocator for Cortex-M processors
//!
//! # Example
//!
//! ```
//! // Plug in the allocator crate
//! extern crate alloc_cortex_m;
//! extern crate collections;
//!
//! use collections::Vec;
//!
//! // These symbols come from a linker script
//! extern "C" {
//!     static mut _heap_start: usize;
//!     static mut _heap_end: usize;
//! }
//!
//! #[no_mangle]
//! pub fn main() -> ! {
//!     // Initialize the heap BEFORE you use the allocator
//!     unsafe { alloc_cortex_m::init(&mut _heap_start, &mut _heap_end) }
//!
//!     let mut xs = Vec::new();
//!     xs.push(1);
//!     // ...
//! }
//! ```
//!
//! And in your linker script, you might have something like:
//!
//! ``` text
//! /* space reserved for the stack */
//! _stack_size = 0x1000;
//!
//! /* `.` is right after the .bss and .data sections */
//! _heap_start = .;
//! _heap_end = ORIGIN(SRAM) + LENGTH(SRAM) - _stack_size;
//! ```

#![feature(const_fn)]
#![no_std]
#![feature(alloc, allocator_api)]

extern crate cortex_m;
extern crate linked_list_allocator;
extern crate alloc;

use alloc::allocator::{Alloc, Layout, AllocErr};

use linked_list_allocator::Heap;
use cortex_m::interrupt::Mutex;

pub struct CortexMHeap {
    heap: Mutex<Heap>,
}

impl CortexMHeap {

    /// Crate a new UNINITIALIZED heap allocator
    ///
    /// You must initialize this heap using the
    /// [`init`](struct.CortexMHeap.html#method.init) method before using the allocator.
    pub unsafe fn empty() -> CortexMHeap {
        CortexMHeap {
            heap: Mutex::new(Heap::empty()),
        }
    }

    /// Initializes the heap
    ///
    /// This function must be called BEFORE you run any code that makes use of the
    /// allocator.
    ///
    /// `start_addr` is the address where the heap will be located.
    ///
    /// `end_addr` points to the end of the heap.
    ///
    /// Note that:
    ///
    /// - The heap grows "upwards", towards larger addresses. Thus `end_addr` must
    ///   be larger than `start_addr`
    ///
    /// - The size of the heap is `(end_addr as usize) - (start_addr as usize)`. The
    ///   allocator won't use the byte at `end_addr`.
    ///
    /// # Unsafety
    ///
    /// Obey these or Bad Stuff will happen.
    ///
    /// - This function must be called exactly ONCE.
    /// - `end_addr` > `start_addr`
    pub unsafe fn init(&self, start_addr: usize, end_addr: usize){
        let size = end_addr - start_addr;
        self.heap.lock(|heap| heap.init(start_addr, size));
    }
}

unsafe impl<'a> Alloc for &'a CortexMHeap {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        self.heap.lock(|heap| {
            heap.allocate_first_fit(layout)
        })
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.heap.lock(|heap| heap.deallocate(ptr, layout));
    }
}