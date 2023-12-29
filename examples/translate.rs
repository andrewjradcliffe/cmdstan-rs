use cmdstan::argument_tree::ArgumentTree;
use cmdstan::translate::Translate;

fn main() {
    let x = ArgumentTree::default();
    println!("{:-^80}", "tree");
    println!("{}", x.to_tree().to_string_lossy());
    println!("{:-^80}", "statement");
    println!("{}", x.to_stmt().to_string_lossy());
}
