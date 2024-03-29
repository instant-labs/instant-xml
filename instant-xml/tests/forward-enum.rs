use similar_asserts::assert_eq;

use instant_xml::{from_str, to_string, FromXml, ToXml};

#[derive(Debug, Eq, FromXml, PartialEq, ToXml)]
#[xml(forward)]
enum Foo {
    Bar(Bar),
    Baz(Baz),
}

#[derive(Debug, Eq, FromXml, PartialEq, ToXml)]
struct Bar {
    bar: u8,
}

#[derive(Debug, Eq, FromXml, PartialEq, ToXml)]
struct Baz {
    baz: String,
}

#[test]
fn wrapped_enum() {
    let v = Foo::Bar(Bar { bar: 42 });
    let xml = r#"<Bar><bar>42</bar></Bar>"#;
    assert_eq!(xml, to_string(&v).unwrap());
    assert_eq!(v, from_str(xml).unwrap());
}
