# نگاه دقیق به Future Trait

trait `Future` در Rust هسته مرکزیه برنامه نویسی Async محسوب میشه.
`Future` یک محاسبه Async هست که در نهایت یک مقداری رو تولید میکنه.(که البته ممکنه اون مقدار خالی باشه مثل `()`). یک مثال _آسون شده_ از trait `Future` میتونه این شکلی باشه:

```rust
{{#include ../../examples/02_02_future_trait/src/lib.rs:simple_future}}
```

Future ها میتونن با استفاده از صدا زدن تابع `poll` اجرا بشن، که این باعث میشه Future رو همینجور به سمتی سوق بده که بالاخره اجراش تکمیل بشه.
اگه Future تموم بشه مقدار `Poll::Ready(result)` رو برمیگردونه. اگه Future هنوز آماده نیست و تکمیل نشده باشه مقدار `Poll::Pending` رو برمیگردونه و هر وقت Future آماده باشه تا پردازش بیشتری رو جلو ببره تابع `wake()` رو صدا میزنه.
وقتی `wake()` صدا زده بشه، اون اجراکننده ای که داره `Future` رو انجام میده دوباره `poll` رو صدا میزنه تا `Future` بتونه پزدازششو جلو ببره.

بدون `wake()` اون اجراکننده هیچ اطلاعی نداره که یه Future خاص میتونه ادامه پردازششو انجام بده، و باید به صورت مداوم همه future ها رو poll بگیره. با تابع `wake()` اجراکننده دقیقا میدونه کدوم future آماده هست تا `poll` گرفته بشه.

برای مثال فرض کنیید قراره از یک socket دیتایی رو بخونیم که هنوز آماده نیست. اگه دیتا آماده باشه میتونیم بخونیمش و مقدار `Poll::Ready(data)` ولی اگه دیتایی آماده نباشه future ما بلاک و مسدود میشه و نمیتونم پردازش رو جلو ببریم. وقتی دیتایی آماده نباشه ما باید یک `wake` رو ثبت کنیم یا به اصطلاح register کنیم تا وقتی دیتا socket آماده بود اونو صدا بزنیم، که این کار باعث میشه به اجراکننده کدمون این پیغامو برسونه که future آماده ادامه پردازششه. یه مثال ساده از `SocketRead` میتونه چیزی شبیه به مثال زیر باشه:

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:socket_read}}
```

This model of `Future`s allows for composing together multiple asynchronous
operations without needing intermediate allocations. Running multiple futures
at once or chaining futures together can be implemented via allocation-free
state machines, like this:

این مدل از `Future` ها امکان ترکیب چندین عملیات async رو بدون نیاز به allocate کردن state های اضافی فراهم می کنند.
اجرا کردن چندین Future به صورت یکجا یا به صورت زنجیره ای از Future ها میتونه بدون نیاز به allocation های اضافی state های مربوطه به صورت زیر پیاده سازی بشه:

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:join}}
```

این نشون دهنده اینه که چندین Future میتونن به صورت همزمان اجرا بشن بدون نیاز به allocation های جداگانه، که همین باعث میشه برنامه های async با بازده بیشتر رو بتونیم بسازیم.
مشابه همین داستان، چندین Future میتونن به صورت خطی پشت سر هم اجرا بشن، مثل مثال زیر:

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:and_then}}
```

این مثال ها نشون میده چطوری trait `Future` میتونه بیان های مختلفی از جریان کنترلی روی یه برنامه async رو بدون نیاز به allocate کردن چندین object و callback های تو در تو ارائه بده.
با این تفاسیر بیاید راجب trait اصلی `Future` و تفاوت هاش صحبت کنیم:

```rust,ignore
{{#include ../../examples/02_02_future_trait/src/lib.rs:real_future}}
```

اولین چیزی که احتمالا متوجهش شدید اینه که `self` دیگه از نوع `&mut Self` نیست و تبدیل شده به `Pin<&mut Self>`.
ما راجب [Pin کردن][pinning] بیشتر در اینده صحبت میکنم، ولی فعلا در همین حد بدونید که Pin کردن این اجازه رو به ما میده تا Future های غیر قابل حرکت (ثابت) بسازیم. Object های ثابت یا غیر قابل حرکت میتونن Pointer مربوط به مقادیرشون رو بین همون مقادیر ذخیره کنن مثلا
`struct MyFut { a: i32, ptr_to_a: *const i32 }`. Pin کردن یه چیز واجب برای استفاده از async/await هست.

در ادامه میبینم که `wake: fn()` تبدیل شده به `&mut Context<'_>`. در `SimpleFuture` ما یک تابع pointer (`fn()`) رو صدا میزدیم تا به اون اجراکننده Future بگیم اون Future ای که نیاز داریم رو poll بگیره یا وضعیتشو چک کنه.
با این وجود، چون `fn()` فقط یه تابع pointer هست، نمیتونه هیچ اطلاعاتی راجب اینکه _کدوم_ `Future` تابع `wake` رو صدا زده ذخیره کنه.

توی یک نرم افزار واقعی که قراره ساخته بشه، یه برنامه پیچیده مثل یک وب سرور شاید هزاران کانکشن داره که اون کانکشن ها باید توابع wake up شون به صورت جداگانه همگی صدا زده بشه. تایپ `Context` این مشکل رو با استفاده از تایپ `Waker` و دسترسی به اون حل کرده، که در حقیقت میتونه باهاش یه task خاص و مشخص رو صدا بزنه و wake up اش کنه.

[pinning]: ../04_pinning/01_chapter.md
