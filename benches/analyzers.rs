// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::hint::black_box;

use cargo_quality::{
    analyzer::Analyzer,
    analyzers::{format_args::FormatArgsAnalyzer, path_import::PathImportAnalyzer}
};
use criterion::{Criterion, criterion_group, criterion_main};
use syn::File;

fn parse_sample_code(code: &str) -> File {
    syn::parse_file(code).unwrap()
}

fn bench_path_import_analyzer(c: &mut Criterion) {
    let analyzer = PathImportAnalyzer::new();

    let simple_code = parse_sample_code(
        r#"
        fn main() {
            let content = std::fs::read_to_string("file.txt");
        }
        "#
    );

    let complex_code = parse_sample_code(
        r#"
        fn main() {
            let content = std::fs::read_to_string("file.txt");
            let data = std::io::stdin();
            let size = core::mem::size_of::<u32>();
            let fmt = alloc::format::format(format_args!("test"));
            let v = Vec::new();
            let s = String::from("hello");
            let opt = Option::Some(42);
            let max = u32::MAX;
        }
        "#
    );

    c.bench_function("path_import_simple", |b| {
        b.iter(|| analyzer.analyze(black_box(&simple_code)))
    });

    c.bench_function("path_import_complex", |b| {
        b.iter(|| analyzer.analyze(black_box(&complex_code)))
    });
}

fn bench_format_args_analyzer(c: &mut Criterion) {
    let analyzer = FormatArgsAnalyzer::new();

    let simple_code = parse_sample_code(
        r#"
        fn main() {
            println!("Hello {}", name);
        }
        "#
    );

    let complex_code = parse_sample_code(
        r#"
        fn main() {
            println!("Values: {} {} {}", a, b, c);
            format!("Data: {} {} {} {}", x, y, z, w);
            print!("Test: {} {}", 1, 2);
            writeln!(buf, "Output: {} {} {} {} {}", a, b, c, d, e).unwrap();
        }
        "#
    );

    c.bench_function("format_args_simple", |b| {
        b.iter(|| analyzer.analyze(black_box(&simple_code)))
    });

    c.bench_function("format_args_complex", |b| {
        b.iter(|| analyzer.analyze(black_box(&complex_code)))
    });
}

fn bench_file_parsing(c: &mut Criterion) {
    let code = r#"
    use std::fs;
    use std::io;

    fn main() {
        let content = std::fs::read_to_string("file.txt");
        let data = std::io::stdin();
        println!("Values: {} {} {}", 1, 2, 3);
    }
    "#;

    c.bench_function("syn_parse_file", |b| {
        b.iter(|| syn::parse_file(black_box(code)))
    });
}

criterion_group!(
    benches,
    bench_path_import_analyzer,
    bench_format_args_analyzer,
    bench_file_parsing
);
criterion_main!(benches);
