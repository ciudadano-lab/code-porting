use tinyexpr::{te_compile, te_eval_expr, TeVariable, TE_FUNCTION2};
use std::os::raw::c_void;

/* An example of calling a C function. */
unsafe fn my_sum(a: f64, b: f64) -> f64 {
    println!("Called C function with {} and {}.", a, b);
    a + b
}

fn main() {
    let vars = [TeVariable {
        name: "mysum",
        address: my_sum as *const c_void,
        type_: TE_FUNCTION2,
        context: std::ptr::null_mut(),
    }];

    let expression = "mysum(5, 6)";
    println!("Evaluating:\n\t{}", expression);

    match te_compile(expression, &vars) {
        Ok(n) => {
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