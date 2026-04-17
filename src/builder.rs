/// 为构建器模式提供辅助方法的宏。
///
/// 此宏为指定的构建器类型添加一个 `when_some` 方法，允许条件性地设置可选值。
/// 当 `Option<T>` 为 `Some` 时，会调用提供的函数来设置值；当为 `None` 时，直接返回构建器本身。
///
/// # 用法
///
/// 有两种调用方式：
/// 1. 为 `Self` 类型实现：
///    ```rust
///    use rust_patterns_components::builder_helper;
///
///    struct MyBuilder1;
///    struct MyBuilder2;
///
///    builder_helper!(Self, MyBuilder1, MyBuilder2);
///
///    // 现在 MyBuilder1 和 MyBuilder2 都有 when_some 方法
///    ```
/// 2. 为 `&mut Self` 类型实现：
///    ```rust
///    use rust_patterns_components::builder_helper;
///
///    struct MyBuilder1;
///    struct MyBuilder2;
///
///    builder_helper!(&mut Self, MyBuilder1, MyBuilder2);
///
///    // 现在 MyBuilder1 和 MyBuilder2 都有 when_some 方法
///    ```
///
/// # 生成的方法
///
/// 宏会为每个指定的构建器类型生成一个 `when_some` 方法：
/// ```rust
/// # use rust_patterns_components::builder_helper;
/// # struct MyBuilder;
/// # builder_helper!(Self, MyBuilder);
/// # impl MyBuilder {
/// fn when_some<T>(self, value: Option<T>, func: impl FnOnce(Self, T) -> Self) -> Self
/// where
///     Self: Sized,
/// {
///     match value {
///         Some(v) => func(self, v),
///         None => self,
///     }
/// }
/// # }
/// ```
///
/// 或者对于 `&mut Self` 版本：
/// ```rust
/// # use rust_patterns_components::builder_helper;
/// # struct MyBuilder;
/// # builder_helper!(&mut Self, MyBuilder);
/// # impl MyBuilder {
/// fn when_some<T>(&mut self, value: Option<T>, func: impl FnOnce(&mut Self, T) -> &mut Self) -> &mut Self
/// where
///     Self: Sized,
/// {
///     match value {
///         Some(v) => func(self, v),
///         None => self,
///     }
/// }
/// # }
/// ```
#[macro_export]
macro_rules! builder_helper {
    (@ { $self:ty, $($builder:ty),+ }) => {
        /// 构建器辅助 trait，提供条件性设置值的方法。
        trait Builder {
            #[inline]
            fn when_some<T>(self: $self, value: Option<T>, func: impl FnOnce($self, T) -> $self) -> $self
            where
                Self: Sized,
            {
                match value {
                    Some(v) => func(self, v),
                    None => self,
                }
            }
        }

        $(impl Builder for $builder {})+
    };

    (Self, $($builder:ty),+ $(,)?) => {
        $crate::builder_helper!(@ {Self, $($builder),+});
    };

    (&mut Self, $($builder:ty),+ $(,)?) => {
        $crate::builder_helper!(@ {&mut Self, $($builder),+});
    };
}

#[cfg(test)]
mod tests {
    // 不需要导入 super::*，因为宏已经在当前作用域

    #[test]
    fn test_builder_helper_self() {
        // 测试 Self 版本的构建器
        struct TestBuilder {
            value: Option<String>,
            count: Option<u32>,
        }

        impl TestBuilder {
            fn new() -> Self {
                Self {
                    value: None,
                    count: None,
                }
            }

            fn set_value(mut self, value: String) -> Self {
                self.value = Some(value);
                self
            }

            fn set_count(mut self, count: u32) -> Self {
                self.count = Some(count);
                self
            }

            fn build(self) -> (Option<String>, Option<u32>) {
                (self.value, self.count)
            }
        }

        // 为 TestBuilder 实现 builder_helper
        builder_helper!(Self, TestBuilder);

        // 测试 when_some 方法
        let builder = TestBuilder::new()
            .when_some(Some("test".to_string()), |b, v| b.set_value(v))
            .when_some(None::<u32>, |b, _| b.set_count(0))
            .when_some(Some(42), |b, c| b.set_count(c));

        let (value, count) = builder.build();
        assert_eq!(value, Some("test".to_string()));
        assert_eq!(count, Some(42));
    }

