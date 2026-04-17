use std::sync::{Arc, Weak};

/// 观察者 trait
///
/// 定义观察者必须实现的接口。观察者可以订阅被观察者的状态变化，
/// 并在状态更新时通过 `update` 方法接收通知。
///
/// # 关联类型
///
/// - `Subject`: 观察者订阅的被观察者类型，必须实现 [`Observable`] trait
///
/// # 方法
///
/// - `update`: 接收状态更新通知
///
/// # 实现要求
///
/// 实现者需要指定具体的被观察者类型，并实现 `update` 方法。
/// `update` 方法应该快速返回，避免阻塞通知过程。
///
/// # 线程安全
///
/// 实现者应确保 `update` 方法是线程安全的，因为可能从多个线程调用。
///
/// # 示例
///
/// ```
/// use std::sync::Arc;
/// use rust_patterns_components::{Observer, Observable};
///
/// struct TemperatureSensor;
///
/// impl Observable for TemperatureSensor {
///     type State = f64;
///     type Error = String;
///
///     fn attach(&mut self, _observer: Arc<dyn Observer<Subject = Self>>) {}
///     fn detach(&mut self, _observer: Arc<dyn Observer<Subject = Self>>) {}
/// }
///
/// struct TemperatureDisplay;
///
/// impl Observer for TemperatureDisplay {
///     type Subject = TemperatureSensor;
///
///     fn update(&self, state: &f64) -> Result<(), String> {
///         println!("当前温度: {:.1}°C", state);
///         Ok(())
///     }
/// }
/// ```
pub trait Observer {
    /// 观察者订阅的被观察者类型
    ///
    /// 此类型必须实现 [`Observable`] trait，定义了观察者关注的状态和错误类型。
    type Subject: Observable;

    /// 接收状态更新通知
    ///
    /// 当被观察者状态发生变化时调用此方法。实现者应该：
    ///
    /// 1. 处理传入的状态
    /// 2. 返回 `Ok(())` 表示处理成功
    /// 3. 返回 `Err(error)` 表示处理失败
    ///
    /// # 参数
    ///
    /// - `state`: 当前的被观察者状态引用
    ///
    /// # 返回值
    ///
    /// - `Ok(())`: 成功处理状态更新
    /// - `Err(<Self::Subject as Observable>::Error)`: 处理状态更新时发生错误
    ///
    /// # 错误处理
    ///
    /// 如果此方法返回错误，被观察者的 `notify` 方法会根据指定的通知策略
    /// 决定是否继续通知其他观察者。
    fn update(
        &self,
        state: &<Self::Subject as Observable>::State,
    ) -> Result<(), <Self::Subject as Observable>::Error>;
}

/// 被观察者 trait
///
/// 定义被观察者必须实现的接口。被观察者维护一组观察者，
/// 并在状态变化时通知它们。
///
/// # 关联类型
///
/// - `State`: 被观察者的状态类型，观察者通过此类型接收状态更新
/// - `Error`: 观察者处理更新时可能返回的错误类型
///
/// # 方法
///
/// - `attach`: 附加观察者到被观察者
/// - `detach`: 从被观察者分离观察者
///
/// # 实现要求
///
/// 实现者需要指定状态类型和错误类型，并实现观察者管理方法。
/// 通常建议使用 [`ObserverRegistry`] 来简化实现。
///
/// # 线程安全
///
/// 实现者应确保观察者管理方法是线程安全的，因为可能从多个线程调用。
///
/// # 示例
///
/// ```
/// use std::sync::Arc;
/// use rust_patterns_components::{Observable, Observer, ObserverRegistry};
///
/// struct WeatherStation {
///     registry: ObserverRegistry<Self>,
/// }
///
/// impl Observable for WeatherStation {
///     type State = f64;
///     type Error = String;
///
///     fn attach(&mut self, observer: Arc<dyn Observer<Subject = Self>>) {
///         self.registry.attach(observer);
///     }
///
///     fn detach(&mut self, observer: Arc<dyn Observer<Subject = Self>>) {
///         self.registry.detach(observer);
///     }
/// }
/// ```
pub trait Observable {
    /// 被观察者的状态类型
    ///
    /// 当状态变化时，会传递此类型的值给观察者。
    type State;

