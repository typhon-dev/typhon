//! Benchmarks for the Typhon parser.
//!
//! This module contains comprehensive benchmarks for parser performance:
//! - Simple constructs (functions, expressions)
//! - Complex constructs (classes, control flow)
//! - Scaling with increasing code size
//! - AST traversal operations
//! - Node allocation patterns
//!
//! Run with: `cargo bench --package typhon-parser`

use std::sync::Arc;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use typhon_parser::parser::Parser;
use typhon_source::types::{FileID, SourceManager};

// Test data for benchmarks
const SIMPLE_FUNCTION: &str = r"
def add(a, b):
    return a + b
";

const FIBONACCI: &str = r"
def fibonacci(n):
    if n <= 1:
        return n
    else:
        return fibonacci(n-1) + fibonacci(n-2)

result = fibonacci(10)
print(result)
";

const CLASS_DEFINITION: &str = r"
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def distance(self, other):
        dx = self.x - other.x
        dy = self.y - other.y
        return (dx * dx + dy * dy) ** 0.5
";

const COMPLEX_EXPRESSIONS: &str = r"
# Complex arithmetic and logical expressions
result = ((a + b) * (c - d) / e) ** 2 + f % g
condition = (x > 0 and y < 100) or (z == 42 and w != 0)
nested = [i * 2 for i in range(10) if i % 2 == 0]
";

const CONTROL_FLOW: &str = r#"
def process(items):
    for item in items:
        if item > 0:
            if item % 2 == 0:
                print(f"Even positive: {item}")
            else:
                print(f"Odd positive: {item}")
        elif item < 0:
            print(f"Negative: {item}")
        else:
            continue

    while len(items) > 0:
        items.pop()
"#;

const COMPREHENSIVE: &str = r#"
# Comprehensive test with multiple constructs
from typing import List, Dict, Optional
import math

class DataProcessor:
    """A class for processing data efficiently."""

    def __init__(self, data: List[int]):
        self.data = data
        self.cache: Dict[int, int] = {}

    def process(self) -> Optional[int]:
        result = 0
        for i, value in enumerate(self.data):
            if value in self.cache:
                result += self.cache[value]
            else:
                computed = self._compute(value)
                self.cache[value] = computed
                result += computed

        return result if result > 0 else None

    def _compute(self, n: int) -> int:
        if n <= 1:
            return n
        return self._compute(n-1) + self._compute(n-2)

def main():
    processor = DataProcessor([1, 2, 3, 4, 5])
    result = processor.process()
    print(f"Result: {result}")

if __name__ == "__main__":
    main()
"#;

/// Benchmark parsing a simple function
fn bench_simple_function(crit: &mut Criterion) {
    let source_manager = Arc::new(SourceManager::new());
    let file_id = FileID::new(0);

    let _ = crit.bench_function("parse_simple_function", |bencher| {
        bencher.iter(|| {
            let mut parser =
                Parser::new(black_box(SIMPLE_FUNCTION), file_id, source_manager.clone());
            parser.parse_module()
        });
    });
}

/// Benchmark parsing fibonacci function
fn bench_fibonacci(crit: &mut Criterion) {
    let source_manager = Arc::new(SourceManager::new());
    let file_id = FileID::new(0);

    let _ = crit.bench_function("parse_fibonacci", |bencher| {
        bencher.iter(|| {
            let mut parser = Parser::new(black_box(FIBONACCI), file_id, source_manager.clone());
            parser.parse_module()
        });
    });
}

/// Benchmark parsing class definition
fn bench_class_definition(crit: &mut Criterion) {
    let source_manager = Arc::new(SourceManager::new());
    let file_id = FileID::new(0);

    let _ = crit.bench_function("parse_class_definition", |bencher| {
        bencher.iter(|| {
            let mut parser =
                Parser::new(black_box(CLASS_DEFINITION), file_id, source_manager.clone());
            parser.parse_module()
        });
    });
}

