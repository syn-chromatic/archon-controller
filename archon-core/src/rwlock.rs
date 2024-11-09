use core::cell::{RefCell, UnsafeCell};
use core::future::poll_fn;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

use embsys::crates::defmt;
use embsys::crates::embassy_sync;

use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::blocking_mutex::Mutex as BlockingMutex;
use embassy_sync::waitqueue::MultiWakerRegistration;

#[derive(defmt::Format, Debug)]
enum LockedState {
    Unlocked,
    ReadLocked(usize),
    WriteLocked,
}

struct State<const N: usize> {
    locked: LockedState,
    writer_pending: usize,
    waker: MultiWakerRegistration<N>,
}

pub struct RwLock<M, T, const N: usize>
where
    M: RawMutex,
    T: ?Sized,
{
    state: BlockingMutex<M, RefCell<State<N>>>,
    inner: UnsafeCell<T>,
}

impl<M, T, const N: usize> RwLock<M, T, N>
where
    M: RawMutex,
{
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
            state: BlockingMutex::new(RefCell::new(State {
                locked: LockedState::Unlocked,
                writer_pending: 0,
                waker: MultiWakerRegistration::new(),
            })),
        }
    }
}

impl<M, T, const N: usize> RwLock<M, T, N>
where
    M: RawMutex,
{
    pub async fn read(&self) -> RwLockReadGuard<'_, M, T, N> {
        poll_fn(|cx| {
            let ready = self.state.lock(|s| {
                let mut s = s.borrow_mut();
                if s.writer_pending > 0 {
                    return false;
                }
                match s.locked {
                    LockedState::Unlocked => {
                        s.locked = LockedState::ReadLocked(1);
                        true
                    }
                    LockedState::ReadLocked(n) => {
                        s.locked = LockedState::ReadLocked(n + 1);
                        true
                    }
                    LockedState::WriteLocked => {
                        s.waker.register(cx.waker()); // TODO could go wrong?
                        false
                    }
                }
            });

            if ready {
                Poll::Ready(RwLockReadGuard { rwlock: self })
            } else {
                Poll::Pending
            }
        })
        .await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, M, T, N> {
        self.state.lock(|s| {
            let mut s = s.borrow_mut();
            s.writer_pending += 1;
        });
        poll_fn(|cx| {
            let ready = self.state.lock(|s| {
                let mut s = s.borrow_mut();
                match s.locked {
                    LockedState::Unlocked => {
                        s.writer_pending -= 1;
                        s.locked = LockedState::WriteLocked;
                        true
                    }
                    _ => {
                        s.waker.register(cx.waker()); // TODO could go wrong?
                        false
                    }
                }
            });

            if ready {
                Poll::Ready(RwLockWriteGuard { rwlock: self })
            } else {
                Poll::Pending
            }
        })
        .await
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

pub struct RwLockReadGuard<'a, M, T, const N: usize>
where
    M: RawMutex,
    T: ?Sized,
{
    rwlock: &'a RwLock<M, T, N>,
}

impl<'a, M, T, const N: usize> Drop for RwLockReadGuard<'a, M, T, N>
where
    M: RawMutex,
    T: ?Sized,
{
    fn drop(&mut self) {
        self.rwlock.state.lock(|s| {
            let mut s = s.borrow_mut();
            match s.locked {
                LockedState::ReadLocked(n) => {
                    if n == 1 {
                        s.locked = LockedState::Unlocked;
                    } else {
                        s.locked = LockedState::ReadLocked(n - 1);
                    }
                }
                _ => panic!("invalid state"),
            };
            s.waker.wake();
        });
    }
}

impl<'a, M, T, const N: usize> Deref for RwLockReadGuard<'a, M, T, N>
where
    M: RawMutex,
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.rwlock.inner.get() as *const T) }
    }
}

pub struct RwLockWriteGuard<'a, M, T, const N: usize>
where
    M: RawMutex,
    T: ?Sized,
{
    rwlock: &'a RwLock<M, T, N>,
}

impl<'a, M, T, const N: usize> Drop for RwLockWriteGuard<'a, M, T, N>
where
    M: RawMutex,
    T: ?Sized,
{
    fn drop(&mut self) {
        self.rwlock.state.lock(|s| {
            let mut s = s.borrow_mut();
            match s.locked {
                LockedState::WriteLocked => {
                    s.locked = LockedState::Unlocked;
                }
                _ => panic!("invalid state"),
            };
            s.waker.wake();
        });
    }
}

impl<'a, M, T, const N: usize> Deref for RwLockWriteGuard<'a, M, T, N>
where
    M: RawMutex,
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.rwlock.inner.get() as *const T) }
    }
}

impl<'a, M, T, const N: usize> DerefMut for RwLockWriteGuard<'a, M, T, N>
where
    M: RawMutex,
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.rwlock.inner.get()) }
    }
}

unsafe impl<M, T, const N: usize> Sync for RwLock<M, T, N> where M: RawMutex {}