    /// 观察者处理更新时可能返回的错误类型
    ///
    /// 如果观察者处理更新失败，可以返回此类型的错误。
    type Error;

    /// 附加观察者
    ///
    /// 将观察者附加到被观察者。附加后，观察者将收到状态更新通知。
    ///
    /// # 参数
    ///
    /// - `observer`: 要附加的观察者强引用
    fn attach(&mut self, observer: Arc<dyn Observer<Subject = Self>>);

    /// 分离观察者
    ///
    /// 从被观察者分离指定的观察者。分离后，该观察者将不再收到状态更新通知。
    ///
    /// # 参数
    ///
    /// - `observer`: 要分离的观察者强引用
    fn detach(&mut self, observer: Arc<dyn Observer<Subject = Self>>);
}

/// 观察者注册表
///
/// 管理一组观察者并在状态变化时通知它们。注册表维护观察者的弱引用列表，
/// 避免强引用循环导致的内存泄漏。
///
/// # 类型参数
///
/// - `T`: 被观察者类型，必须实现 [`Observable`] trait
///
/// # 设计特点
///
/// - **弱引用管理**: 使用 `Weak` 引用存储观察者，允许观察者在不再需要时被释放
/// - **自动清理**: 在通知时自动跳过已释放的观察者弱引用
/// - **通知策略**: 支持两种通知策略
/// - **防止重复**: 检查观察者是否已存在，防止重复添加
///
/// # 示例
///
/// ```
/// use std::sync::Arc;
/// use rust_patterns_components::{Observable, Observer, ObserverRegistry};
///
/// struct Sensor;
///
/// impl Observable for Sensor {
///     type State = String;
///     type Error = String;
///
///     fn attach(&mut self, _observer: Arc<dyn Observer<Subject = Self>>) {}
///     fn detach(&mut self, _observer: Arc<dyn Observer<Subject = Self>>) {}
/// }
///
/// let mut registry = ObserverRegistry::<Sensor>::default();
/// // 使用 registry.attach() 添加观察者
/// // 使用 registry.notify() 通知观察者
/// ```
pub struct ObserverRegistry<T: Observable> {
    /// 观察者弱引用列表
    ///
    /// 使用弱引用避免循环引用。当观察者被释放时，
    /// 对应的弱引用会自动变为无效。
    observers: Vec<Weak<dyn Observer<Subject = T>>>,
}

