use criterion::{criterion_group, criterion_main, Criterion};
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
    ];

    let files: Vec<(String, String)> = urls.iter().map(|(name, url)| {
        let resp = ureq::get(url).call().unwrap();
        let mut buf = String::new();
        resp.into_reader().read_to_string(&mut buf).unwrap();
        (name.to_string(), buf)
    }).collect();

    files
}

fn get_test_data() -> &'static Vec<(String, String)> {
    TEST_DATA.get_or_init(load_test_files)
}

fn bench_my_parse(c: &mut Criterion) {
    let test_data = get_test_data();
    c.bench_function("my_parse", |b| {
        b.iter(|| {
            for (_, data) in test_data.iter() {
                let _ = my_parse(data);
            }
        });
    });
}

fn bench_ext_json_parse(c: &mut Criterion) {
    let test_data = get_test_data();
    c.bench_function("ext_json_parse", |b| {
        b.iter(|| {
            for (_, data) in test_data.iter() {
                let _ = ext_json::parse(data);
            }
        });
    });
}

criterion_group!(benches, bench_my_parse, bench_ext_json_parse);
criterion_main!(benches);