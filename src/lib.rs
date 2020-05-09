//!

#![deny(
    missing_docs,
    bare_trait_objects,
    missing_copy_implementations,
    single_use_lifetimes,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    variant_size_differences,
    unsafe_code,
    trivial_casts,
    // missing_debug_implementations,
    // 把所有warnings级别的改为deny 
    warnings,
    clippy::all,
    clippy::correctness,
    clippy::restriction,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::cargo,
    clippy::nursery ,
    clippy::pedantic
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::wildcard_enum_match_arm,
    // clippy::integer_arithmetic, // 直接整数都不能用运算符了。。得用checked函数，酌情开启吧，可以防止除0异常
    // clippy::use_self, // 泛型会出问题
    // clippy::default_trait_access, // 宏里面使用Default::default
)]

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
