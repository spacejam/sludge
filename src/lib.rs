//! slow as lightning web thing [SLUDGE]
use std::{
    //cmp::Reverse,
    //collections::BinaryHeap,
    sync::{Arc, Condvar, Mutex},
    task::{
        Context, Poll, RawWaker, RawWakerVTable, Waker,
    },
    pin::Pin,
    future::Future,
    io::Result,
    net::TcpStream,
};

/// a web thing or something
pub struct Sludge;

#[derive(Default)]
struct Scheduler {
    readable: Vec<Box<dyn Future<Output = ()>>>,
    writable: Vec<Box<dyn Future<Output = ()>>>,
    acceptaz: Vec<Box<dyn Future<Output = ()>>>,
}

struct RioAccept<'a>(rio::Completion<'a, TcpStream>);

impl<'a> Future for RioAccept <'a>{
    type Output = Result<TcpStream>;

    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        Poll::Pending
    }
}

impl Sludge {
    pub async fn run() -> sled::Result<()> {
        let mut scheduler = Scheduler::default();
        let mut scheduler = unsafe {
            std::pin::Pin::new_unchecked(&mut scheduler)
        };

        let _rio = rio::new()?;
        let _db = sled::open("sludge_db")?;

        loop {
            let mut work = false;

            while let Some(_) = scheduler.writable.pop() {
                // write into socketz
                work = true;
            }
            while let Some(_) = scheduler.readable.pop() {
                // read off of socketz
                work = true;
            }

            if !work && scheduler.acceptaz.len() > 0 {
                // accept some more work
            }
        }
    }
}

#[derive(Default)]
struct Park(Mutex<bool>, Condvar);

fn unpark(park: &Park) {
    *park.0.lock().unwrap() = true;
    park.1.notify_one();
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |clone_me| unsafe {
        let arc = Arc::from_raw(clone_me as *const Park);
        std::mem::forget(arc.clone());
        RawWaker::new(
            Arc::into_raw(arc) as *const (),
            &VTABLE,
        )
    },
    |wake_me| unsafe {
        unpark(&Arc::from_raw(wake_me as *const Park))
    },
    |wake_by_ref_me| unsafe {
        unpark(&*(wake_by_ref_me as *const Park))
    },
    |drop_me| unsafe {
        drop(Arc::from_raw(drop_me as *const Park))
    },
);

/// Run a `Future`.
pub fn run<F: std::future::Future>(mut f: F) -> F::Output {
    let mut f =
        unsafe { std::pin::Pin::new_unchecked(&mut f) };
    let park = Arc::new(Park::default());
    let sender = Arc::into_raw(park.clone());
    let raw_waker =
        RawWaker::new(sender as *const _, &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    loop {
        match f.as_mut().poll(&mut cx) {
            Poll::Pending => {
                let mut runnable = park.0.lock().unwrap();
                while !*runnable {
                    runnable =
                        park.1.wait(runnable).unwrap();
                }
                *runnable = false;
            }
            Poll::Ready(val) => return val,
        }
    }
}
