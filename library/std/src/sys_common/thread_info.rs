#![allow(dead_code)] // stack_guard isn't used right now on all platforms
#![allow(unused_unsafe)] // thread_local with `const {}` triggers this liny

use crate::cell::{Cell, RefCell};
use crate::sys::thread::guard::Guard;
use crate::thread::{Thread, ThreadId};

struct ThreadInfo {
    stack_guard: Option<Guard>,
    thread: Thread,
}

thread_local! {
    static THREAD_INFO: RefCell<Option<ThreadInfo>> = const { RefCell::new(None) };
    static THREAD_ID: Cell<Option<ThreadId>> = const { Cell::new(None) };
}

impl ThreadInfo {
    fn with<R, F>(f: F) -> Option<R>
    where
        F: FnOnce(&mut ThreadInfo) -> R,
    {
        THREAD_INFO
            .try_with(move |thread_info| {
                let mut thread_info = thread_info.borrow_mut();
                let thread_info = thread_info.get_or_insert_with(|| ThreadInfo {
                    stack_guard: None,
                    thread: Thread::new(None),
                });
                f(thread_info)
            })
            .ok()
    }
}

#[inline]
pub(crate) fn current_thread_id() -> ThreadId {
    fn init(cache: &Cell<Option<ThreadId>>) -> ThreadId {
        let id = ThreadId::new();
        cache.set(Some(id));
        id
    }
    // The `with` here's panic branch should be DCEd on platforms with
    // `#[thread_local]`, as `THREAD_ID` is implemented to avoid needing
    // registration of a TLS destructor on these targets.
    //
    // That said, this is just best-effort and not a guarantee. If we have to
    // fallback to non-static TLS (e.g. `pthread_key_t`, `TlsAlloc`, ...),
    // there's we have no choice but to panic.
    THREAD_ID.with(|c: &Cell<Option<ThreadId>>| c.get().unwrap_or_else(|| init(c)))
}

pub fn current_thread() -> Option<Thread> {
    ThreadInfo::with(|info| info.thread.clone())
}

pub fn stack_guard() -> Option<Guard> {
    ThreadInfo::with(|info| info.stack_guard.clone()).and_then(|o| o)
}

pub fn set(stack_guard: Option<Guard>, thread: Thread) {
    let id = thread.id();
    THREAD_INFO.with(move |thread_info| {
        let mut thread_info = thread_info.borrow_mut();
        rtassert!(thread_info.is_none());
        *thread_info = Some(ThreadInfo { stack_guard, thread });
    });
    THREAD_ID.with(|c| {
        rtassert!(c.get().is_none() || c.get() == Some(id));
        c.set(Some(id));
    });
}