impl<T> ObserverRegistry<T>
where
    T: Observable,
{
    /// 创建新的观察者注册表实例
    ///
    /// # 返回值
    ///
    /// 返回一个空的观察者注册表，不包含任何观察者。
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    /// 创建具有初始容量的观察者注册表实例
    ///
    /// # 参数
    ///
    /// - `capacity`: 初始容量，用于预分配内存
    ///
    /// # 返回值
    ///
    /// 返回一个具有指定初始容量的空注册表。
    ///
    /// # 性能
    ///
    /// 预分配容量可以避免后续添加观察者时的多次内存重新分配，
    /// 当已知观察者数量时建议使用此方法。
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            observers: Vec::with_capacity(capacity),
        }
    }

    /// 附加观察者
    ///
    /// 将观察者附加到注册表。方法内部会将观察者的强引用转换为弱引用，
    /// 并确保不会重复添加相同的观察者。
    ///
    /// # 参数
    ///
    /// - `observer`: 要附加的观察者强引用
    ///
    /// # 注意
    ///
    /// - 使用 `Arc::downgrade` 将强引用转换为弱引用，避免循环引用
    /// - 使用 `Weak::ptr_eq` 检查观察者是否已存在，防止重复添加
    /// - 观察者将在下次调用 `notify` 方法时收到状态更新通知
    pub fn attach(&mut self, observer: Arc<dyn Observer<Subject = T>>) {
        let weak = Arc::downgrade(&observer);
        if !self.observers.iter().any(|item| weak.ptr_eq(item)) {
            self.observers.push(weak);
        }
    }

    /// 分离观察者
    ///
    /// 从注册表中分离指定的观察者。分离后，该观察者将不再收到状态更新通知。
    ///
    /// # 参数
    ///
    /// - `observer`: 要分离的观察者强引用
    ///
    /// # 注意
    ///
    /// - 使用 `Arc::downgrade` 将强引用转换为弱引用进行匹配
    /// - 使用 `Weak::ptr_eq` 进行引用相等性比较
    /// - 如果观察者不存在于列表中，此方法不会有任何效果
    pub fn detach(&mut self, observer: Arc<dyn Observer<Subject = T>>) {
        let weak = Arc::downgrade(&observer);
        self.observers.retain(|item| !weak.ptr_eq(item));
    }

    /// 通知所有观察者
    ///
    /// 向所有有效的观察者发送状态更新通知。
    ///
    /// # 参数
    ///
    /// - `state`: 要通知的新状态引用
    ///
    /// # 返回值
    ///
    /// - `Ok(())`: 所有观察者都成功处理了更新
    /// - `Err(<T as Observable>::Error)`: 某个观察者处理更新时返回了错误，立即停止通知其他观察者
    ///
    /// # 行为
    ///
    /// - 遍历所有观察者并调用其 `update` 方法
    /// - 当某个观察者返回错误时，立即停止并返回该错误
    /// - 自动清理无效的观察者弱引用（通过 `Weak::upgrade` 过滤）
    ///
    /// # 性能
    ///
    /// 此方法会自动清理无效的观察者弱引用（通过 `Weak::upgrade` 过滤）。
    pub fn notify(&self, state: &<T as Observable>::State) -> Result<(), <T as Observable>::Error> {
        self.observers
            .iter()
            .flat_map(Weak::upgrade)
            .try_for_each(|observer| observer.update(state))
    }

    /// 通知所有观察者状态变化，忽略错误
    ///
    /// 此方法会通知所有已注册的观察者状态变化，但会忽略任何观察者返回的错误。
    /// 即使某个观察者处理更新失败，也会继续通知其他观察者。
    ///
    /// # 参数
    ///
    /// * `state` - 要通知的状态变化
    ///
    /// # 行为
    ///
    /// - 遍历所有观察者并调用其 `update` 方法
    /// - 忽略所有观察者返回的错误（使用 `let _ = ...`）
    /// - 自动清理无效的观察者弱引用（通过 `Weak::upgrade` 过滤）
    ///
    /// # 使用场景
    ///
    /// 适用于以下情况：
    /// - 观察者的错误不应该阻止其他观察者接收通知
    /// - 错误处理不是关键，可以安全忽略
    /// - 需要确保所有观察者都能收到通知，即使某些观察者可能失败
    ///
    /// # 与 `notify` 方法的区别
    ///
    /// - `notify`: 遇到第一个错误就停止，并返回该错误
    /// - `notify_ignore_error`: 忽略所有错误，继续通知所有观察者
    ///
    /// # 示例
    ///
    /// ```rust
    /// use rust_pattern_components::{Observable, Observer, ObserverRegistry};
    /// use std::sync::{Arc, Weak};
    ///
    /// struct Counter {
    ///     registry: ObserverRegistry<Self>,
    ///     value: u64,
    /// }
    ///
    /// impl Observable for Counter {
    ///     type State = u64;
    ///     type Error = String;
    ///
    ///     fn attach(&mut self, observer: Arc<dyn Observer<Subject = Self>>) {
    ///         self.registry.attach(observer);
    ///     }
    ///
    ///     fn detach(&mut self, observer: Arc<dyn Observer<Subject = Self>>) {
    ///         self.registry.detach(observer);
    ///     }
    /// }
    ///
    /// let counter = Counter {
    ///     registry: ObserverRegistry::new(),
    ///     value: 42,
    /// };
    ///
    /// // 即使观察者可能失败，也会通知所有观察者
    /// counter.registry.notify_ignore_error(&counter.value);
    /// ```
    pub fn notify_ignore_error(&self, state: &<T as Observable>::State) {
        self.observers
            .iter()
            .flat_map(Weak::upgrade)
            .for_each(|observer| {
                let _ = observer.update(state);
            })
    }
}

