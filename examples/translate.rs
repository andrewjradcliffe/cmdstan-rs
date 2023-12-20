use cmdstan::translate::*;
use cmdstan::sample::*;
use std::ffi::OsString;
fn main() {
    translations(NutsBuilder::new().build());
    translations(HmcBuilder::new().build());
    translations(SampleAdapt::default());
}


fn translations<T>(x: T)
where T: Translate + std::fmt::Debug
{
    println!("{:-<80}", "");
    println!("{:#?}", x);
    let a = x.to_args();
    println!("{:#?}", a);
    let s = x.to_tree();
    println!("{:#?}", s);
    println!("{}", s.to_string_lossy());
    let s = x.to_stmt();
    println!("{:#?}", s);
    println!("{}", s.to_string_lossy());
}
