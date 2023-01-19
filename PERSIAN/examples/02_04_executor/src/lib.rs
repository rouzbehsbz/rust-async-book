#![cfg(test)]

// ANCHOR: imports
use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::{
    future::Future,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex},
    task::Context,
    time::Duration,
};
// اون تایمری که توی قسمت قبل با هم نوشتیم
use timer_future::TimerFuture;
// ANCHOR_END: imports

// ANCHOR: executor_decl
/// اجراکننده ای که تسک ها رو از یه کانال میگیره و اجراشون میکنه
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// این یه "فیوچر" جدید میسازه و میفرسته به کانال
#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// یک "فیوچر" ی که میتونه خودشو دوباره برنامه ریزی که که توسط اجراکننده دوباره "پول" بشه
struct Task {
    /// فیوچری که در حاله پردازشه و باید به داخل صف گذاشته بشه تا انجام بشه
    /// 
    /// اینجا استفاده از "میوتکس" اجباری نیست، به دلیل اینکه ما فقط از طریق یک "ترد" داریم به مقادریمون دسترسی
    /// پیدا میکنیم، اما "راست" اونقدر باهوش نیست که بفهمه ما فقط از یه "ترد" استفاده میکنیم
    /// برای همین مجبوریم از "میتوکس" استفاده کنیم تا قوانین امن بودن مموری در
    /// زبان "راست" رو رعایت کرده باشیم
    /// البته میشه از تایپ زیر هم به جای میتوکس استفاده کرد
    /// `UnsafeCell`
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// یک هندلر که تسک رو برنامه ریزی میکنه و برش میگردونه به داخل صف
    task_sender: SyncSender<Arc<Task>>,
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    // بیشترین حد تسک هایی که میتونیم داخل صف داشته باشیم از طریق کانال در یک لحظه
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}
// ANCHOR_END: executor_decl

// ANCHOR: spawn_fn
impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}
// ANCHOR_END: spawn_fn

// ANCHOR: arcwake_for_task
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // تابع "بیدار کننده" رو با فرستادن تسک توی کانال پیاده سازی میکنیم
        // تا دوباره توسط اجرا کننده "پول" بشه
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}
// ANCHOR_END: arcwake_for_task

// ANCHOR: executor_run
impl Executor {
    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            // فیوچر رو میگیره و اگه هنوز تکمیل نشده باشه "پول" اش میکنه
            // و سعی میکنه تکمیلش کنه
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // یک "بیدار شونده" از خود تسک میسازیم
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                // `BoxFuture<T>` این تایپ یه تایپ مستعار برای تایپ زیره
                // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                if future.as_mut().poll(context).is_pending() {
                    // ایمنجا هنوز تسک ما تکمیل نشده برای همین
                    // دوباره برش میگردونیم تا دوباره توسط اجراکننده اجرا بشه
                    *future_slot = Some(future);
                }
            }
        }
    }
}
// ANCHOR_END: executor_run

// ANCHOR: main
fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    //یک تسک برای پرینت کردن قبل و بعد از اتمام تایمر میسازیم
    spawner.spawn(async {
        println!("howdy!");
        // صبر میکنیم تا تایمر "فیوچر" ما بعد از 2 ثانیه تکمیل بشه
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
    });

    // حالا از حافظه پاکش میکنیم تا اجرا کننده بدونه تموم شده کارش
    // و تسک های بیشتری در آینده نمیگیره ازش تا اجراش کنه
    drop(spawner);

    // Run the executor until the task queue is empty.
    // This will print "howdy!", pause, and then print "done!".
    // اجرا کننده رو اجرا میکنه تا زمانی که صف تسک ها خالی بشه
    // که در نهایت برای ما اول پرینت میکنه:
    // "howdy!"
    // و بعدش پرینت میکنه
    // "done!"
    executor.run();
}
// ANCHOR_END: main

#[test]
fn run_main() {
    main()
}
