#[cfg(test)]
mod tests {
    use filestorage_core::greet;

    #[test]
    fn greeting_mentions_name() {
        assert!(greet("workspace").contains("workspace"));
    }
}
