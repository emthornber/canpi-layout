mod test_lpv {
    use canpi_layout::LPV;

    use glob::glob_with;
    use std::fs;
    use std::path::Path;

    const LAYOUT_PATH: &str = "tests/layouts";

    #[test]
    fn list_layouts() {
        let layout_path = Path::new(LAYOUT_PATH);

        let entries = match fs::read_dir(layout_path) {
            Ok(entries) => entries,
            Err(_) => panic!("Cannot read directory {:?}", layout_path),
        };

        for entry_result in entries {
            match entry_result {
                Ok(entry) => println!("Name: {:?}", entry.path()),
                Err(_) => panic!("Duff DirEntry result"),
            }
        }
    }

    #[test]
    /// Test reading directory of layout definitions
    fn read_layouts() {
        let layout_path = Path::new(LAYOUT_PATH);
        assert!(layout_path.is_dir());
        let lpv = LPV::new(layout_path);
        println!("Length: {:?}", lpv.lpv.len());
        assert_eq!(lpv.lpv.len(), 2);
    }

    #[test]
    fn load_layouts() {
        let pattern_result = Path::new(LAYOUT_PATH).join("*.json").to_path_buf();
        match pattern_result.to_str() {
            Some(pattern) => {
                println!("Pattern: {:?}", pattern);
                for entry in glob_with(pattern, LPV::match_options()).unwrap().flatten() {
                    println!("Entry: {:?}", entry);
                }
            }
            None => panic!("No pattern created"),
        }
    }
}
