use cmdstan::argtree::ArgTree;
use cmdstan::translate::Translate;

fn main() {
    let x = ArgTree::default();
    println!("{:-^80}", "tree");
    println!("{}", x.to_tree().to_string_lossy());
    println!("{:-^80}", "statement");
    println!("{}", x.to_stmt().to_string_lossy());
}
