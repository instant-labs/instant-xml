use similar_asserts::assert_eq;

use instant_xml::{to_string, ToXml};

#[derive(Debug, Eq, PartialEq, ToXml)]
#[xml(ns(bar = "BAZ", foo = "BAR"))]
struct StructWithNamedFields {
    flag: bool,
    #[xml(ns("BAZ"))]
    string: String,
    #[xml(ns("typo"))]
    number: i32,
}

// Tests:
// - Empty default namespace
// - Prefix namespace
// - Direct namespace

#[test]
fn struct_with_named_fields() {
    assert_eq!(
        to_string(&StructWithNamedFields {
            flag: true,
            string: "test".to_string(),
            number: 1,
        })
        .unwrap(),
        "<StructWithNamedFields xmlns:bar=\"BAZ\" xmlns:foo=\"BAR\"><flag>true</flag><bar:string>test</bar:string><number xmlns=\"typo\">1</number></StructWithNamedFields>"
    );
}
