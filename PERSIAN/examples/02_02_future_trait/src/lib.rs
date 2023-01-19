// ANCHOR: simple_future
trait SimpleFuture {
    type Output;
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
// ANCHOR_END: simple_future

struct Socket;
impl Socket {
    fn has_data_to_read(&self) -> bool {
        // چک کن اگر سوکت در حال حاضر آماده خوندن هست
        true
    }
    fn read_buf(&self) -> Vec<u8> {
        // دیتا رو از سوکت بخون
        vec![]
    }
    fn set_readable_callback(&self, _wake: fn()) {
        // اون تابع "_بیدار شدن" رو ثبت میکنه تا زمانی که
        // دیتایی از طرف سوکت قابل خوندن بود صدا زده بشه
        // که این دیتا میتونه از طرف اونت لوپ "ای پول" که برای لینوکس هست باشه.
    }
}

// ANCHOR: socket_read
pub struct SocketRead<'a> {
    socket: &'a Socket,
}

impl SimpleFuture for SocketRead<'_> {
    type Output = Vec<u8>;

    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if self.socket.has_data_to_read() {
            // سوکت دیتا رو آماده داره -- بخونش داخل یه بافر و برگردونش
            Poll::Ready(self.socket.read_buf())
        } else {
            // سوکت هنوز دیتایی نداره
            //
            // تابع "بیدار شدن" رو زمانی که دیتا آماده بود صدا بزن.
            // وقتی دیتا آماده باشه تابع "بیدار شدن" صدا زده میشه و در نتیجه
            // اون "فیوچر" میدونه که الان باید "پول" رو صدا بزنه دوباره و دیتا رو بگیره
            self.socket.set_readable_callback(wake);
            Poll::Pending
        }
    }
}
// ANCHOR_END: socket_read

// ANCHOR: join
/// یک "فیوچر ساده" که دو تا تابع "فیوچر" دیگه رو اجرا میکنه تا همزمان تموم بشن
/// 
/// همزمانی در اینجا با استفاده از صدا کردن "پول" روی تک تک "فیوچر" ها
/// انجام میشه، که این اجازه رو میده که اگر "فیوچر"ی خواست میتونه دیگه اجرا نشه و بقیه
/// اجرا بشن و در واقع هر "فیوچر"ی با سرعت خودش اجرا بشه
pub struct Join<FutureA, FutureB> {
    // هر کدوم از فیلد ها زیر ممکنه توشون "فیوچر" ی باشه که باید اجرا شه
    // اگر "فیوچرز" تکمیل شده باشه مقدار فیلد به "نان" تغییر پیدا میکنه
    // که این باعث میشه ما "فیوچر" هایی که تکمیل شدن رو دوباره اجرا نکنیم و "پول" نگیریم
    a: Option<FutureA>,
    b: Option<FutureB>,
}

impl<FutureA, FutureB> SimpleFuture for Join<FutureA, FutureB>
where
    FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        // سعی کن "فیوچر" اول رو انجلم بدی
        if let Some(a) = &mut self.a {
            if let Poll::Ready(()) = a.poll(wake) {
                self.a.take();
            }
        }

        // سعی کن "فیوچر" بعدی رو انجام بدی
        if let Some(b) = &mut self.b {
            if let Poll::Ready(()) = b.poll(wake) {
                self.b.take();
            }
        }

        if self.a.is_none() && self.b.is_none() {
            // هر دوی "فیوچر" ها با موفقیت تموم شدن -- حالا میتونیم با موفقیت نتیجه رو برگردونیم
            Poll::Ready(())
        } else {
            // یک یا هر دوی "فیوچر" ها آماده نیست و هنوز کار دارن تا تموم بشن
            // اونا تابع "بیدار شدن" رو زمانی که بتونن پردازشی رو جلو ببرن صدا میزنن
            Poll::Pending
        }
    }
}
// ANCHOR_END: join

// ANCHOR: and_then
/// یک "فیچوچر" دیگه که دو تا "فیوچر" رو انجام میده، یکی بعد از اونی یک و پشت سر هم
//
// نکته: برای اهداف این مثال هر دوی "فیوچر" ها در لحظه ساختن در دسترس هستن
// در واقعیت خروجی "فویچر" دوم میتونه به عنوان ورودی به "فیوچر" بعدئی منتقل شه
pub struct AndThenFut<FutureA, FutureB> {
    first: Option<FutureA>,
    second: FutureB,
}

impl<FutureA, FutureB> SimpleFuture for AndThenFut<FutureA, FutureB>
where
    FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if let Some(first) = &mut self.first {
            match first.poll(wake) {
                // ما اولین "فیوچر" رو انجام دادیم -- پاکش کن
                // و دومی رو شروع کن
                Poll::Ready(()) => self.first.take(),
                // ما هنوز نمیتونیم اولین "فیوچر" روو انجام بدیم
                Poll::Pending => return Poll::Pending,
            };
        }
        // حالا که اولین "فیوچر" تموم شده، سعی کن دومی رو انجام بدی
        self.second.poll(wake)
    }
}
// ANCHOR_END: and_then

mod real_future {
use std::{
    future::Future as RealFuture,
    pin::Pin,
    task::{Context, Poll},
};

// ANCHOR: real_future
trait Future {
    type Output;
    fn poll(
        // به تغییر تایپ این مقدار پایین دقیت کنید
        self: Pin<&mut Self>,
        // تایپ مقدار پایین هم عوض شده
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output>;
}
// ANCHOR_END: real_future

// مطمئن شو که "فیوچر" با "ریل فیوچر" یکی هست
impl<O> Future for dyn RealFuture<Output = O> {
    type Output = O;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        RealFuture::poll(self, cx)
    }
}
}
