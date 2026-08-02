use tinyexpr::te_interp;

fn main() {
    let c = "sqrt(5^2+7^2+11^2+(8-2)^2)";
    let r = te_interp(c).unwrap_or(f64::NAN);
    println!("The expression:\n\t{}\nevaluates to:\n\t{}", c, r);
}