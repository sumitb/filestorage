use filestorage_core::greet;

#[test]
fn greet_formats_expected_message() {
    let name = "Rustacean";
    let greeting = greet(name);
    assert_eq!(greeting, "Hello, Rustacean!");
}
