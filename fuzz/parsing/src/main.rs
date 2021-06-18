use honggfuzz::fuzz;
use ophelia_logic::parse::{Parse, Template};
use std::str;

fn main() {
    loop {
        fuzz!(|input: &[u8]| {
            if let Ok(input) = str::from_utf8(input) {
                if let Ok((template, _)) = Template::parse(input) {
                    let output = template.to_string();

                    let (parsed, _) = Template::parse(&output).unwrap();

                    assert_eq!(template, parsed);
                }
            }
        })
    }
}
