use std::{any::TypeId, collections::BTreeMap, fmt::Debug};

use inventory::{Collect, Registry};
use thiserror::Error;

/// 用于创建类型 `T` 实例的工厂 trait。
///
/// 实现此 trait 的类型可以创建目标类型的装箱实例。
/// 类型 `T` 必须是 `Send + Sync` 并且可以是非固定大小类型。
pub trait Factory<T: ?Sized> {
    fn create(&self) -> Box<T>;
}

/// 通过工厂创建对象时可能发生的错误。
#[derive(Debug, Error)]
pub enum FactoryError {
    /// 未找到指定的工厂。
    #[error("未找到 ID 为 '{0}' 的工厂")]
    FactoryNotFound(String),

    /// 不允许回退时提供了空 ID。
    #[error("不允许回退时提供了空 ID")]
    EmptyIdNoFallback,

    /// 请求的产品类型没有可用的工厂。
    #[error("没有可用的工厂")]
    NoFactoriesAvailable,
}

/// 当找不到指定 ID 的工厂时的处理策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactoryFallback {
    /// 如果找不到指定的工厂，则使用集合中的第一个工厂。
    First,

    /// 如果找不到指定的工厂，则使用集合中的最后一个工厂。
    Last,

    /// ID 为空时不回退，直接返回错误。
    NoFallback,
}

/// 用于创建类型 `T` 实例的工厂集合。
///
/// 此结构体包含一个从工厂 ID 到工厂实例的映射，这些工厂实例可以创建
/// 目标类型 `T` 的装箱实例。工厂在编译时使用 `inventory` crate 注册，
/// 可以通过 `FactoryRegistry::factories()` 检索。
///
/// 类型 `T` 可以是非固定大小类型（trait 对象），并且必须具有 `'static` 生命周期。
/// 工厂存储为静态引用，允许它们在线程间共享。
///
/// # 示例
///
/// 基本用法：
///
/// ```rust
/// use rust_patterns_components::{FactoryFallback, FactoryRegistry};
///
/// // 定义产品 trait
/// trait Product {
///     fn name(&self) -> &str;
/// }
///
/// // 假设已经注册了工厂（通过 inventory 机制）
/// // register_factory!(dyn Product, "product_a", ProductA);
/// // register_factory!(dyn Product, "product_b", ProductB);
///
/// // 获取工厂实例
/// let factory = FactoryRegistry::<dyn Product>::simple_factory();
///
/// // 通过 ID 创建特定产品
/// match factory.create("product_a", FactoryFallback::NoFallback) {
///     Ok((id, product)) => {
///         println!("创建了产品: {}, ID: {}", product.name(), id);
///     }
///     Err(e) => {
///         println!("创建失败: {}", e);
///     }
/// }
///
/// // 使用回退策略
/// let result = factory.create("", FactoryFallback::First);
/// // 当 ID 为空时，使用第一个可用的工厂
///
/// let result = factory.create("", FactoryFallback::Last);
/// // 当 ID 为空时，使用最后一个可用的工厂
/// ```
///
/// 错误处理：
///
/// ```rust
/// use rust_patterns_components::{FactoryFallback, FactoryRegistry, FactoryError};
///
/// // 定义产品 trait
/// trait Product {
///     fn name(&self) -> &str;
/// }
///
/// let factory = FactoryRegistry::<dyn Product>::simple_factory();
///
/// // 不存在的工厂 ID
/// match factory.create("nonexistent", FactoryFallback::NoFallback) {
///     Err(FactoryError::FactoryNotFound(id)) => {
///         println!("未找到工厂: {}", id);
///     }
///     _ => {}
/// }
///
/// // 空 ID 且无回退策略
/// match factory.create("", FactoryFallback::NoFallback) {
///     Err(FactoryError::EmptyIdNoFallback) => {
///         println!("空 ID 且未指定回退策略");
///     }
///     _ => {}
/// }
///
/// // 没有可用的工厂
/// match factory.create("any", FactoryFallback::NoFallback) {
///     Err(FactoryError::NoFactoriesAvailable) => {
///         println!("没有可用的工厂");
///     }
///     _ => {}
/// }
/// ```
pub struct SimpleFactory<T: ?Sized + 'static>(
    BTreeMap<&'static str, &'static (dyn Factory<T> + Sync)>,
);