/// Benchmark parsing complex expressions
fn bench_complex_expressions(crit: &mut Criterion) {
    let source_manager = Arc::new(SourceManager::new());
    let file_id = FileID::new(0);

    let _ = crit.bench_function("parse_complex_expressions", |bencher| {
        bencher.iter(|| {
            let mut parser =
                Parser::new(black_box(COMPLEX_EXPRESSIONS), file_id, source_manager.clone());
            parser.parse_module()
        });
    });
}

/// Benchmark parsing control flow statements
fn bench_control_flow(crit: &mut Criterion) {
    let source_manager = Arc::new(SourceManager::new());
    let file_id = FileID::new(0);

    let _ = crit.bench_function("parse_control_flow", |bencher| {
        bencher.iter(|| {
            let mut parser = Parser::new(black_box(CONTROL_FLOW), file_id, source_manager.clone());
            parser.parse_module()
        });
    });
}

/// Benchmark parsing comprehensive code
fn bench_comprehensive(crit: &mut Criterion) {
    let source_manager = Arc::new(SourceManager::new());
    let file_id = FileID::new(0);

    let mut group = crit.benchmark_group("parse_comprehensive");
    let _ = group.throughput(Throughput::Bytes(COMPREHENSIVE.len() as u64));
    let _ = group.bench_function("comprehensive", |bencher| {
        bencher.iter(|| {
            let mut parser = Parser::new(black_box(COMPREHENSIVE), file_id, source_manager.clone());
            parser.parse_module()
        });
    });

    group.finish();
}

/// Benchmark parsing with varying code sizes
fn bench_scaling(crit: &mut Criterion) {
    let source_manager = Arc::new(SourceManager::new());
    let file_id = FileID::new(0);

    let mut group = crit.benchmark_group("parse_scaling");

    for size in &[10, 50, 100, 500] {
        // Generate code with N simple functions
        let code = (0..*size).fold(String::new(), |acc, idx| {
            format!("{acc}\n\ndef func_{idx}(x):\n    return x * {idx}\n")
        });

        let _ = group.throughput(Throughput::Bytes(code.len() as u64));
        let _ =
            group.bench_with_input(BenchmarkId::from_parameter(size), &code, |bencher, code| {
                bencher.iter(|| {
                    let mut parser = Parser::new(black_box(code), file_id, source_manager.clone());
                    parser.parse_module()
                });
            });
    }

    group.finish();
}

/// Benchmark AST traversal operations
fn bench_ast_traversal(crit: &mut Criterion) {
    let source_manager = Arc::new(SourceManager::new());
    let file_id = FileID::new(0);

    // Parse once to get AST - use simple function for faster parsing
    let mut parser = Parser::new(SIMPLE_FUNCTION, file_id, source_manager);
    let module_id = parser.parse_module().expect("Failed to parse");

    // Get AST reference for benchmarking
    let ast = parser.ast();

    let _ = crit.bench_function("ast_pre_order_traversal", |bencher| {
        bencher.iter(|| {
            let nodes = black_box(ast.collect_nodes_pre_order(module_id));
            nodes.len()
        });
    });

    let _ = crit.bench_function("ast_post_order_traversal", |bencher| {
        bencher.iter(|| {
            let nodes = black_box(ast.collect_nodes_post_order(module_id));
            nodes.len()
        });
    });
}

/// Benchmark node allocation patterns
fn bench_node_allocation(crit: &mut Criterion) {
    let source_manager = Arc::new(SourceManager::new());
    let file_id = FileID::new(0);

    let mut group = crit.benchmark_group("node_allocation");

    // Measure allocation overhead for different constructs
    let _ = group.bench_function("allocate_simple_function", |bencher| {
        bencher.iter(|| {
            let mut parser =
                Parser::new(black_box(SIMPLE_FUNCTION), file_id, source_manager.clone());
            parser.parse_module()
        });
    });

    let _ = group.bench_function("allocate_complex_class", |bencher| {
        bencher.iter(|| {
            let mut parser =
                Parser::new(black_box(CLASS_DEFINITION), file_id, source_manager.clone());
            parser.parse_module()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_function,
    bench_fibonacci,
    bench_class_definition,
    bench_complex_expressions,
    bench_control_flow,
    bench_comprehensive,
    bench_scaling,
    bench_ast_traversal,
    bench_node_allocation,
);
criterion_main!(benches);
