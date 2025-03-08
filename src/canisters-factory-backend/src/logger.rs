// src/logger.rs
#[macro_export]
macro_rules! debug_log {  
    ($($arg:tt)*) => {
        ic_cdk::api::print(format!($($arg)*));
    }
}