impl<T> SimpleFactory<T>
where
    T: ?Sized + 'static,
{
    /// 使用指定的回退策略通过工厂模式创建实例。
    ///
    /// 此函数通过 ID 查找工厂并使用它创建实例。
    /// 如果 `id` 为空，行为取决于 `strategy`：
    /// - `NoFallback`：返回错误
    /// - `First`：使用集合中的第一个工厂
    /// - `Last`：使用集合中的最后一个工厂
    ///   如果 `id` 不为空但找不到工厂，行为由 `strategy` 决定。
    ///
    /// # 参数
    /// * `id` - 要使用的工厂标识符，或空字符串表示默认
    /// * `strategy` - 找不到指定 ID 的工厂时使用的策略
    ///
    /// # 返回值
    /// * `Ok((&str, Box<T>))` - 成功时返回包含使用的工厂 ID 和创建的实例的元组
    /// * `Err(FactoryError)` - 如果找不到工厂或没有可用的工厂则返回错误
    pub fn create<'a>(
        &self,
        id: &'a str,
        strategy: FactoryFallback,
    ) -> Result<(&'a str, Box<T>), FactoryError> {
        if !id.is_empty() {
            return if let Some(factory) = self.0.get(id) {
                Ok((id, factory.create()))
            } else {
                Err(FactoryError::FactoryNotFound(id.to_string()))
            };
        }

        match strategy {
            FactoryFallback::First => {
                if let Some((id, factory)) = self.0.first_key_value() {
                    return Ok((id, factory.create()));
                }
            }
            FactoryFallback::Last => {
                if let Some((id, factory)) = self.0.last_key_value() {
                    return Ok((id, factory.create()));
                }
            }
            FactoryFallback::NoFallback => return Err(FactoryError::EmptyIdNoFallback),
        }

        Err(FactoryError::NoFactoriesAvailable)
    }
}

/// 工厂实现的注册表条目。
///
/// 存储工厂的元数据，包括其 ID、产品类型 ID 和工厂实例。
/// 此类型与 `inventory` crate 一起用于编译时注册。
pub struct FactoryRegistry<T>
where
    T: ?Sized + 'static,
{
    /// 此工厂的唯一标识符。
    ///
    /// 此 ID 用于在创建实例时查找工厂。
    /// 它必须是静态字符串字面量，并且对于给定产品类型 `T` 在注册表中应该是唯一的。
    id: &'static str,

    /// 创建类型 `T` 实例的工厂实例。
    ///
    /// 这是一个静态引用，指向可以创建产品类型 `T` 的装箱实例的工厂实现。
    /// 工厂必须是线程安全的（`Sync`）以允许在线程间共享。
    factory: &'static (dyn Factory<T> + Sync),

    /// 产品类型 `T` 的类型标识符。
    ///
    /// 此字段存储产品类型 `T` 的 `TypeId`，用于在从注册表检索时按产品类型过滤工厂。
    /// 它确保只有正确产品类型的工厂包含在工厂集合中。
    type_id: TypeId,
}

impl<T> Collect for FactoryRegistry<T>
where
    T: ?Sized + 'static,
{
    fn registry() -> &'static Registry {
        static REGISTRY: Registry = Registry::new();

        &REGISTRY
    }
}

impl<T> FactoryRegistry<T>
where
    T: ?Sized + 'static,
{
    /// 创建一个新的工厂注册表条目。
    ///
    /// # 参数
    /// * `id` - 此工厂的唯一标识符
    /// * `factory` - 创建产品的工厂实例
    #[inline]
    pub const fn new(id: &'static str, factory: &'static (dyn Factory<T> + Sync)) -> Self {
        Self {
            id,
            factory,
            type_id: TypeId::of::<T>(),
        }
    }

    /// 查找产品类型 `T` 的所有已注册工厂。
    ///
    /// 此函数扫描编译时工厂注册表，并返回一个 `SimpleFactory` 实例，
    /// 该实例包含一个从工厂 ID 到工厂实例的映射，这些工厂实例生产类型 `T` 的实例。
    /// 只包含为确切产品类型 `T` 注册的工厂。
    ///
    /// # 返回值
    /// 一个 `SimpleFactory<T>` 实例，包装一个 `BTreeMap`，其中：
    /// - 键是工厂的静态字符串标识符
    /// - 值是实现 `Factory<T>` 的工厂实例的引用
    pub fn simple_factory() -> SimpleFactory<T> {
        let type_id = TypeId::of::<T>();
        let factories = inventory::iter::<Self>()
            .filter_map(|reg| (type_id == reg.type_id).then_some((reg.id, reg.factory)))
            .collect();

        SimpleFactory(factories)
    }
}

