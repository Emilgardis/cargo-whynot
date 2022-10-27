#[test]
#[ignore]
fn cli_tests() {
    trycmd::TestCases::new()
        .register_bin(
            "cargo",
            trycmd::schema::Bin::Path(std::env::var("CARGO").unwrap().into()),
        )
        .register_bin("cargo-whynot", trycmd::cargo::cargo_bin!("cargo-whynot"))
        .register_bin("whynot", trycmd::cargo::cargo_bin!("whynot"))
        .case("tests/cmd/*.trycmd");
}
