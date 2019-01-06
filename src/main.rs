use cache::Cache;

struct A(usize);

impl A {
    fn new(x: usize) -> Self {
        A(x)
    }

    fn inner(&self) -> usize {
        self.0
    }
}

fn main() {
    let c = Cache::new(Box::new(|| A::new(0)));

    assert_eq!(c.get().inner(), 0);
}
