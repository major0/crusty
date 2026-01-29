#[test]
pub fn test_function() -> i32 {
    return 42;
}

macro_rules! add {
    ($a:expr, $b:expr) => {{
        $a + $b 
    }};
}

#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
