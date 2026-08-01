// Port of example2.c

use tinyexpr::Variable;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: example2 \"expression\"");
        return;
    }

    let expression = &args[1];
    println!("Evaluating:\n\t{expression}");

    // This shows an example where the variables x and y are bound at
    // eval-time.
    let mut x = 0.0f64;
    let mut y = 0.0f64;
    let vars = [Variable::variable("x", &x), Variable::variable("y", &y)];

    // This will compile the expression and check for errors.
    match tinyexpr::te_compile(expression, &vars) {
        Ok(n) => {
            // The variables can be changed here, and eval can be called as
            // many times as you like. This is fairly efficient because the
            // parsing has already been done.
            #[allow(unused_assignments)]
            {
                x = 3.0;
                y = 4.0;
            }
            let r = n.eval();
            println!("Result:\n\t{r:.6}");
        }
        Err(err) => {
            // Show the user where the error is at.
            println!("\t{:>width$}^\nError near here", "", width = err.saturating_sub(1));
        }
    }
}
