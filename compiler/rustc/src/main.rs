// A note about jemalloc: rustc uses jemalloc when built for CI and
// distribution. The obvious way to do this is with the `#[global_allocator]`
// mechanism. However, for complicated reasons (see
// https://github.com/rust-lang/rust/pull/81782#issuecomment-784438001 for some
// details) that mechanism doesn't work here. Also, we must use a consistent
// allocator across the rustc <-> llvm boundary, and `#[global_allocator]`
// wouldn't provide that.
//
// Instead, we use a lower-level mechanism. rustc is linked with jemalloc in a
// way such that jemalloc's implementation of `malloc`, `free`, etc., override
// the libc allocator's implementation. This means that Rust's `System`
// allocator, which calls `libc::malloc()` et al., is actually calling into
// jemalloc.
//
// A consequence of not using `GlobalAlloc` (and the `tikv-jemallocator` crate
// provides an impl of that trait, which is called `Jemalloc`) is that we
// cannot use the sized deallocation APIs (`sdallocx`) that jemalloc provides.
// It's unclear how much performance is lost because of this.
//
// As for the symbol overrides in `main` below: we're pulling in a static copy
// of jemalloc. We need to actually reference its symbols for it to get linked.
// The two crates we link to here, `std` and `rustc_driver`, are both dynamic
// libraries. So we must reference jemalloc symbols one way or another, because
// this file is the only object code in the rustc executable.

fn main() {
    rustc_driver::alloc_hack();
    rustc_driver::set_sigpipe_handler();
    rustc_driver::main()
}
