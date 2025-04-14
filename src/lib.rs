use std::{
    marker::PhantomPinned,
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll, Waker},
};

pub use gen_macro::generator;

#[derive(Debug, Clone, Copy)]
pub struct SuspendOnce(bool);

impl Default for SuspendOnce {
    fn default() -> Self {
        Self(true)
    }
}

impl Future for SuspendOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 {
            self.0 = false;
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

#[derive(Default)]
pub enum GeneratorState<T> {
    #[default]
    None,
    Some(T),
    Stopped,
}

impl<T> GeneratorState<T> {
    /// Reset state to None and return original state.
    fn reset(&mut self) -> Self {
        let mut other = Self::None;
        std::mem::swap(self, &mut other);
        other
    }

    /// Take yielded value if any.
    pub fn take(&mut self) -> Option<T> {
        match self {
            Self::Some(_) => Some(unsafe { self.take_unchecked() }),
            _ => None,
        }
    }

    /// Replace self with Some(v)
    pub fn replace(&mut self, v: T) {
        *self = Self::Some(v);
    }

    /// Take yielded value unchecked.
    /// ## Safety
    /// Call if no yielded value stored is an undefined behavior.
    unsafe fn take_unchecked(&mut self) -> T {
        match self.reset() {
            Self::Some(v) => v,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

impl<T: Clone> Clone for GeneratorState<T> {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Some(v) => Self::Some(v.clone()),
            Self::Stopped => Self::Stopped,
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for GeneratorState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const NAME: &str = "GeneratorState";
        match self {
            Self::None => write!(f, "{NAME}::None"),
            Self::Some(v) => f.debug_tuple(&format!("{NAME}::Some")).field(v).finish(),
            Self::Stopped => write!(f, "{NAME}::Stopped"),
        }
    }
}

pub struct Generator<T, F: Future> {
    state: GeneratorState<T>,
    future: MaybeUninit<Pin<Box<F>>>,
    _pin: PhantomPinned,
}

impl<T, F: Future> Generator<T, F> {
    /// Create a new Generator without future.
    /// ## Safety
    /// The Generator is not indended for manual creating.
    /// Use proc-macro [generator] to create a Generator instead.
    /// Generator will always assume future is inited and drop it,
    /// not initialize the future field before drop generator and call methods
    /// other than [Generator#set_future] is an undefined behavior.
    pub unsafe fn new() -> Self {
        Self {
            state: Default::default(),
            future: MaybeUninit::uninit(),
            _pin: Default::default(),
        }
    }

    /// Set corresponding future.
    /// ## Safety
    /// This is not indended for manual calling.
    /// Must init future before any other methods called.
    /// Must NOT called twice.
    pub unsafe fn set_future(&mut self, future: Pin<Box<F>>) {
        self.future.write(future);
    }

    /// Get generator's state pointer.
    /// ## Safety
    /// This is not indended for manual calling.
    /// Call the function returned if Generator is dropped is an undefined behavior
    pub unsafe fn get_state_ptr(&mut self) -> *mut GeneratorState<T> {
        std::ptr::from_mut(&mut self.state)
    }

    /// Take the yilded item stored in generator's state unchecked.
    /// ## Safety
    /// Call if generator has not started, already ended or item stored has already taken
    /// is an undefined behavior.
    unsafe fn take_item_unchecked(&mut self) -> T {
        unsafe { self.state.take_unchecked() }
    }

    /// Check if the generator ends.
    pub fn check_finished(&mut self) -> bool {
        matches!(self.state, GeneratorState::Stopped)
    }

    // Set the generator as finished.
    fn set_finished(&mut self) {
        self.state = GeneratorState::Stopped;
    }
}

impl<T, F: Future<Output = ()>> Iterator for Generator<T, F> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.check_finished() {
            return None;
        }
        let cx = &mut Context::from_waker(Waker::noop());
        match unsafe { self.future.assume_init_mut() }.as_mut().poll(cx) {
            Poll::Pending => Some(unsafe { self.take_item_unchecked() }),
            Poll::Ready(_) => {
                self.set_finished();
                None
            }
        }
    }
}

impl<T, F: Future> Drop for Generator<T, F> {
    fn drop(&mut self) {
        unsafe { self.future.assume_init_drop() };
    }
}

impl<T, F: Future> std::fmt::Debug for Generator<T, F>
where
    T: std::fmt::Debug,
    F::Output: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("Generator<{}>", std::any::type_name::<T>());
        f.debug_struct(&name).field("state", &self.state).finish()
    }
}

#[cfg(test)]
mod test {
    use super::{Generator, SuspendOnce};

    fn fib() -> impl Iterator<Item = usize> {
        unsafe {
            let mut generator = Box::new(Generator::new());
            let state_ptr = generator.get_state_ptr();
            generator.set_future(Box::pin(async move {
                let state = &mut *state_ptr;
                let mut n1 = 1;
                let mut n2 = 1;
                let mut n3 = 2;
                // yield n1;
                {
                    state.replace(n1);
                    SuspendOnce::default().await;
                }
                // yield n2;
                {
                    state.replace(n2);
                    SuspendOnce::default().await;
                }
                loop {
                    // yield n3;
                    {
                        state.replace(n3);
                        SuspendOnce::default().await;
                    }
                    n1 = n2;
                    n2 = n3;
                    n3 = n1 + n2;
                }
            }));
            generator
        }
    }

    #[test]
    fn testmain() {
        let fib: Vec<_> = fib().take(10).collect();
        assert_eq!(fib, vec![1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
    }
}
