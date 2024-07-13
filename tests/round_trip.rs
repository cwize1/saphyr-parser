use saphyr_parser::{writer::YamlWriter, Parser};

#[test]
fn test_round_trip_single_doc_string_value() {
    round_trip_test("a", "a", true, true);
}

#[test]
fn test_round_trip_multi_docs_string_values() {
    round_trip_test("---\na\n---\nb\n---\nc", "a\n---\nb\n---\nc", true, true);
}

#[test]
fn test_round_trip_json_list() {
    let expected = r"- 1
- 2
- 3";
    round_trip_test("[1,2,3]", expected, true, true);
}

#[test]
fn test_round_trip_yaml_list() {
    let input = r"- 1
- 2
- 3";
    round_trip_test(input, input, true, true);
}

#[test]
fn test_round_trip_json_object_nested() {
    let input = "{\"a\":\"b\",\"c\":{\"d\":\"e\"}}";
    let expected = r"a: b
c:
  d: e";
    round_trip_test(input, expected, true, true);
}

#[test]
fn test_round_trip_yaml_objects_and_lists() {
    let input = r"a:
  - b: c
    d:
      - e
      - f: g
        h:
          i: j
          k:
            - l
            - m: ~
              n: o";
    round_trip_test(input, input, true, true);
}

#[test]
fn test_round_trip_yaml_empty_objects_and_lists() {
    let input = r"a: []
b: {}";
    round_trip_test(input, input, true, true);
}

fn round_trip_test(input: &str, expected: &str, compact: bool, multiline: bool) {
    let mut parser = Parser::new(input.chars());

    let mut output = String::new();
    let mut writer = YamlWriter::new(&mut output);
    writer.compact(compact);
    writer.multiline_strings(multiline);

    while let Some(next) = parser.next_event() {
        let (event, _) = next.expect("no parse error");
        let write_event = event.into();
        writer.event(write_event).expect("no write error");
    }

    println!("expected:\n{expected}");
    println!("emitted:\n{output}");
    assert_eq!(expected, output);
}