impl<T> Default for ObserverRegistry<T>
where
    T: Observable,
{
    /// 创建默认的观察者注册表实例
    ///
    /// 等同于调用 [`ObserverRegistry::new()`]。
    fn default() -> Self {
        Self {
            observers: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // 测试用的被观察者
    struct TestObservable {
        registry: ObserverRegistry<Self>,
        value: i32,
    }

    impl TestObservable {
        fn new(initial_value: i32) -> Self {
            Self {
                registry: ObserverRegistry::new(),
                value: initial_value,
            }
        }

        fn update_value(&mut self, new_value: i32) -> Result<(), String> {
            self.value = new_value;
            self.registry.notify(&self.value)
        }

        fn get_value(&self) -> i32 {
            self.value
        }
    }

    impl Observable for TestObservable {
        type State = i32;
        type Error = String;

        fn attach(&mut self, observer: Arc<dyn Observer<Subject = Self>>) {
            self.registry.attach(observer);
        }

        fn detach(&mut self, observer: Arc<dyn Observer<Subject = Self>>) {
            self.registry.detach(observer);
        }
    }

    // 简单的测试观察者
    struct TestObserver {
        _name: String,
        last_value: AtomicUsize,
    }

    impl TestObserver {
        fn new(name: &str) -> Self {
            Self {
                _name: name.to_string(),
                last_value: AtomicUsize::new(0),
            }
        }

        fn get_last_value(&self) -> usize {
            self.last_value.load(Ordering::SeqCst)
        }
    }

    impl Observer for TestObserver {
        type Subject = TestObservable;

        fn update(&self, value: &i32) -> Result<(), String> {
            self.last_value.store(*value as usize, Ordering::SeqCst);
            Ok(())
        }
    }

    // 可能失败的测试观察者
    struct FailingObserver {
        fail_after: usize,
        call_count: AtomicUsize,
    }

    impl FailingObserver {
        fn new(fail_after: usize) -> Self {
            Self {
                fail_after,
                call_count: AtomicUsize::new(0),
            }
        }

        fn get_call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    impl Observer for FailingObserver {
        type Subject = TestObservable;

        fn update(&self, _value: &i32) -> Result<(), String> {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
            if count >= self.fail_after {
                Err(format!("Failed after {} calls", count))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_attach_and_notify() {
        let mut observable = TestObservable::new(0);
        let observer = Arc::new(TestObserver::new("test"));

        // 附加观察者
        observable.attach(observer.clone());

        // 更新值并通知
        assert!(observable.update_value(42).is_ok());
        assert_eq!(observable.get_value(), 42);
        assert_eq!(observer.get_last_value(), 42);
    }

    #[test]
    fn test_detach() {
        let mut observable = TestObservable::new(0);
        let observer = Arc::new(TestObserver::new("test"));

        // 附加观察者
        observable.attach(observer.clone());

        // 更新一次
        assert!(observable.update_value(10).is_ok());
        assert_eq!(observer.get_last_value(), 10);

        // 分离观察者
        observable.detach(observer.clone());

        // 再次更新，观察者不应该收到通知
        assert!(observable.update_value(20).is_ok());
        assert_eq!(observer.get_last_value(), 10); // 仍然是旧值
    }

    #[test]
    fn test_multiple_observers() {
        let mut observable = TestObservable::new(0);
        let observer1 = Arc::new(TestObserver::new("observer1"));
        let observer2 = Arc::new(TestObserver::new("observer2"));

        observable.attach(observer1.clone());
        observable.attach(observer2.clone());

        assert!(observable.update_value(100).is_ok());

        assert_eq!(observer1.get_last_value(), 100);
        assert_eq!(observer2.get_last_value(), 100);
    }

    #[test]
    fn test_notify_stop_on_error() {
        let mut observable = TestObservable::new(0);
        let failing_observer = Arc::new(FailingObserver::new(2)); // 第二次调用失败
        let normal_observer = Arc::new(TestObserver::new("normal"));

        observable.attach(failing_observer.clone());
        observable.attach(normal_observer.clone());

        // 第一次更新应该成功
        assert!(observable.update_value(1).is_ok());
        assert_eq!(failing_observer.get_call_count(), 1);
        assert_eq!(normal_observer.get_last_value(), 1);

        // 第二次更新应该失败（StopOnError 策略）
        assert!(observable.update_value(2).is_err());
        assert_eq!(failing_observer.get_call_count(), 2);
        assert_eq!(normal_observer.get_last_value(), 1); // 正常观察者不应该收到第二次通知
    }

    #[test]
    fn test_notify_ignore_error() {
        // 为 IgnoreErrorObservable 创建专门的观察者
        struct IgnoreErrorObservable {
            registry: ObserverRegistry<Self>,
            value: i32,
        }

        struct IgnoreErrorObserver {
            call_count: AtomicUsize,
        }

        impl IgnoreErrorObserver {
            fn new() -> Self {
                Self {
                    call_count: AtomicUsize::new(0),
                }
            }

            fn get_call_count(&self) -> usize {
                self.call_count.load(Ordering::SeqCst)
            }
        }

        impl Observer for IgnoreErrorObserver {
            type Subject = IgnoreErrorObservable;

            fn update(&self, _value: &i32) -> Result<(), String> {
                let count = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= 2 {
                    Err(format!("Failed after {} calls", count))
                } else {
                    Ok(())
                }
            }
        }

        struct NormalObserver {
            last_value: AtomicUsize,
        }

        impl NormalObserver {
            fn new() -> Self {
                Self {
                    last_value: AtomicUsize::new(0),
                }
            }

            fn get_last_value(&self) -> usize {
                self.last_value.load(Ordering::SeqCst)
            }
        }

        impl Observer for NormalObserver {
            type Subject = IgnoreErrorObservable;

            fn update(&self, value: &i32) -> Result<(), String> {
                self.last_value.store(*value as usize, Ordering::SeqCst);
                Ok(())
            }
        }

        impl IgnoreErrorObservable {
            fn new(initial_value: i32) -> Self {
                Self {
                    registry: ObserverRegistry::new(),
                    value: initial_value,
                }
            }

            fn update_value(&mut self, new_value: i32) {
                self.value = new_value;
                self.registry.notify_ignore_error(&self.value);
            }
        }

        impl Observable for IgnoreErrorObservable {
            type State = i32;
            type Error = String;

            fn attach(&mut self, observer: Arc<dyn Observer<Subject = Self>>) {
                self.registry.attach(observer);
            }

            fn detach(&mut self, observer: Arc<dyn Observer<Subject = Self>>) {
                self.registry.detach(observer);
            }
        }

        let mut observable = IgnoreErrorObservable::new(0);
        let failing_observer = Arc::new(IgnoreErrorObserver::new());
        let normal_observer = Arc::new(NormalObserver::new());

        observable.attach(failing_observer.clone());
        observable.attach(normal_observer.clone());

        // 第一次更新应该成功
        observable.update_value(1);
        assert_eq!(failing_observer.get_call_count(), 1);
        assert_eq!(normal_observer.get_last_value(), 1);

        // 第二次更新应该成功（IgnoreError 策略）
        observable.update_value(2);
        assert_eq!(failing_observer.get_call_count(), 2);
        assert_eq!(normal_observer.get_last_value(), 2); // 正常观察者应该收到第二次通知
    }

    #[test]
    fn test_observer_weak_references() {
        let mut observable = TestObservable::new(0);

        {
            let observer = Arc::new(TestObserver::new("temp"));
            observable.attach(observer.clone());

            // 更新一次
            assert!(observable.update_value(50).is_ok());
            assert_eq!(observer.get_last_value(), 50);
        } // observer 在这里被丢弃

        // 再次更新，应该仍然工作（弱引用会被清理）
        assert!(observable.update_value(60).is_ok());
    }

    #[test]
    fn test_duplicate_attach() {
        let mut observable = TestObservable::new(0);
        let observer = Arc::new(TestObserver::new("test"));

        // 多次附加同一个观察者
        observable.attach(observer.clone());
        observable.attach(observer.clone());
        observable.attach(observer.clone());

        // 应该只通知一次
        assert!(observable.update_value(99).is_ok());
        assert_eq!(observer.get_last_value(), 99);
    }

    #[test]
    fn test_detach_non_existent() {
        let mut observable = TestObservable::new(0);
        let observer = Arc::new(TestObserver::new("test"));
        let another_observer = Arc::new(TestObserver::new("another"));

        // 附加一个观察者
        observable.attach(observer.clone());

        // 尝试分离一个未附加的观察者（应该没有效果）
        observable.detach(another_observer.clone());

        // 更新应该仍然工作
        assert!(observable.update_value(33).is_ok());
        assert_eq!(observer.get_last_value(), 33);
    }

    #[test]
    fn test_notify_with_no_observers() {
        let mut observable = TestObservable::new(0);

        // 没有观察者时更新应该成功
        assert!(observable.update_value(77).is_ok());
        assert_eq!(observable.get_value(), 77);
    }
}
