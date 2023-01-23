# `async`/`.await`

توی [فصل اول] نگاه کوتاهی به ‍‍`async`/`.await` داشتیم.
توی این فصل با جزئیات بیشتری روی `async`/`.await` بحث میکنیم و میبینم چجوری کد async با کد عادی که قبلا تو Rust میدیدیم فرق داره.

`async`/`.await` کلیدواژه های ویژه ای هستند که قدرت برنامه نویسی async رو به Rust میدن و اجازه میدن قدرت به thread برگرده و عملیات بعدی رو انجام بده در حالی که منتظر جواب یه کد async هست.

دو راه اصلی برای استفاده از `async` وجود داره: یکی `async fn` و اون یکی بلاک های `async` هستن.
که هر کدوم مقداری رو بر میگردونن که trait ‍‍`‍‍Future` رو پیاده سازی کرده.

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:async_fn_and_block_examples}}
```

همونطور که توی فصل اول دیدیم بدنه `async` و بقیه Future ها تنبل هستن:
به این معنی که تا زمانی که اجرا نشن هیچ کاری نمیکنن. معمول ترین روش اجرا کردن `Future` اینه که منتظرش بمونید یا `.await` اش کنید.
زمانی که `.await` روی یه `Future` صدا زده میشه سعی میکنه اونو اجراش کنه تا با موفقیت تموم بشه. اگه `Future` بلاک کننده و مسدود کندده thread باشه کنترل رو به thread بر میگردونه. وقتی پردازش بیشتری میتونه انجام بشه `Future` دوباره توسط اجرا کننده برداشته میشه و ادامه پردازشش انجام میشه و این باعث میشه `.await` با موفقیت تکمیل بشه.

## Lifetime های `async`

برعکس توابع معمول در Rust، توابع `async fn` ای که رفرنس ها یا آرگومان های غیر از `static` رو به عنوان ورودی میگیرن، `Future` ای بر میگردونن که lifetime شون دقیقا مثل lifetime آرگومان هاشون هست:

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:lifetimes_expanded}}
```

که یعنی تابع `async fn` ای که یک Future بر میگردونه باید توسط `.await` صدا زده بشه در حالی که هنوز آرگومان های غیر `static` اش موجود هستن.
تو حالت عادی موقعی که `.await` میکنید بعد از اینکه تابع رو صدا میزنید مثلا مثل `foo(&x).await` موردی به موجود نمیاد و مشکلی نیست.
با این حال، ذخیره کردن Future ها یا ارسالشون به یک task یا thread دیگه ممکنه مشکل ساز بشه.

One common workaround for turning an `async fn` with references-as-arguments
into a `'static` future is to bundle the arguments with the call to the
`async fn` inside an `async` block:

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:static_future_with_borrow}}
```

By moving the argument into the `async` block, we extend its lifetime to match
that of the `Future` returned from the call to `good`.

## `async move`

`async` blocks and closures allow the `move` keyword, much like normal
closures. An `async move` block will take ownership of the variables it
references, allowing it to outlive the current scope, but giving up the ability
to share those variables with other code:

```rust,edition2018,ignore
{{#include ../../examples/03_01_async_await/src/lib.rs:async_move_examples}}
```

## `.await`ing on a Multithreaded Executor

Note that, when using a multithreaded `Future` executor, a `Future` may move
between threads, so any variables used in `async` bodies must be able to travel
between threads, as any `.await` can potentially result in a switch to a new
thread.

This means that it is not safe to use `Rc`, `&RefCell` or any other types
that don't implement the `Send` trait, including references to types that don't
implement the `Sync` trait.

(Caveat: it is possible to use these types as long as they aren't in scope
during a call to `.await`.)

Similarly, it isn't a good idea to hold a traditional non-futures-aware lock
across an `.await`, as it can cause the threadpool to lock up: one task could
take out a lock, `.await` and yield to the executor, allowing another task to
attempt to take the lock and cause a deadlock. To avoid this, use the `Mutex`
in `futures::lock` rather than the one from `std::sync`.

[فصل اول]: ../01_getting_started/04_async_await_primer.md
