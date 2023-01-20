# rust-async-book

برنامه نویسی Async در Rust

## پیش نیاز ها

این کتاب با استفاده از [`mdbook`] ساخته شده است، شما میتونید این پکیج رو با cargo نصب کنید.

```
cargo install mdbook
cargo install mdbook-linkcheck
```

[`mdbook`]: https://github.com/rust-lang/mdBook

## ساخت نسخته قابل نمایش از کتاب

برای ساختن نسخه نهایی و قابل اجرای کتاب، دستور `mdbook build` رو اجرا کنید تا نسخه نهایی رو تو پوشه `/book` ببینید.

```
mdbook build
```

## Development

اگه میخواید کتاب توسط یک وب سرور آماده روی شبکه لوکالتون بارگزاری شه میتونید خیلی راحت از دستور `mdbook serve` استفاده کنید.

```
mdbook serve
```
