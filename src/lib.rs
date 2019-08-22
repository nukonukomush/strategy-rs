pub mod indicator;

#[no_mangle]
pub extern "C" fn test_fn() -> i32 {
    1234
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
