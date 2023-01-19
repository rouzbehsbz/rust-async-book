// ANCHOR: imports
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};
// ANCHOR_END: imports

// ANCHOR: timer_decl
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

/// وضعیت اشتراکی بین "فیوچر" و "ترد" ی که منتظر وایساده
struct SharedState {
    /// این مقدار نشون میده که تایمر کارش تموم شده یا نه
    completed: bool,

    /// تابع "بیدار شونده" برای این تایمر. که وقتی تایمر کارش تموم شد
    /// این تابع صدا زده بشه و ادامه عملیات انجام بشه
    waker: Option<Waker>,
}
// ANCHOR_END: timer_decl

// ANCHOR: future_for_timer
impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // بررسی کن میکنه ببینه تایمر کارش تموم شده یا نه
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // تابع "بیدار شونده" رو ست میکنیم تا ترد بتونه
            // در زمانی که تایمر کارش تموم میشه اون صدا بزنه
            //
            // اینجا فقط یک بار عملیات کپی کردن از تابع "بیدار شونده" انجام میشه به جای
            // اینکه هر سری بیاد و کپی کنه
            //
            // البته میشه از تابع زیر هم برای بررسی اینکه آیا اون "فیوچر" بیدار میشه یا نه
            // هم استفاده کرد ولی برای اینکه مثال رو ساده نگه داریم اینجوری استفاده کردیم
            // `Waker::will_wake`
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
// ANCHOR_END: future_for_timer

// ANCHOR: timer_new
impl TimerFuture {
    /// یک "فیوچر" جدید درست میکنه زمانی که تایمر کارش انجام میشه
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // یک ترد جدید میسازه
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            // زمانی که تایمر تموم شد یک سیگنال میفرسته و آخرین تسکی که "پول" شده رو بیدار میکنه
            // البته اگه تسکی موجود باشه
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });

        TimerFuture { shared_state }
    }
}
// ANCHOR_END: timer_new

#[test]
fn block_on_timer() {
    futures::executor::block_on(async {
        TimerFuture::new(Duration::from_secs(1)).await
    })
}
