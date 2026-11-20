/// Generate a `#[test]` fn that runs every unit under `cases/<name>`.
///
/// With `--features bless`, snapshots are regenerated before checking.
#[macro_export]
macro_rules! unit_test {
    ($name:ident) => {
        #[test]
        fn $name() -> ::anyhow::Result<()> {
            let dir = ::std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join(stringify!($name));
            let mut units = ::lsp_test::TestUnits::new(&dir)?;

            #[cfg(feature = "bless")]
            units.bless()?;

            units.check()
        }
    };
}
