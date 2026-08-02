use tinyexpr::{te_compile, te_eval_expr, TeVariable, TE_VARIABLE};
use std::env;
use std::os::raw::c_void;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: example2 \"expression\"");
        return;
    }
    let expression = &args[1];
    println!("Evaluating:\n\t{}", expression);

    /* This shows an example where the variables
     * x and y are bound at eval-time. */
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let vars = [
        TeVariable {
            name: "x",
            address: &x as *const f64 as *const c_void,
            type_: TE_VARIABLE,
            context: std::ptr::null_mut(),
        },
        TeVariable {
            name: "y",
            address: &y as *const f64 as *const c_void,
            type_: TE_VARIABLE,
            context: std::ptr::null_mut(),
        },
    ];

    /* This will compile the expression and check for errors. */
    match te_compile(expression, &vars) {
        Ok(n) => {
            /* The variables can be changed here, and eval can be called as many
             * times as you like. This is fairly efficient because the parsing has
             * already been done. */
            x = 3.0;
            y = 4.0;
            let r = te_eval_expr(&n);
            println!("Result:\n\t{}", r);
            // te_free is automatic via Drop of Box<Expr>
        }
        Err(err) => {
            /* Show the user where the error is at. */
            println!("\t{:>width$}^\nError near here", "", width = (err - 1) as usize);
        }
    }
}