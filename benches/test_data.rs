use std::{fs::File, io::Read};

pub const MEDIUM_PATTERN: &str = "fooo--foo-----fo";
pub const MEDIUM_TEXT: &str = "foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--foo-----fo--foo-f--fooo--";

pub fn get_cia_text() -> String {
    get_text("benches/test_files/world192.txt")
}

pub fn get_ecoli_text() -> String {
    get_text("benches/test_files/E.coli")
}

fn get_text(path: &str) -> String {
    let mut file = File::open(path).expect("huh, where did the file go?");
    let mut text = String::new();
    file.read_to_string(&mut text)
        .expect("uh, failed reading file...");

    text
}
