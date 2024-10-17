use std::collections::HashMap;

use fastrace::collector::Reporter;

pub struct RConsoleReporter;

impl Reporter for RConsoleReporter {
    fn report(&mut self, spans: Vec<fastrace::prelude::SpanRecord>) {
        let mut result: HashMap<String, (u64, u64)> = HashMap::new();
        for span in spans {
            let k = span.name.to_string();

            if let Some((count, duration)) = result.get_mut(&k) {
                *count += 1;
                *duration += span.duration_ns;
            } else {
                result.insert(span.name.into(), (1, span.duration_ns));
            }
        }

        let total = result.remove("root");

        let mut result_vec: Vec<_> = result.into_iter().collect();
        result_vec.sort_by_key(|x| u64::MAX - x.1 .1); // sort by duration time

        for (name, (count, duration)) in result_vec {
            savvy::r_eprintln!(
                "{name}: {count} times ({:.3} ms)",
                duration as f64 / 1_000_000.0
            )
        }

        if let Some((_, duration)) = total {
            savvy::r_eprintln!("[Total] {:.3} ms", duration as f64 / 1_000_000.0);
        }
    }
}
