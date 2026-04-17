# Rust Pattern Components

[![Crates.io](https://img.shields.io/crates/v/rust-pattern-components.svg)](https://crates.io/crates/rust-pattern-components)
[![Documentation](https://docs.rs/rust-pattern-components/badge.svg)](https://docs.rs/rust-pattern-components)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/sdht-siset/rust-pattern-components/actions/workflows/rust.yml/badge.svg)](https://github.com/sdht-siset/rust-pattern-components/actions)

一个高质量的 Rust 设计模式组件库，提供了构建器、工厂、观察者等常用设计模式的现代化实现。

## 特性

- **零成本抽象**：利用 Rust 的类型系统和编译时优化
- **线程安全**：所有组件都设计为线程安全，支持并发使用
- **易于使用**：简洁的 API 设计，提供丰富的文档和示例
- **生产就绪**：经过充分测试，包含单元测试和文档测试
- **无依赖**：核心功能不依赖外部 crate（除了必要的工具库）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
rust-pattern-components = "0.1.2"
```

## 包含的设计模式

### 1. 构建器模式 (Builder)

提供 `builder_helper!` 宏，为构建器类型添加条件设置方法。

#### 功能特点
- 支持 `Self` 和 `&mut Self` 两种实现方式
- 类型安全的链式调用
- 条件性设置可选值

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
}

// 为 UserBuilder 添加 when_some 方法
builder_helper!(Self, UserBuilder);

fn main() {
    let optional_name = Some("Alice");
    let optional_age = None::<u32>;
    
    let builder = UserBuilder::new()
        .when_some(optional_name, |b, name| b.name(name))
        .when_some(optional_age, |b, age| b.age(age));
}
```

### 2. 工厂模式 (Factory)

提供类型安全的工厂系统，支持运行时工厂注册和创建。

#### 功能特点
- 编译时工厂注册（使用 `inventory` crate）
- 支持 trait 对象的工厂创建
- 多种回退策略
- 完整的错误处理

#### 核心组件
- `Factory<T>`：工厂 trait
- `SimpleFactory<T>`：工厂集合
- `FactoryRegistry<T>`：工厂注册表
- `FactoryFallback`：回退策略枚举
- `FactoryError`：错误类型

#### 示例

```rust
use rust_patterns_components::{FactoryFallback, FactoryRegistry};

// 定义产品 trait
trait Product {
    fn name(&self) -> &str;
}

// 产品实现
struct ProductA;
impl Product for ProductA {
    fn name(&self) -> &str { "ProductA" }
}
impl Default for ProductA {
    fn default() -> Self { Self }
}

struct ProductB;
impl Product for ProductB {
    fn name(&self) -> &str { "ProductB" }
}
impl Default for ProductB {
    fn default() -> Self { Self }
}

// 注册工厂（通常在单独的模块中）
// register_factory!(dyn Product, "product_a", ProductA);
// register_factory!(dyn Product, "product_b", ProductB);

fn main() {
    let factory = FactoryRegistry::<dyn Product>::simple_factory();
    
    // 通过 ID 创建产品
    match factory.create("product_a", FactoryFallback::NoFallback) {
        Ok((id, product)) => {
            println!("创建了产品: {}, ID: {}", product.name(), id);
        }
        Err(e) => {
            println!("创建失败: {}", e);
        }
    }
    
    // 使用回退策略
    let result = factory.create("", FactoryFallback::First);
    // 当 ID 为空时，使用第一个可用的工厂
}
```

### 3. 观察者模式 (Observer)

提供线程安全的观察者模式实现，支持弱引用避免循环引用。

#### 功能特点
- 线程安全的观察者管理
- 使用弱引用避免内存泄漏
- 支持错误传播策略
- 类型安全的观察者注册

#### 核心组件
- `Observer<Subject>`：观察者 trait
- `Observable`：被观察者 trait
- `ObserverRegistry`：观察者注册表


#### 示例

```rust
use std::sync::Arc;
use rust_patterns_components::{Observer, Observable, ObserverRegistry};

// 定义被观察者
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
        self.registry.notify(&self.temperature).unwrap();
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

// 定义观察者
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
    
    sensor.attach(display.clone());
    sensor.set_temperature(25.5); // 输出: 当前温度: 25.5°C
}
```

## API 文档

完整的 API 文档可在 [docs.rs](https://docs.rs/rust-pattern-components) 查看。

## 运行测试

```bash
# 运行所有测试
cargo test

# 运行文档测试
cargo test --doc

# 运行特定模块的测试
cargo test -- factory
```

## 贡献指南

欢迎贡献！请遵循以下步骤：

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开 Pull Request

## 许可证

本项目基于 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 作者

- **Linshan Yang** - [yanglsh@yeah.net](mailto:yanglsh@yeah.net)

## 致谢

- 感谢所有贡献者和用户
- 灵感来源于经典的设计模式书籍和 Rust 社区的最佳实践

## 版本信息

查看 [crates.io](https://crates.io/crates/rust-pattern-components) 获取最新版本，或访问 [GitHub Releases](https://github.com/sdht-siset/rust-pattern-components/releases) 查看版本历史。

## 相关项目

- [rust-design-patterns](https://github.com/rust-unofficial/patterns) - Rust 设计模式集合
- [inventory](https://crates.io/crates/inventory) - 编译时注册系统
- [thiserror](https://crates.io/crates/thiserror) - 错误处理库

---

**提示**: 本项目正在积极开发中，API 可能会发生变化。建议在生产环境中使用时锁定特定版本。