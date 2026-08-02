# Benchmark Methodology

## Overview

This document describes the benchmarking methodology used to evaluate the performance of the Rust implementation of TinyExpr and compare it with the original C implementation.

The objective is to measure execution performance while ensuring both implementations execute the same expressions under comparable conditions.

---

# Benchmark Environment

Operating System:
- Windows (current workspace environment)

Processor:
- Recorded during local benchmark runs as needed

Memory:
- Recorded during local benchmark runs as needed

Rust Version:

```bash
rustc --version
```

Compiler Optimization:

```bash
cargo bench --release
```

Original C Compiler:

```bash
gcc -O3
```

---

# Benchmark Tool

Rust benchmarks are implemented using the Criterion benchmarking library.

Criterion was chosen because it provides:

- Statistical analysis
- Automatic warm-up
- Outlier detection
- Confidence intervals
- Stable timing measurements

---

# Benchmark Cases

The following expression categories are benchmarked.

## Basic Arithmetic

Examples

```
2+3

10*25

100/4

7-5
```

---

## Operator Precedence

Examples

```
2+3*4

5*(3+2)

2^8
```

---

## Nested Expressions

Examples

```
((2+3)*(4+5))/3

sqrt((5+3)^2)

sin(cos(1))
```

---

## Built-in Functions

Examples

```
sqrt(25)

sin(pi)

log(100)

pow(2,10)
```

---

## Variable Evaluation

Examples

```
x+y

2*x+3*y

sqrt(x*x+y*y)
```

---

# Metrics Collected

The following metrics are recorded.

- Average execution time
- Median execution time
- Standard deviation
- Throughput
- Memory allocations (if measured)

---

# Benchmark Procedure

Each benchmark follows the same procedure.

1. Compile the expression.
2. Evaluate the compiled expression repeatedly.
3. Record execution statistics.
4. Repeat multiple times.
5. Report Criterion statistics.

All benchmarks are executed in release mode.

```bash
cargo bench
```

---

# Comparison with Original C Version

The original TinyExpr implementation is compiled using compiler optimizations.

```bash
gcc -O3
```

Equivalent benchmark expressions are executed in both implementations.

Performance comparisons include:

- Parsing speed
- Evaluation speed
- Total execution time

---

# Notes

The objective of this benchmark is not to outperform the original implementation, but to demonstrate that the Rust implementation preserves functionality while maintaining competitive performance.

Any performance differences should be interpreted together with Rust's safety guarantees, ownership model, and improved maintainability.

---

# Results

(To be completed after running benchmarks.)

| Benchmark | C | Rust | Difference |
|-----------|---|------|------------|
| Basic Arithmetic | | | |
| Nested Expressions | | | |
| Functions | | | |
| Variables | | | |

---

# Conclusion

(To be completed after benchmarking.)

Summarize the observed performance and discuss any significant findings.