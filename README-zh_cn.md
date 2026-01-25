# Error2

[English](README.md) | 简体中文

[![Crates.io version](https://img.shields.io/crates/v/error2.svg?style=flat-square)](https://crates.io/crates/error2)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/error2)

一个全面的 Rust 错误处理库，提供增强的错误回溯支持、错误链和便捷的错误转换功能。

## 安装

```toml
[dependencies]
error2 = "0.13.2"
```

## 用法

### 定义错误类型

Error2 根据字段结构支持三种类型的错误：

**1. 根错误** - 仅 `backtrace` 字段（新错误源）

```rust
use error2::prelude::*;

#[derive(Debug, Error2)]
#[error2(display("用户 {username} 未找到"))]
struct UserNotFound {
    username: String,
    backtrace: Backtrace,
}
```

**2. 标准错误** - `source` + `backtrace`（包装 `std::error::Error`）

```rust
use error2::prelude::*;

#[derive(Debug, Error2)]
#[error2(display("IO 错误: {path}"))]
struct IoError {
    path: String,
    source: std::io::Error,
    backtrace: Backtrace,
}
```

**3. Error2 链** - 仅 `source` 字段（链接另一个 `Error2` 类型，重用其回溯）

```rust
use error2::prelude::*;

#[derive(Debug, Error2)]
#[error2(display("应用程序错误"))]
struct AppError {
    source: IoError,  // 另一个 Error2 类型
}
```

### 错误转换

```rust
// Result<T, Source> -> Result<T, Target>
std::fs::read_to_string("file.txt")
    .context(IoError2 { path: "file.txt" })?;

// Option<T> -> Result<T, E>
database.get(id)
    .context(NotFound2 { item: "user" })?;

// E -> Result<T, E>
some_error.context(MyError2)
```

**注意：** 辅助结构体的字段是泛型且带有 `Into` trait 约束，所以不需要手动调用 `.into()`。对于开销较大的转换，使用 `.with_context()` 进行惰性求值：

```rust
// 惰性求值 - 仅在出错时才转换
std::fs::read(&path)
    .with_context(|| IoError2 { path: path.display().to_string() })?;
```

### 追踪传播

```rust
fn inner() -> Result<(), MyError> {
    operation().context(IoError2 { path: "file.txt" })?
}

fn outer() -> Result<(), MyError> {
    inner().attach()?  // 捕获此位置
}
```

### 创建根错误

```rust
fn validate(value: i32) -> Result<(), ValidationError> {
    if value < 0 {
        return ValidationError2 { value }.fail();
    }
    Ok(())
}
```

### 类型擦除错误

```rust
fn operation() -> Result<(), BoxedError2> {
    std::fs::read("file.txt").context(ViaStd)?;

    if condition {
        ViaRoot("错误消息").fail()?;
    }

    other_operation().context(ViaErr2)?;

    Ok(())
}
```

### 打印错误栈

```rust
fn main() {
    match operation().attach() {
        Ok(_) => println!("成功"),
        Err(e) => {
            // 打印完整的错误链及位置：
            //
            // IoError: io 错误: config.txt
            //     at src/main.rs:15:42
            //     at src/main.rs:23:5
            // std::io::Error: No such file or directory (os error 2)
            eprintln!("{}", e.backtrace().error_message());
        }
    }
}
```

更多详情请查看 [API 文档](https://docs.rs/error2)

## 许可证

可选以下任一许可证：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
