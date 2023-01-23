#![allow(unused)]
#![cfg(test)]

mod async_fn_and_block_examples {
use std::future::Future;
// ANCHOR: async_fn_and_block_examples

// تابع زیر تایپی رو برمیگردونه که "تریت" زیر رو پیاده سازی کرده:
// `Future<Output = u8>`
// که باعث میشه در نهایت تایپ زیر برگرده:
// u8
async fn foo() -> u8 { 5 }

fn bar() -> impl Future<Output = u8> {
    // بلاک زیر تایپی رو برمیگردونه از نوع:
    // `Future<Output = u8>`
    async {
        let x: u8 = foo().await;
        x + 5
    }
}
// ANCHOR_END: async_fn_and_block_examples
}

mod async_lifetimes_examples {
use std::future::Future;
// ANCHOR: lifetimes_expanded
// این تابع:
async fn foo(x: &u8) -> u8 { *x }

// با این تابع برابره:
fn foo_expanded<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
    async move { *x }
}
// ANCHOR_END: lifetimes_expanded

async fn borrow_x(x: &u8) -> u8 { *x }

#[cfg(feature = "never_compiled")]
// ANCHOR: static_future_with_borrow
fn bad() -> impl Future<Output = u8> {
    let x = 5;
    borrow_x(&x) // ارور: مقدار "ایکس" زمان زیادی زنده نیست یعنی لایف تایمش اجازه نمیده
}

fn good() -> impl Future<Output = u8> {
    async {
        let x = 5;
        borrow_x(&x).await
    }
}
// ANCHOR_END: static_future_with_borrow
}

mod async_move_examples {
use std::future::Future;
// ANCHOR: async_move_examples
/// `async` block:
///
/// چندین بلاک از نوع زیر میتونن به متغیر های لوکال دسترسی داشته باشن
/// البته تا وقتی که تو اون اسکوپ اجرا بشن
async fn blocks() {
    let my_string = "foo".to_string();

    let future_one = async {
        // ...
        println!("{my_string}");
    };

    let future_two = async {
        // ...
        println!("{my_string}");
    };

    // هر دو "فیوچر" اجرا میشه تا در نهایت دو بار مقدار زیر تپرینت میشه:
    // "foo"
    let ((), ()) = futures::join!(future_one, future_two);
}

/// `async move` block:
///
/// فقط یک بلاکی که کلیدواژه "موو" داره میتونه به مقادیر لوکال در لحظه دسترسی داشته باشه
/// به خاطر اینکه اون مقادیر به اسکوپ "فیوچر" منتقل شدن
/// البته همین باعث میشه اون "فیوچر" بیشتر از اسکوپ اصلی زنده بمونه و دووم بیاره:
fn move_block() -> impl Future<Output = ()> {
    let my_string = "foo".to_string();
    async move {
        // ...
        println!("{my_string}");
    }
}
// ANCHOR_END: async_move_examples
}
