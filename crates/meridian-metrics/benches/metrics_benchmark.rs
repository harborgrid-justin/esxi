//! Benchmarks for the Meridian Metrics system.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use meridian_metrics::{Counter, Gauge, HistogramMetric, MetricsCollector, Summary};
use std::sync::Arc;

fn bench_counter_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("counter");

    let counter = Counter::new("test_counter", "Test counter");

    group.bench_function("inc", |b| {
        b.iter(|| {
            counter.inc();
        });
    });

    group.bench_function("add", |b| {
        b.iter(|| {
            counter.add(black_box(5));
        });
    });

    group.bench_function("get", |b| {
        b.iter(|| {
            black_box(counter.get());
        });
    });

    group.finish();
}

fn bench_gauge_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("gauge");

    let gauge = Gauge::new("test_gauge", "Test gauge");

    group.bench_function("set", |b| {
        b.iter(|| {
            gauge.set(black_box(42.0));
        });
    });

    group.bench_function("add", |b| {
        b.iter(|| {
            gauge.add(black_box(1.5));
        });
    });

    group.bench_function("get", |b| {
        b.iter(|| {
            black_box(gauge.get());
        });
    });

    group.finish();
}

fn bench_histogram_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("histogram");

    let histogram = HistogramMetric::new("test_histogram", "Test histogram").unwrap();

    group.bench_function("observe", |b| {
        b.iter(|| {
            histogram.observe(black_box(123)).unwrap();
        });
    });

    // Pre-populate for stats benchmarks
    for i in 0..1000 {
        histogram.observe(i).unwrap();
    }

    group.bench_function("percentile", |b| {
        b.iter(|| {
            black_box(histogram.percentile(black_box(0.95)));
        });
    });

    group.bench_function("mean", |b| {
        b.iter(|| {
            black_box(histogram.mean());
        });
    });

    group.finish();
}

fn bench_summary_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("summary");

    let summary = Summary::new("test_summary", "Test summary");

    group.bench_function("observe", |b| {
        b.iter(|| {
            summary.observe(black_box(45.6));
        });
    });

    // Pre-populate for stats benchmarks
    for i in 0..1000 {
        summary.observe(i as f64);
    }

    group.bench_function("get_stats", |b| {
        b.iter(|| {
            black_box(summary.get_stats());
        });
    });

    group.finish();
}

fn bench_collector_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("collector");

    let collector = Arc::new(MetricsCollector::default());

    group.bench_function("register_counter", |b| {
        let mut i = 0;
        b.iter(|| {
            i += 1;
            let name = format!("counter_{}", i);
            collector.register_counter(&name, "Test").unwrap();
        });
    });

    // Pre-populate with metrics
    for i in 0..100 {
        collector.register_counter(&format!("pre_counter_{}", i), "Test").unwrap();
        collector.register_gauge(&format!("pre_gauge_{}", i), "Test").unwrap();
    }

    group.bench_function("collect_snapshots", |b| {
        b.iter(|| {
            black_box(collector.collect_snapshots());
        });
    });

    group.bench_function("metrics_count", |b| {
        b.iter(|| {
            black_box(collector.metrics_count());
        });
    });

    group.finish();
}

fn bench_concurrent_counter_access(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let mut group = c.benchmark_group("concurrent");

    let counter = Arc::new(Counter::new("concurrent_counter", "Test"));

    for thread_count in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("counter_threads", thread_count),
            thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let mut handles = vec![];

                    for _ in 0..thread_count {
                        let counter = Arc::clone(&counter);
                        let handle = thread::spawn(move || {
                            for _ in 0..1000 {
                                counter.inc();
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_label_overhead(c: &mut Criterion) {
    use std::collections::HashMap;

    let mut group = c.benchmark_group("labels");

    let collector = Arc::new(MetricsCollector::default());

    group.bench_function("counter_no_labels", |b| {
        let mut i = 0;
        b.iter(|| {
            i += 1;
            let counter = collector
                .register_counter(&format!("no_labels_{}", i), "Test")
                .unwrap();
            counter.inc();
        });
    });

    group.bench_function("counter_with_labels", |b| {
        let mut i = 0;
        b.iter(|| {
            i += 1;
            let mut labels = HashMap::new();
            labels.insert("tenant".to_string(), "test".to_string());
            labels.insert("region".to_string(), "us-west".to_string());

            let counter = collector
                .register_counter_with_labels(&format!("with_labels_{}", i), "Test", labels)
                .unwrap();
            counter.inc();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_counter_operations,
    bench_gauge_operations,
    bench_histogram_operations,
    bench_summary_operations,
    bench_collector_operations,
    bench_concurrent_counter_access,
    bench_label_overhead,
);

criterion_main!(benches);
