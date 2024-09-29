use std::fmt::{Debug, Display, Formatter};
use std::fs::read_to_string;
use std::thread::sleep;
use std::time::Duration;
use chrono::Local;

//
// THIS IS PEAK RUST
//

macro_rules! my_print {
    ($str:expr) => {
        println!($str)
    };
}

macro_rules! calculate {
    ($num:expr, $op:tt, $num2:expr) => { $num $op $num2 };
}

macro_rules! repeat_n {
    ($n:expr, $thing:stmt) => {{
        for _ in 0..$n {
            $thing
        }
    }};
}

#[cfg(debug_assertions)]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

macro_rules! create_point_struct {
    ($name:ident) => {
        #[derive(Debug)]
        struct $name {
            x: f32,
            y: f32,
        }

        impl $name {
            fn new(x: f32, y: f32) -> Self {
                Self {
                    x,
                    y,
                }
            }
        }
    };
}

macro_rules! test_point_struct {
    ($name:ident, $x:expr, $y:expr) => {
        let point = $name::new(1.0, 2.0);
        debug_log!("{:?}", point);
    };
}

create_point_struct!(Point);
create_point_struct!(OtherPoint);
create_point_struct!(AnotherPoint);

macro_rules! generate_getters {
    ($name:ident, $getter:ident -> $ty:ty) => {
            fn $getter(&self) -> $ty {
                self.$getter
            }
    };
    ($name:ident, $getter:ident -> $ty:ty, $($getters:ident -> $tys:ty),+) => {
        impl $name {
            generate_getters!($name, $getter -> $ty);
            generate_getters!($name, $($getters -> $tys),+);
        }
    };
}

generate_getters!(Point, x -> f32, y -> f32);

macro_rules! sum_all {
    ($a:expr, $b:expr) => {
        $a + $b
    };
    ($a:expr, $($rest:expr),+) => {
        $a + sum_all!($($rest),+)
    }
}

macro_rules! try_or_log {
    ($func:expr) => {
        match $func {
            Ok(ok_result) => ok_result,
            Err(e) => panic!("{}", e),
        }
    };
}

macro_rules! benchmark {
    ($code:block) => {
        let start = Local::now();
        println!("Benchmark started: {}", start);
        $code
        let end = Local::now();
        println!("Benchmark ended: {}", end);
        println!("Benchmarked code took: {}", end - start);
    };
}

// HTML DSL

#[derive(Debug)]
struct HTMLElement<'a> {
    name: &'a str,
    inner: Vec<HTMLElement<'a>>,
    inner_text: String,
}

impl<'a> HTMLElement<'a> {
    fn inner_fmt(&self, f: &mut Formatter<'_>, padding: usize) -> std::fmt::Result {
        write!(f, "{}<{}>\n", "  ".repeat(padding), self.name)?;
        for inner in &self.inner {
            inner.inner_fmt(f, padding + 1)?
        }
        if !self.inner_text.is_empty() {
            write!(f, "{}{}\n", "  ".repeat(padding), self.inner_text)?;
        }
        write!(f, "{}</{}>\n", "  ".repeat(padding), self.name)
    }
}

impl<'a> Display for HTMLElement<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner_fmt(f, 0)
    }
}

macro_rules! html {
    () => {
        html!("")
    };
    ($text:expr) => {
        HTMLElement {
            name: "html",
            inner: vec![],
            inner_text: text.to_string(),
        }
    };
    ($( $element:ident ! $args:tt )*) => {
        HTMLElement {
            name: "html",
            inner: vec![ $( element! { $element! $args } ),* ],
            inner_text: "".to_string(),
        }
    };
}

macro_rules! element {
    ($name:ident! { $($inner:tt)* }) => {
        HTMLElement {
            name: stringify!($name),
            inner: vec![element! {$($inner)*}],
            inner_text: "".to_string(),
        }
    };
    ($name:ident!($text:expr)) => {
        HTMLElement {
            name: stringify!($name),
            inner: vec![],
            inner_text: $text.to_string(),
        }
    };
    ($name:ident!()) => {
        HTMLElement {
            name: stringify!($name),
            inner: vec![],
            inner_text: "".to_string(),
        }
    };
}

fn main() {
    my_print!("Meow!");
    
    println!("5 + 5 = {}", calculate!(5, +, 5));
    
    println!("Printing mew 5 times:");
    repeat_n!(5, println!("mew"));

    debug_log!("WE ARE IN DEBUG MODE!! {} {}", "mew", "mew");

    test_point_struct!(Point, 1.0, 2.0);
    test_point_struct!(OtherPoint, 1.0, 2.0);
    test_point_struct!(AnotherPoint, 1.0, 2.0);

    let point = Point::new(1.0, 2.0);
    let x = point.x();
    let y = point.y();
    assert_eq!(x, 1.0);
	assert_eq!(y, 2.0);

    assert_eq!(sum_all!(1, 2, 3, 4, 5), 1 + 2 + 3 + 4 + 5);

    let meow = try_or_log!(read_to_string("src/main.rs"));

    benchmark!({
        sleep(Duration::from_secs(1));
    });

    let page = html! {
        head! {
            title!("My page")
        }
        body! {
            div! {
                p!("This is a paragraph")
            }
        }
        footer!()
    };
    println!("Debugged Page: {:?}", page);
    println!("My HTML Page: {}", page);
}