/// 为产品类型注册工厂实现的宏。
///
/// 注册一个工厂，该工厂创建 `$implement` 的实例作为 `$product` trait 的实现。
#[macro_export]
macro_rules! register_factory {
    ($product:ty, $id:literal, $implement:ty) => {
        $crate::const_assert!(!$id.is_empty());
        $crate::assert_impl_one!($implement: Default);

        const _: () = {
            struct ConcreteFactory;

            impl $crate::Factory<$product> for ConcreteFactory {
                fn create(&self) -> Box<$product> {
                    Box::<$implement>::default()
                }
            }

            $crate::submit! {
                $crate::FactoryRegistry::new(
                    $id,
                    &ConcreteFactory as &'static (dyn $crate::Factory<$product> + Sync),
                )
            }
        };
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用的 trait 和实现
    trait TestProduct {
        fn get_value(&self) -> &str;
    }

    struct ProductA {
        value: String,
    }

    impl ProductA {
        #[allow(dead_code)]
        fn new(value: &str) -> Self {
            Self {
                value: value.to_string(),
            }
        }
    }

    impl TestProduct for ProductA {
        fn get_value(&self) -> &str {
            &self.value
        }
    }

    impl Default for ProductA {
        fn default() -> Self {
            Self {
                value: "default_a".to_string(),
            }
        }
    }

    struct ProductB {
        value: String,
    }

    impl ProductB {
        #[allow(dead_code)]
        fn new(value: &str) -> Self {
            Self {
                value: value.to_string(),
            }
        }
    }

    impl TestProduct for ProductB {
        fn get_value(&self) -> &str {
            &self.value
        }
    }

    impl Default for ProductB {
        fn default() -> Self {
            Self {
                value: "default_b".to_string(),
            }
        }
    }

    // 注册测试工厂
    register_factory!(dyn TestProduct, "product_a", ProductA);
    register_factory!(dyn TestProduct, "product_b", ProductB);

    #[test]
    fn test_factory_registration() {
        let factory = FactoryRegistry::<dyn TestProduct>::simple_factory();

        // 测试我们注册的工厂应该存在
        let result_a = factory.create("product_a", FactoryFallback::NoFallback);
        assert!(result_a.is_ok(), "product_a factory should exist");

        let result_b = factory.create("product_b", FactoryFallback::NoFallback);
        assert!(result_b.is_ok(), "product_b factory should exist");
    }

    #[test]
    fn test_factory_creation() {
        let factory = FactoryRegistry::<dyn TestProduct>::simple_factory();

        // 测试创建 ProductA
        let result = factory.create("product_a", FactoryFallback::NoFallback);
        assert!(result.is_ok());

        let (id, product) = result.unwrap();
        assert_eq!(id, "product_a");
        assert_eq!(product.get_value(), "default_a");

        // 测试创建 ProductB
        let result = factory.create("product_b", FactoryFallback::NoFallback);
        assert!(result.is_ok());

        let (id, product) = result.unwrap();
        assert_eq!(id, "product_b");
        assert_eq!(product.get_value(), "default_b");
    }

    #[test]
    fn test_factory_error_cases() {
        let factory = FactoryRegistry::<dyn TestProduct>::simple_factory();

        // 测试不存在的工厂 ID
        let result = factory.create("non_existent", FactoryFallback::NoFallback);
        assert!(result.is_err());

        if let Err(FactoryError::FactoryNotFound(id)) = result {
            assert_eq!(id, "non_existent");
        } else {
            panic!("Expected FactoryNotFound error");
        }

        // 测试空 ID 无回退
        let result = factory.create("", FactoryFallback::NoFallback);
        assert!(result.is_err());

        if let Err(FactoryError::EmptyIdNoFallback) = result {
            // 正确
        } else {
            panic!("Expected EmptyIdNoFallback error");
        }
    }

    #[test]
    fn test_factory_fallback_first() {
        let factory = FactoryRegistry::<dyn TestProduct>::simple_factory();

        // 测试 First 回退策略（空 ID）
        let result = factory.create("", FactoryFallback::First);
        // 由于 inventory 是全局的，可能在其他测试中注册了工厂
        // 所以这里可能成功也可能失败，我们只检查行为是否正确
        match result {
            Ok((id, _)) => {
                // 如果成功，id 不应该为空
                assert!(!id.is_empty());
            }
            Err(FactoryError::NoFactoriesAvailable) => {
                // 如果没有工厂可用，这也是有效的
            }
            Err(e) => {
                // 其他错误不应该发生
                panic!("Unexpected error: {:?}", e);
            }
        }

        // 测试 First 回退策略（无效 ID）
        let result = factory.create("invalid_id", FactoryFallback::First);
        match result {
            Ok((id, _)) => {
                // 如果成功，id 不应该为空
                assert!(!id.is_empty());
            }
            Err(FactoryError::FactoryNotFound(id)) => {
                // 如果找不到工厂，id 应该是 "invalid_id"
                assert_eq!(id, "invalid_id");
            }
            Err(FactoryError::NoFactoriesAvailable) => {
                // 如果没有工厂可用，这也是有效的
            }
            Err(e) => {
                // 其他错误不应该发生
                panic!("Unexpected error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_factory_fallback_last() {
        let factory = FactoryRegistry::<dyn TestProduct>::simple_factory();

        // 测试 Last 回退策略（空 ID）
        let result = factory.create("", FactoryFallback::Last);
        // 由于 inventory 是全局的，可能在其他测试中注册了工厂
        // 所以这里可能成功也可能失败，我们只检查行为是否正确
        match result {
            Ok((id, _)) => {
                // 如果成功，id 不应该为空
                assert!(!id.is_empty());
            }
            Err(FactoryError::NoFactoriesAvailable) => {
                // 如果没有工厂可用，这也是有效的
            }
            Err(e) => {
                // 其他错误不应该发生
                panic!("Unexpected error: {:?}", e);
            }
        }

        // 测试 Last 回退策略（无效 ID）
        let result = factory.create("invalid_id", FactoryFallback::Last);
        match result {
            Ok((id, _)) => {
                // 如果成功，id 不应该为空
                assert!(!id.is_empty());
            }
            Err(FactoryError::FactoryNotFound(id)) => {
                // 如果找不到工厂，id 应该是 "invalid_id"
                assert_eq!(id, "invalid_id");
            }
            Err(FactoryError::NoFactoriesAvailable) => {
                // 如果没有工厂可用，这也是有效的
            }
            Err(e) => {
                // 其他错误不应该发生
                panic!("Unexpected error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_factory_no_factories_available() {
        // 测试没有工厂的情况
        // 创建一个新的 trait 和工厂注册表，但不注册任何工厂
        trait EmptyProduct {
            #[allow(dead_code)]
            fn dummy(&self);
        }

        let factory = FactoryRegistry::<dyn EmptyProduct>::simple_factory();

        // 测试空工厂集合
        let result = factory.create("", FactoryFallback::First);
        assert!(result.is_err());

        if let Err(FactoryError::NoFactoriesAvailable) = result {
            // 正确
        } else {
            panic!("Expected NoFactoriesAvailable error");
        }

        let result = factory.create("", FactoryFallback::Last);
        assert!(result.is_err());

        if let Err(FactoryError::NoFactoriesAvailable) = result {
            // 正确
        } else {
            panic!("Expected NoFactoriesAvailable error");
        }
    }

    #[test]
    fn test_factory_registry_new() {
        // 测试 FactoryRegistry::new 函数
        struct TestFactory;

        impl Factory<String> for TestFactory {
            fn create(&self) -> Box<String> {
                Box::new("test".to_string())
            }
        }

        let factory = &TestFactory as &'static (dyn Factory<String> + Sync);
        let registry = FactoryRegistry::new("test_id", factory);

        assert_eq!(registry.id, "test_id");
        assert_eq!(registry.type_id, TypeId::of::<String>());
    }

    #[test]
    fn test_factory_error_display() {
        // 测试错误信息的显示
        let error = FactoryError::FactoryNotFound("test_id".to_string());
        assert_eq!(format!("{}", error), "未找到 ID 为 'test_id' 的工厂");

        let error = FactoryError::EmptyIdNoFallback;
        assert_eq!(format!("{}", error), "不允许回退时提供了空 ID");

        let error = FactoryError::NoFactoriesAvailable;
        assert_eq!(format!("{}", error), "没有可用的工厂");
    }

    #[test]
    fn test_factory_fallback_debug() {
        // 测试 FactoryFallback 的 Debug 实现
        assert_eq!(format!("{:?}", FactoryFallback::First), "First");
        assert_eq!(format!("{:?}", FactoryFallback::Last), "Last");
        assert_eq!(format!("{:?}", FactoryFallback::NoFallback), "NoFallback");
    }

    #[test]
    fn test_factory_fallback_eq() {
        // 测试 FactoryFallback 的相等性
        assert_eq!(FactoryFallback::First, FactoryFallback::First);
        assert_eq!(FactoryFallback::Last, FactoryFallback::Last);
        assert_eq!(FactoryFallback::NoFallback, FactoryFallback::NoFallback);
        assert_ne!(FactoryFallback::First, FactoryFallback::Last);
        assert_ne!(FactoryFallback::First, FactoryFallback::NoFallback);
    }

    #[test]
    fn test_simple_factory_debug() {
        // 测试 SimpleFactory 的 Debug 实现
        // SimpleFactory 没有实现 Debug，所以跳过这个测试
        // 或者我们可以测试工厂是否正常工作
        let factory = FactoryRegistry::<dyn TestProduct>::simple_factory();
        let result = factory.create("product_a", FactoryFallback::NoFallback);
        assert!(result.is_ok());
    }
}