    #[test]
    fn test_builder_helper_mut_self() {
        // 测试 &mut Self 版本的构建器
        struct MutTestBuilder {
            value: Option<String>,
            count: Option<u32>,
        }

        impl MutTestBuilder {
            fn new() -> Self {
                Self {
                    value: None,
                    count: None,
                }
            }

            fn set_value(&mut self, value: String) -> &mut Self {
                self.value = Some(value);
                self
            }

            fn set_count(&mut self, count: u32) -> &mut Self {
                self.count = Some(count);
                self
            }

            fn build(&self) -> (Option<String>, Option<u32>) {
                (self.value.clone(), self.count)
            }
        }

        // 为 MutTestBuilder 实现 builder_helper
        builder_helper!(&mut Self, MutTestBuilder);

        // 测试 when_some 方法
        let mut builder = MutTestBuilder::new();
        builder
            .when_some(Some("mut_test".to_string()), |b, v| b.set_value(v))
            .when_some(None::<u32>, |b, _| b.set_count(0))
            .when_some(Some(99), |b, c| b.set_count(c));

        let (value, count) = builder.build();
        assert_eq!(value, Some("mut_test".to_string()));
        assert_eq!(count, Some(99));
    }

    #[test]
    fn test_builder_helper_multiple_types() {
        // 测试为多个类型同时实现
        struct Builder1 {
            value: Option<String>,
        }

        struct Builder2 {
            value: Option<String>,
        }

        impl Builder1 {
            fn new() -> Self {
                Self { value: None }
            }

            fn set_value(mut self, value: String) -> Self {
                self.value = Some(value);
                self
            }

            fn build(self) -> Option<String> {
                self.value
            }
        }

        impl Builder2 {
            fn new() -> Self {
                Self { value: None }
            }

            fn set_value(mut self, value: String) -> Self {
                self.value = Some(value);
                self
            }

            fn build(self) -> Option<String> {
                self.value
            }
        }

        // 为两个构建器同时实现
        builder_helper!(Self, Builder1, Builder2);

        // 测试 Builder1
        let builder1 =
            Builder1::new().when_some(Some("builder1".to_string()), |b, v| b.set_value(v));
        assert_eq!(builder1.build(), Some("builder1".to_string()));

        // 测试 Builder2
        let builder2 =
            Builder2::new().when_some(Some("builder2".to_string()), |b, v| b.set_value(v));
        assert_eq!(builder2.build(), Some("builder2".to_string()));
    }

    #[test]
    fn test_when_some_with_none() {
        struct SimpleBuilder {
            value: Option<String>,
        }

        impl SimpleBuilder {
            fn new() -> Self {
                Self { value: None }
            }

            fn set_value(mut self, value: String) -> Self {
                self.value = Some(value);
                self
            }

            fn build(self) -> Option<String> {
                self.value
            }
        }

        builder_helper!(Self, SimpleBuilder);

        // 当值为 None 时，应该直接返回构建器而不调用函数
        let builder = SimpleBuilder::new().when_some(None::<String>, |b, _| {
            b.set_value("should not be set".to_string())
        });

        assert_eq!(builder.build(), None);
    }

    #[test]
    fn test_when_some_with_some() {
        struct SimpleBuilder {
            value: Option<String>,
        }

        impl SimpleBuilder {
            fn new() -> Self {
                Self { value: None }
            }

            fn set_value(mut self, value: String) -> Self {
                self.value = Some(value);
                self
            }

            fn build(self) -> Option<String> {
                self.value
            }
        }

        builder_helper!(Self, SimpleBuilder);

        // 当值为 Some 时，应该调用函数
        let builder =
            SimpleBuilder::new().when_some(Some("test_value".to_string()), |b, v| b.set_value(v));

        assert_eq!(builder.build(), Some("test_value".to_string()));
    }

    #[test]
    fn test_chaining_with_when_some() {
        struct ChainBuilder {
            value1: Option<String>,
            value2: Option<String>,
            count: Option<u32>,
        }

        impl ChainBuilder {
            fn new() -> Self {
                Self {
                    value1: None,
                    value2: None,
                    count: None,
                }
            }

            fn set_value1(mut self, value: String) -> Self {
                self.value1 = Some(value);
                self
            }

            fn set_value2(mut self, value: String) -> Self {
                self.value2 = Some(value);
                self
            }

            fn set_count(mut self, count: u32) -> Self {
                self.count = Some(count);
                self
            }

            fn build(self) -> (Option<String>, Option<String>, Option<u32>) {
                (self.value1, self.value2, self.count)
            }
        }

        builder_helper!(Self, ChainBuilder);

        // 测试链式调用
        let builder = ChainBuilder::new()
            .set_value1("value1".to_string())
            .when_some(Some("value2".to_string()), |b, v| b.set_value2(v))
            .when_some(Some(100), |b, c| b.set_count(c))
            .when_some(None::<String>, |b, _| b.set_value1("ignored".to_string()));

        let (value1, value2, count) = builder.build();
        assert_eq!(value1, Some("value1".to_string()));
        assert_eq!(value2, Some("value2".to_string()));
        assert_eq!(count, Some(100));
    }
}
