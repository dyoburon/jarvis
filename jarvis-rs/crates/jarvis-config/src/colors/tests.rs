//! Tests for color parsing and validation.

use super::*;

#[test]
fn parse_hex_6_digit() {
    let c = parse_color("#00d4ff").unwrap();
    assert_eq!(c, Color::from_rgba(0, 212, 255, 255));
}

#[test]
fn parse_hex_8_digit() {
    let c = parse_color("#00d4ff80").unwrap();
    assert_eq!(c, Color::from_rgba(0, 212, 255, 128));
}

#[test]
fn parse_hex_3_digit() {
    let c = parse_color("#f00").unwrap();
    assert_eq!(c, Color::from_rgba(255, 0, 0, 255));
}

#[test]
fn parse_rgba_float_alpha() {
    let c = parse_color("rgba(0,212,255,0.12)").unwrap();
    assert_eq!(c.r, 0);
    assert_eq!(c.g, 212);
    assert_eq!(c.b, 255);
    // 0.12 * 255 = 30.6 -> 31
    assert_eq!(c.a, 31);
}

#[test]
fn parse_rgba_half_alpha() {
    let c = parse_color("rgba(0,0,0,0.5)").unwrap();
    assert_eq!(c.a, 128);
}

#[test]
fn parse_rgba_full_alpha() {
    let c = parse_color("rgba(255,255,255,1.0)").unwrap();
    assert_eq!(c, Color::from_rgba(255, 255, 255, 255));
}

#[test]
fn parse_rgba_zero_alpha() {
    let c = parse_color("rgba(0,0,0,0.0)").unwrap();
    assert_eq!(c.a, 0);
}

#[test]
fn parse_rgba_with_spaces() {
    let c = parse_color("rgba( 100 , 180 , 255 , 0.9 )").unwrap();
    assert_eq!(c.r, 100);
    assert_eq!(c.g, 180);
    assert_eq!(c.b, 255);
    // 0.9 * 255 = 229.5 -> 230
    assert_eq!(c.a, 230);
}

#[test]
fn parse_color_invalid_format() {
    assert!(parse_color("not-a-color").is_err());
    assert!(parse_color("").is_err());
    assert!(parse_color("#xyz").is_err());
    assert!(parse_color("rgba(300,0,0,1.0)").is_err());
}

#[test]
fn validate_color_accepts_valid() {
    assert!(validate_color("#00d4ff"));
    assert!(validate_color("#00d4ff80"));
    assert!(validate_color("#f00"));
    assert!(validate_color("rgba(0,212,255,0.12)"));
    assert!(validate_color("rgba(255,255,255,1.0)"));
}

#[test]
fn validate_color_rejects_invalid() {
    assert!(!validate_color(""));
    assert!(!validate_color("not-a-color"));
    assert!(!validate_color("#12345")); // 5 digits
    assert!(!validate_color("rgb(10,20)"));
}

#[test]
fn parse_all_default_colors() {
    // Verify all default colors from the schema are parseable
    let colors = [
        "#ffcc66",
        "#ffa659",
        "#1f2430",
        "rgba(36,41,54,0.88)",
        "#cccac2",
        "#707a8c",
        "#171B24",
        "rgba(255,204,102,0.12)",
        "rgba(115,208,255,0.75)",
        "rgba(115,208,255,0.9)",
        "rgba(255,213,128,0.9)",
        "rgba(255,166,89,0.9)",
        "rgba(186,230,126,0.9)",
        "rgba(223,191,255,0.9)",
        "#87d96c",
        "#ffa659",
        "#ff6666",
    ];
    for c in &colors {
        assert!(parse_color(c).is_ok(), "failed to parse default color: {c}");
    }
}
