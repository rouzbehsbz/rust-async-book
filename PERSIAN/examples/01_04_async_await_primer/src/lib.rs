#![cfg(test)]

use futures::executor::block_on;

mod first {
// ANCHOR: hello_world
// تابع "بلاک آن" تردی که در آن اجرا می شود را بلاک و مسدود میکند
// و وقتی آزاد می شود که اون "فیوچر"ی که در حال اجرای آن است به پایان برسد
// اجرا کننده های دیگه رفتار های پیچیده تری از خودشون نشون میدن
// مثل زمان بندی برای اجرای چند فیوچر در یک ترد
use futures::executor::block_on;

async fn hello_world() {
    println!("hello, world!");
}

fn main() {
    let future = hello_world(); // اینجا هیچی پرینت نمیشه
    block_on(future); // تابع "فیوچر" اجرا شد و متن بالا پرینت شد
}
// ANCHOR_END: hello_world

#[test]
fn run_main() { main() }
}

struct Song;
async fn learn_song() -> Song { Song }
async fn sing_song(_: Song) {}
async fn dance() {}

mod second {
use super::*;
// ANCHOR: block_on_each
fn main() {
    let song = block_on(learn_song());
    block_on(sing_song(song));
    block_on(dance());
}
// ANCHOR_END: block_on_each

#[test]
fn run_main() { main() }
}

mod third {
use super::*;
// ANCHOR: block_on_main
async fn learn_and_sing() {
    // اینجا اول صبر میکنیم آهنگ یاد گرفته بشه قبل از خوندنش
    // انیجا ما از "اویت" به جای "بلاک آن" استفاده میکنیم تا جلوگیری
    // بشه از بلاک شدن و مسدود شدن ترد اینجوری میتونیم همزمان با این کار
    // تابع رقصیدن هم اجرا کنیم
    let song = learn_song().await;
    sing_song(song).await;
}

async fn async_main() {
    let f1 = learn_and_sing();
    let f2 = dance();

    // ماکرو "جوین" مثل "اویت" هست. با استفاده از این ماکرو میشه
    // برای تموم شدن چندین "فیوچر" به صورت همزمان صبر کرد.
    // اگه به صورت موقت تابع اولی ترد رو بخواد مسدود کنه کار تابع بعدی یعنی رقصیدن انجام میشه.
    // و برعکسش هم میتونه اتفاق بیفته، یعنی اگه رقصیدن بخواد مسدود بشه اون یکی تابع اجرا میشه
    // و اگه جفتشون بلاک بشن این فانکشن کلا بلاک میشه و اجراکنندش صبر میکنه تموم بشه کارش
    futures::join!(f1, f2);
}

fn main() {
    block_on(async_main());
}
// ANCHOR_END: block_on_main

#[test]
fn run_main() { main() }
}
