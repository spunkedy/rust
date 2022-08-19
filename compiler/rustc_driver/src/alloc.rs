use std::os::raw::*;

#[used]
pub static _F1: unsafe extern "C" fn(usize, usize) -> *mut c_void = tikv_jemalloc_sys::calloc;
#[used]
pub static _F2: unsafe extern "C" fn(*mut *mut c_void, usize, usize) -> c_int =
    tikv_jemalloc_sys::posix_memalign;
#[used]
pub static _F3: unsafe extern "C" fn(usize, usize) -> *mut c_void =
    tikv_jemalloc_sys::aligned_alloc;
#[used]
pub static _F4: unsafe extern "C" fn(usize) -> *mut c_void = tikv_jemalloc_sys::malloc;
#[used]
pub static _F5: unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void =
    tikv_jemalloc_sys::realloc;
#[used]
pub static _F6: unsafe extern "C" fn(*mut c_void) = tikv_jemalloc_sys::free;
#[cfg(target_os = "macos")]
extern "C" {
    fn _rjem_je_zone_register();
}
// On OSX, jemalloc doesn't directly override malloc/free, but instead
// registers itself with the allocator's zone APIs in a ctor. However,
// the linker doesn't seem to consider ctors as "used" when statically
// linking, so we need to explicitly depend on the function.
#[used]
#[cfg(target_os = "macos")]
pub static _F7: unsafe extern "C" fn() = _rjem_je_zone_register;

pub fn ensure_jemalloc_symbols_are_used() {
    fn ensure_used<T: Copy + 'static>(x: &T) {
        let _ = unsafe { core::ptr::read_volatile(x) };
    }
    ensure_used(&_F1);
    ensure_used(&_F2);
    ensure_used(&_F3);
    ensure_used(&_F4);
    ensure_used(&_F5);
    ensure_used(&_F6);
    #[cfg(target_os = "macos")]
    ensure_used(&_F7);
}

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
