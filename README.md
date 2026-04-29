# Rust Pattern Components

[![Crates.io](https://img.shields.io/crates/v/rust-pattern-components.svg)](https://crates.io/crates/rust-pattern-components)
[![Documentation](https://docs.rs/rust-pattern-components/badge.svg)](https://docs.rs/rust-pattern-components)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个高质量的 Rust 设计模式组件库，提供构建器、工厂、观察者等常用设计模式的现代化实现。

## 特性

- **零成本抽象**：利用 Rust 的类型系统和编译时优化
- **线程安全**：所有组件都设计为线程安全，支持并发使用
- **弱引用管理**：观察者模式使用弱引用避免循环引用和内存泄漏
- **编译时注册**：工厂模式通过 `inventory` crate 在编译时注册工厂
- **丰富的错误处理**：完整的错误类型和回退策略

## 安装

```toml
[dependencies]
rust-pattern-components = "0.1"
```

如果还需要过程宏（`#[simple_factory]`、`#[observable]`），可以使用统一的 `rust-patterns` crate：

```toml
[dependencies]
rust-patterns = "0.1"
```

## 包含的设计模式

### 1. 构建器模式 (Builder)

通过 `builder_helper!` 宏为构建器类型添加条件设置方法 `when_some`，避免大量 `if let Some(v) = x {}` 样板代码。

#### 功能特点

- 支持 `Self`（消费型）和 `&mut Self`（可变引用型）两种方式
- 可为多个构建器类型一次性实现
- 类型安全的链式调用
- 当值为 `None` 时自动跳过，不调用闭包

#### 使用方式

```rust
use rust_patterns_components::builder_helper;

// Self 版本（消费型构建器）
builder_helper!(Self, MyBuilder);

// &mut Self 版本（可变引用构建器）
builder_helper!(&mut Self, MyMutBuilder);

// 同时为多个类型实现
builder_helper!(Self, Builder1, Builder2);
```

#### 示例

```rust
use rust_patterns_components::builder_helper;

struct UserBuilder {
    name: Option<String>,
    age: Option<u32>,
}

impl UserBuilder {
    fn new() -> Self {
        Self { name: None, age: None }
    }

    fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }

    fn build(self) -> String {
        format!("{} ({})", self.name.unwrap_or_default(), self.age.unwrap_or(0))
    }
}

builder_helper!(Self, UserBuilder);

fn main() {
    let user = UserBuilder::new()
        .name("Alice")
        .when_some(Some(30), |b, age| b.age(age))
        // None 值会被自动跳过
        .when_some(None::<u32>, |b, age| b.age(age))
        .build();

    assert_eq!(user, "Alice (30)");
}
```

### 2. 工厂模式 (Factory)

类型安全的工厂系统，支持编译时工厂注册、多种回退策略和完整的错误处理。

#### 核心组件

| 组件 | 说明 |
|------|------|
| `Factory<T>` | 工厂 trait，定义 `create` 方法 |
| `SimpleFactory<T>` | 工厂集合，按 ID 查找并创建对象 |
| `FactoryRegistry<T>` | 工厂注册表，配合 `inventory` 在编译时注册工厂 |
| `FactoryFallback` | 回退策略枚举（`First` / `Last` / `NoFallback`） |
| `FactoryError` | 错误类型（`FactoryNotFound` / `EmptyIdNoFallback` / `NoFactoriesAvailable`） |
| `register_factory!` | 注册工厂的宏 |

#### 示例

```rust
use rust_pattern_components::{FactoryFallback, FactoryRegistry, FactoryError, register_factory};

// 定义产品 trait
trait Product: Send + Sync {
    fn name(&self) -> &str;
}

// 产品实现 A
struct ProductA;
impl Product for ProductA {
    fn name(&self) -> &str { "ProductA" }
}
impl Default for ProductA {
    fn default() -> Self { Self }
}
impl Factory<Product> for ProductA {
    fn create(&self) -> Box<Product> {
        Box::new(ProductA)
    }
}

// 产品实现 B
struct ProductB;
impl Product for ProductB {
    fn name(&self) -> &str { "ProductB" }
}
impl Default for ProductB {
    fn default() -> Self { Self }
}
impl Factory<Product> for ProductB {
    fn create(&self) -> Box<Product> {
        Box::new(ProductB)
    }
}

// 编译时注册工厂
register_factory!(dyn Product, "product_a", ProductA);
register_factory!(dyn Product, "product_b", ProductB);

fn main() {
    // 获取工厂实例（通过 inventory 自动收集已注册的工厂）
    let factory = FactoryRegistry::<dyn Product>::simple_factory();

    // 通过 ID 创建产品
    match factory.create("product_a", FactoryFallback::NoFallback) {
        Ok(product) => println!("创建产品: {}", product.name()),
        Err(FactoryError::FactoryNotFound(id)) => {
            println!("未找到 ID 为 '{}' 的工厂", id);
        }
        Err(e) => println!("创建失败: {}", e),
    }

    // 空 ID + First 回退策略 → 使用第一个可用的工厂
    match factory.create("", FactoryFallback::First) {
        Ok(product) => println!("回退创建产品: {}", product.name()),
        Err(FactoryError::NoFactoriesAvailable) => {
            println!("没有可用的工厂");
        }
        _ => {}
    }
}
```

#### 回退策略

| 策略 | 空 ID 时行为 |
|------|-------------|
| `First` | 使用集合中的第一个工厂 |
| `Last` | 使用集合中的最后一个工厂 |
| `NoFallback` | 直接返回 `EmptyIdNoFallback` 错误 |

### 3. 观察者模式 (Observer)

线程安全的观察者模式实现，使用弱引用避免循环引用。

#### 核心组件

| 组件 | 说明 |
|------|------|
| `Observer` trait | 观察者接口，定义 `update` 方法 |
| `Observable` trait | 被观察者接口，定义 `attach` / `detach` 方法 |
| `ObserverRegistry<T>` | 观察者注册表，管理弱引用列表，提供 `notify` 和 `notify_ignore_error` 方法 |

#### 设计特点

- **弱引用管理**: 使用 `Weak<dyn Observer>` 存储观察者，避免强引用循环
- **自动清理**: 通知时自动跳过已释放的观察者
- **防止重复**: 通过 `Weak::ptr_eq` 检查防止重复添加
- **两种通知策略**:
  - `notify`: 遇到错误立即停止通知，并返回错误
  - `notify_ignore_error`: 忽略所有错误，继续通知所有观察者

#### 示例

```rust
use std::sync::Arc;
use rust_pattern_components::{Observer, Observable, ObserverRegistry};

// 被观察者：温度传感器
struct TemperatureSensor {
    registry: ObserverRegistry<Self>,
    temperature: f64,
}

impl TemperatureSensor {
    fn new() -> Self {
        Self {
            registry: ObserverRegistry::new(),
            temperature: 20.0,
        }
    }

    fn set_temperature(&mut self, temp: f64) {
        self.temperature = temp;
        // 通知所有观察者
        let _ = self.registry.notify(&self.temperature);
    }
}

impl Observable for TemperatureSensor {
    type State = f64;
    type Error = String;

    fn attach(&mut self, observer: Arc<dyn Observer<Subject = Self>>) {
        self.registry.attach(observer);
    }

    fn detach(&mut self, observer: Arc<dyn Observer<Subject = Self>>) {
        self.registry.detach(observer);
    }
}

// 观察者：温度显示器
struct TemperatureDisplay;

impl Observer for TemperatureDisplay {
    type Subject = TemperatureSensor;

    fn update(&self, state: &f64) -> Result<(), String> {
        println!("当前温度: {:.1}°C", state);
        Ok(())
    }
}

fn main() {
    let mut sensor = TemperatureSensor::new();
    let display = Arc::new(TemperatureDisplay);

    sensor.attach(display);
    sensor.set_temperature(25.5); // 输出: 当前温度: 25.5°C
}
```

## 过程宏（需 `rust-patterns` crate）

`rust-pattern-macros` 提供了两个过程宏，可通过 `rust-patterns` crate 统一使用。

### `#[simple_factory]`

自动为 trait 生成工厂结构体和 `create` 方法：

```rust,ignore
use rust_patterns::simple_factory;

#[simple_factory]
pub trait MyTrait {
    fn do_something(&self);
}

// 宏会生成：
// pub struct MyTraitFactory;
// impl MyTraitFactory {
//     pub fn create(id: &str, strategy: FactoryFallback)
//         -> Result<(&str, Box<dyn MyTrait>), FactoryError>
// }
```

### `#[observable]`

自动为结构体添加 `ObserverRegistry` 字段并实现 `Observable` trait：

```rust,ignore
use rust_patterns::observable;

#[observable(state = u64, error = String)]
struct Counter {
    value: u64,
}

// 宏会添加 registry 字段，实现 Observable trait，
// 并提供 notify 和 notify_ignore_error 方法。
```

## API 文档

完整的 API 文档可在 [docs.rs](https://docs.rs/rust-pattern-components) 查看。

## 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test -- factory
cargo test -- builder
cargo test -- observer
```

## 运行方式

此库提供 `rust-pattern-components`（核心组件）和 `rust-patterns`（组件 + 过程宏）两种使用方式。

建议在大多数情况下使用 `rust-patterns`，因为它同时提供了运行时代码和过程宏，使用更加统一方便。

## 许可证

本项目基于 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 作者

- **Linshan Yang** - yanglsh@yeah.net

## 相关项目

- [rust-design-patterns](https://github.com/rust-unofficial/patterns) - Rust 设计模式集合
- [inventory](https://crates.io/crates/inventory) - 编译时注册系统
- [thiserror](https://crates.io/crates/thiserror) - 错误处理库

---

**提示**: 本项目正在积极开发中，API 可能会发生变化。建议在生产环境中使用时锁定特定版本。