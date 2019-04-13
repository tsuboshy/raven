pub mod output_method;

pub use output_method::Output;
pub use output_method::OutputMethod;
pub use output_method::OutputMethod::*;

pub use output_method::OutputError;

pub mod local_file;
pub mod s3;
