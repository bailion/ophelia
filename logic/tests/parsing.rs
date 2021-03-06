use ophelia_logic::parse::{Parse, Template};

fn parse_and_check_result(input: &str) {
    let (template, _) = Template::parse(input).expect("first parsing failed");

    let output = template.to_string();

    let (template2, _) = Template::parse(&output).expect("second parsing failed");

    assert_eq!(template, template2);
}

#[test]
fn parsing_regression_1() {
    parse_and_check_result("\u{0}EEEEEEE");
}

#[test]
fn parsing_regression_2() {
    assert!(Template::parse(r#"s\{{{{{{{؉{"'"#).is_err());
}

#[test]
fn parsing_regression_3() {
    assert!(Template::parse(r#"{{""|{{{"|{͸""|"#).is_err());
}

#[test]
fn parsing_regression_4() {
    assert!(Template::parse(r#"|{{%%%%%%%%%{"#).is_err());
}

#[test]
// grr, utf8
fn parsing_regression_5() {
    parse_and_check_result("47241");
}

#[test]
fn parsing_regression_6() {
    assert!(Template::parse(r#"{"{{{z"#).is_err());
}

#[test]
fn parsing_regression_7() {
    assert!(Template::parse(r#"{{(/0{"(()({{(((((/-/{q{{{[{q{{-/{q{{{["#).is_err());
}

#[test]
fn parsing_regression_8() {
    assert!(Template::parse(r#"{{/////</-///</-/{{{{["#).is_err());
}

#[test]
// in this one I confused byte width for display width (very different, oops)
fn parsing_regression_9() {
    parse_and_check_result("!͝X");
}

#[test]
// grr, utf8
fn parsing_regression_10() {
    assert!(Template::parse(r#"{{["["""{"#).is_err());
}

#[test]
// did I mention utf8?
fn parsing_regression_11() {
    let parsed = Template::parse("֓z{{{˦'{{{");
    assert!(parsed.is_err());
}

#[test]
// this one wasn't too bad actually
fn parsing_regression_12() {
    assert!(Template::parse("{{><̦'@").is_err());
}

#[test]
fn parsing_regression_13() {
    assert!(Template::parse("a{{").is_err());
}

#[test]
fn parsing_regression_14() {
    assert!(Template::parse(
        r#"{>%%{{((((񒩎{{{{_[,
        <!DO"#
    )
    .is_err());
}

#[test]
fn parsing_regression_15() {
    assert!(Template::parse("O!!|{{(((((((((((݉'O{{{{{{'w''''''''").is_err())
}

#[test]
fn parsing_regression_16() {
    assert!(Template::parse(r#"5,{98''''DO''D{{----""*{{{O'"#).is_err())
}

#[test]
fn parsing_regression_17() {
    assert!(Template::parse_optional("<{#<1G046!O#")
        .unwrap()
        .0
        .is_none())
}

#[test]
fn parsing_regression_18() {
    assert!(Template::parse("O	{{(((({///{)*{z{{{{{{{{'DO'").is_err());
}

#[test]
fn parsing_regression_19() {
    assert!(Template::parse("{#- --91{#").is_err());
}
