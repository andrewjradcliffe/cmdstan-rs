use cmdstan::translate::*;
use cmdstan::sample::*;
use std::ffi::OsString;
fn main() {
    translations(NutsBuilder::new().build());
    translations(HmcBuilder::new().build());
    translations(SampleAdapt::default());
}

fn translations<T>(x: T)
where T: Translate<Args, Output = Vec<OsString>> + Translate<Stmt, Output = OsString> + Translate<Tree, Output = OsString> + std::fmt::Debug
{
    println!("{:-<80}", "");
    println!("{:#?}", x);
    let a = Translate::<Args>::translate(&x);
    println!("{:#?}", a);
    let s = Translate::<Tree>::translate(&x);
    println!("{:#?}", s);
    println!("{}", s.to_string_lossy());
    let s = Translate::<Stmt>::translate(&x);
    println!("{:#?}", s);
    println!("{}", s.to_string_lossy());
}
