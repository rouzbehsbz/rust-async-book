#![cfg(test)]

use futures::{executor::block_on, join};
use std::thread;

fn download(_url: &str) {
    // ...
}

#[test]
// ANCHOR: get_two_sites
fn get_two_sites() {
    // ایجاد دو تا thread جدید که کار دانلود رو انجام بده
    let thread_one = thread::spawn(|| download("https://www.foo.com"));
    let thread_two = thread::spawn(|| download("https://www.bar.com"));

    // صبر میکنیم تا هر دو تا thread کارشون تموم شه
    thread_one.join().expect("thread one panicked");
    thread_two.join().expect("thread two panicked");
}
// ANCHOR_END: get_two_sites

async fn download_async(_url: &str) {
    // ...
}

// ANCHOR: get_two_sites_async
async fn get_two_sites_async() {
    // ایجاد دو تا Future جدید که پس از تموم شدن کارشون
    // صفحات وب رو به صورت ناهمزمان دانلود میکنن
    let future_one = download_async("https://www.foo.com");
    let future_two = download_async("https://www.bar.com");

    // هر دو Future رو اجرا میکنیم که در یک زمان کارشون تموم شه
    join!(future_one, future_two);
}
// ANCHOR_END: get_two_sites_async

#[test]
fn get_two_sites_async_test() {
    block_on(get_two_sites_async());
}
