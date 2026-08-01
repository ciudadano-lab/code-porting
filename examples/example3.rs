// Port of example3.c

use tinyexpr::Variable;

// An example of calling a Rust function.
fn my_sum(a: f64, b: f64) -> f64 {
    println!("Called Rust function with {a:.6} and {b:.6}.");
    a + b
}

fn main() {
    let vars = [Variable::function2("mysum", my_sum, false)];

    let expression = "mysum(5, 6)";
    println!("Evaluating:\n\t{expression}");

    match tinyexpr::te_compile(expression, &vars) {
        Ok(n) => {
            let r = n.eval();
            println!("Result:\n\t{r:.6}");
        }
        Err(err) => {
            println!("\t{:>width$}^\nError near here", "", width = err.saturating_sub(1));
        }
    }
}
