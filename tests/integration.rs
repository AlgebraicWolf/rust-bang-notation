use bang_notation::bang;

#[test]
fn simple001() {
    let x: Option<i32> = Some(42);
    let y: Option<i32> = None;

    assert_eq!(bang!(Some(!x + !y)), None);
}

#[test]
fn simple002() {
    let x: Option<i32> = Some(42);
    let y: Option<i32> = Some(58);

    assert_eq!(bang!(Some(!x + !y)), Some(100));
}

#[test]
fn simple003() {
    let res1: Option<i32> = bang!(Some(!Some(!Some(!None))));
    assert_eq!(res1, None);

    let res2 = bang!(Some(!Some(!Some(!Some(42)))));
    assert_eq!(res2, Some(42));
}

// This is a testing utility intended to check that binding occurs
// in appropriate order.
struct LoggingBox<T> {
    val: T,
    tag: Vec<u32>,
}

impl<T> LoggingBox<T> {
    fn new(val: T, tag: u32) -> LoggingBox<T> {
        LoggingBox {
            val: val,
            tag: vec![tag],
        }
    }

    fn and_then<U, F>(mut self, f: F) -> LoggingBox<U>
    where
        F: FnOnce(T) -> LoggingBox<U>,
    {
        let mut new_box = f(self.val);
        self.tag.append(&mut new_box.tag);
        new_box.tag = self.tag;
        new_box
    }

    fn pure(x: T) -> LoggingBox<T> {
        LoggingBox {
            val: x,
            tag: Vec::new(),
        }
    }
}

fn sum3(x: u32, y: u32, z: u32) -> u32 {
    x + y + z
}

fn sum3_logged(x: u32, y: u32, z: u32, tag: u32) -> LoggingBox<u32> {
    LoggingBox::new(x + y + z, tag)
}

// Make sure that the variables are bound left-to-right
#[test]
fn order001() {
    let x1 = LoggingBox::<u32>::new(1, 0);
    let x2 = LoggingBox::<u32>::new(2, 1);
    let x3 = LoggingBox::<u32>::new(3, 2);

    let res = bang!(LoggingBox::pure(sum3(!x1, !x2, !x3)));
    assert_eq!(res.val, 6);

    for i in 0..res.tag.len() {
        assert_eq!(res.tag[i], i.try_into().unwrap());
    }
}

#[test]
fn order002() {
    let x1 = LoggingBox::<u32>::new(1, 0);
    let x2 = LoggingBox::<u32>::new(2, 1);
    let x3 = LoggingBox::<u32>::new(3, 2);
    let x4 = LoggingBox::<u32>::new(4, 4);
    let x5 = LoggingBox::<u32>::new(5, 5);
    let x6 = LoggingBox::<u32>::new(6, 6);
    let x7 = LoggingBox::<u32>::new(7, 8);
    let x8 = LoggingBox::<u32>::new(8, 10);
    let x9 = LoggingBox::<u32>::new(9, 11);
    let res = bang!(sum3_logged(
        !sum3_logged(
            !sum3_logged(!x1, !x2, !x3, 3),
            !sum3_logged(!x4, !x5, !x6, 7),
            !x7,
            9
        ),
        !x8,
        !x9,
        12
    ));

    assert_eq!(res.val, 45);

    for i in 0..res.tag.len() {
        assert_eq!(res.tag[i], i.try_into().unwrap());
    }
}

// For fun
#[derive(Debug)]
enum List<T> {
    Nil,
    Cons(T, Box<List<T>>),
}

impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Nil => Self::Nil,
            Self::Cons(arg0, arg1) => Self::Cons(arg0.clone(), arg1.clone()),
        }
    }
}

impl<T> List<T> {
    fn new() -> List<T> {
        List::Nil
    }

    fn from_vec(mut xs: Vec<T>) -> List<T> {
        let mut list = List::Nil;

        xs.reverse();
        for elem in xs {
            list = List::Cons(elem, Box::new(list))
        }

        list
    }

    fn pure(x: T) -> List<T> {
        List::Cons(x, Box::new(List::Nil))
    }

    fn append(self, ys: List<T>) -> List<T> {
        match self {
            List::Nil => ys,
            List::Cons(x, xs) => List::Cons(x, Box::new(xs.append(ys))),
        }
    }

    fn and_then<F, U>(self, f: F) -> List<U>
    where
        F: FnOnce(T) -> List<U>,
        F: Clone,
    {
        match self {
            List::Nil => List::Nil,
            List::Cons(x, xs) => f.clone()(x).append(xs.and_then(f)),
        }
    }
}

impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Cons(l0, l1), Self::Cons(r0, r1)) => l0 == r0 && l1 == r1,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<T: Eq> Eq for List<T> {}

#[test]
fn list001() {
    let xs = List::from_vec(vec![1, 3, 5]);
    let ys = List::from_vec(vec![2, 4, 6]);

    let zs = bang!(List::pure(!xs + !ys));
    let zss = List::from_vec(vec![3, 5, 7, 5, 7, 9, 7, 9, 11]);
    assert_eq!(zs, zss);
}
