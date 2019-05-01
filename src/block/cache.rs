
#[derive(Debug, Clone)]
pub enum Cache<T> {
    Empty,
    Cached(usize, T),
}

impl<T> Cache<T> {
    pub fn is_valid(&self, cache_id: usize) -> bool {
        match *self {
            Cache::Empty => false,
            Cache::Cached(id, _) => cache_id == id,
        }
    }
    pub fn as_ref(&self) -> Cache<&T> {
        match self {
            Cache::Empty => Cache::Empty,
            Cache::Cached(id, ref value) => Cache::Cached(*id, value),
        }
    }
    pub fn unwrap(self) -> T {
        match self {
            Cache::Empty => panic!("Unwrapped empty cache"),
            Cache::Cached(_, value) => value,
        }
    }
    pub fn update(&mut self, id: usize, value: T) {
        *self = Cache::Cached(id, value);
    }
    pub fn map<F, U>(self, f: F) -> Cache<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Cache::Empty => Cache::Empty,
            Cache::Cached(id, value) => Cache::Cached(id, f(value)),
        }
    }
}

impl<T> Cache<T>
where
    T: Default,
{
    pub fn unwrap_or_default(self) -> T {
        match self {
            Cache::Empty => Default::default(),
            Cache::Cached(_, value) => value,
        }
    }
}

