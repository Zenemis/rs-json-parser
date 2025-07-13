use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::OnceLock;
use std::io::Read;

use json_parser::parse as my_parse;
use json as ext_json;

static TEST_DATA: OnceLock<Vec<(String, String)>> = OnceLock::new();

fn load_test_files() -> Vec<(String, String)> {
    let urls = [
        (
            "englishmonarchs",
            "https://mysafeinfo.com/api/data?list=englishmonarchs&format=json"
        ),
        (
            "freetestdata",
            "https://freetestdata.com/wp-content/uploads/2023/04/2.4KB_JSON-File_FreeTestData.json"
        ),
        (
            "sample.cat",
            "https://disk.sample.cat/samples/json/sample-1.json"
        ),
        (
            "large-file-github",
            "https://raw.githubusercontent.com/json-iterator/test-data/master/large-file.json"
        )
    ];

    urls.iter().map(|(name, url)| {
        println!("Downloading {name} from {url} ...");
        let resp = ureq::get(url).call().unwrap();
        let mut buf = String::new();
        resp.into_reader().read_to_string(&mut buf).unwrap();
        println!("Downloaded {name} ({} bytes)", buf.len());
        (name.to_string(), buf)
    }).collect()
}

fn get_test_data() -> &'static Vec<(String, String)> {
    TEST_DATA.get_or_init(load_test_files)
}

fn bench_parsers_on_files(c: &mut Criterion) {
    let test_data = get_test_data();

    for (name, data) in test_data.iter() {
        // Benchmark my_parse parser
        c.bench_with_input(
            BenchmarkId::new("my_parse", name),
            data,
            |b, data| {
                b.iter(|| {
                    let _ = my_parse(data);
                });
            },
        );

        // Benchmark ext_json parser
        c.bench_with_input(
            BenchmarkId::new("ext_json_parse", name),
            data,
            |b, data| {
                b.iter(|| {
                    let _ = ext_json::parse(data);
                });
            },
        );
    }
}

criterion_group!(benches, bench_parsers_on_files);
criterion_main!(benches);