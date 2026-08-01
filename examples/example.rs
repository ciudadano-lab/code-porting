// Port of example.c

fn main() {
    let c = "sqrt(5^2+7^2+11^2+(8-2)^2)";
    let r = tinyexpr::te_interp(c).unwrap_or(f64::NAN);
    println!("The expression:\n\t{c}\nevaluates to:\n\t{r:.6}");
}
