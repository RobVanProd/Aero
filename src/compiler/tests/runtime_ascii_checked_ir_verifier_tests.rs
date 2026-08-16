use std::fs;
use std::path::PathBuf;

const PRODUCT_RELATIVE_PATH: &str =
    "../../examples/aero_frontend_v0/runtime_ascii_checked_ir_verifier.aero";
const M1B_RELATIVE_PATH: &str = "../../examples/aero_frontend_v0/runtime_ascii_checked_ir.aero";
const WORKFLOW_RELATIVE_PATH: &str = "../../.github/workflows/rust.yml";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-045 intentional product red: tracked runtime ASCII checked IR verifier is absent";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Diagnostic {
    status: i32,
    word_index: i32,
    record_id: i32,
    code: i32,
    expected: i32,
    actual: i32,
}

impl Diagnostic {
    const fn success() -> Self {
        Self {
            status: 0,
            word_index: -1,
            record_id: 0,
            code: 0,
            expected: 0,
            actual: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Seal {
    attempted: i32,
    diagnostic: Diagnostic,
    instruction_count: i32,
    result_count: i32,
    root_value: i32,
    verified_results: Vec<i32>,
    checksum: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Instruction {
    opcode: i32,
    left_kind: i32,
    left_payload: i32,
    right_kind: i32,
    right_payload: i32,
    origin: i32,
}

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn checksum_step(checksum: i32, word: i32) -> i32 {
    i32::try_from((i64::from(checksum) * 31 + i64::from(word)) % 1_000_003)
        .expect("bounded checksum")
}

fn signed_magnitude(value: i32) -> (i32, i32, i32) {
    if value >= 0 {
        return (0, value / 32_768, value % 32_768);
    }
    if value == i32::MIN {
        return (1, 65_536, 0);
    }
    let magnitude = -value;
    (1, magnitude / 32_768, magnitude % 32_768)
}

fn failure(
    status: i32,
    word_index: usize,
    record_id: i32,
    code: i32,
    expected: i32,
    actual: i32,
) -> Diagnostic {
    Diagnostic {
        status,
        word_index: i32::try_from(word_index).expect("bounded word index"),
        record_id,
        code,
        expected,
        actual,
    }
}

fn finish(
    view: &[i32],
    diagnostic: Diagnostic,
    instruction_count: i32,
    result_count: i32,
    root_value: i32,
    verified_results: Vec<i32>,
) -> Seal {
    let mut checksum = 29;
    for word in view {
        checksum = checksum_step(checksum, *word);
    }
    checksum = checksum_step(checksum, 995);
    let (root_sign, root_high, root_low) = if diagnostic.status == 0 {
        signed_magnitude(root_value)
    } else {
        (0, 0, 0)
    };
    for word in [
        diagnostic.status,
        diagnostic.word_index + 1,
        diagnostic.record_id,
        diagnostic.code,
        diagnostic.expected,
        diagnostic.actual,
        1,
        instruction_count,
        result_count,
        root_sign,
        root_high,
        root_low,
        i32::try_from(verified_results.len()).expect("bounded verifier results"),
    ] {
        checksum = checksum_step(checksum, word);
    }
    Seal {
        attempted: 1,
        diagnostic,
        instruction_count,
        result_count,
        root_value: if diagnostic.status == 0 {
            root_value
        } else {
            0
        },
        verified_results,
        checksum,
    }
}

fn resolve_operand(
    kind: i32,
    payload: i32,
    results: &[i32],
    instruction_id: i32,
    kind_word: usize,
    payload_word: usize,
) -> Result<i32, Diagnostic> {
    match kind {
        1 => Ok(payload),
        2 if payload > 0 && payload < instruction_id => {
            let index = usize::try_from(payload - 1).expect("positive result ID");
            results.get(index).copied().ok_or_else(|| {
                failure(
                    4,
                    payload_word,
                    instruction_id,
                    3,
                    i32::try_from(results.len()).expect("bounded result count"),
                    payload,
                )
            })
        }
        2 => Err(failure(
            4,
            payload_word,
            instruction_id,
            3,
            instruction_id - 1,
            payload,
        )),
        _ => Err(failure(4, kind_word, instruction_id, 1, 1, kind)),
    }
}

fn verify(words: &[i32], fault: Option<(usize, i32)>) -> Seal {
    let mut view = words.to_vec();
    if let Some((index, value)) = fault {
        if index >= view.len() {
            return finish(
                &view,
                failure(1, index, 0, 1, i32::try_from(view.len()).unwrap(), 0),
                0,
                0,
                0,
                Vec::new(),
            );
        }
        if value < 0 {
            return finish(&view, failure(1, index, 0, 1, 0, 0), 0, 0, 0, Vec::new());
        }
        view[index] = value;
    }

    if let Some((index, actual)) = view.iter().copied().enumerate().find(|(_, word)| *word < 0) {
        return finish(
            &view,
            failure(1, index, 0, 1, 0, actual.saturating_abs()),
            0,
            0,
            0,
            Vec::new(),
        );
    }
    if view.len() < 9 {
        return finish(
            &view,
            failure(1, view.len(), 0, 3, 9, i32::try_from(view.len()).unwrap()),
            0,
            0,
            0,
            Vec::new(),
        );
    }

    for (index, expected) in [(0, 1), (1, 1), (2, 1), (5, 1), (8, 1)] {
        if view[index] != expected {
            return finish(
                &view,
                failure(1, index, 0, 2, expected, view[index]),
                0,
                0,
                0,
                Vec::new(),
            );
        }
    }
    let instruction_count = view[3];
    let result_count = view[4];
    if !(1..=510).contains(&instruction_count) {
        return finish(
            &view,
            failure(1, 3, 0, 2, 510, instruction_count),
            0,
            0,
            0,
            Vec::new(),
        );
    }
    if !(0..=509).contains(&result_count) {
        return finish(
            &view,
            failure(1, 4, 0, 2, 509, result_count),
            instruction_count,
            0,
            0,
            Vec::new(),
        );
    }
    if instruction_count != result_count + 1 {
        return finish(
            &view,
            failure(1, 4, 0, 2, instruction_count - 1, result_count),
            instruction_count,
            result_count,
            0,
            Vec::new(),
        );
    }
    let expected_words = 25_i32
        .checked_add(
            instruction_count
                .checked_mul(11)
                .expect("bounded instructions"),
        )
        .and_then(|count| count.checked_add(result_count.checked_mul(6)?))
        .expect("bounded module length");
    if usize::try_from(expected_words).unwrap() != view.len() {
        return finish(
            &view,
            failure(
                1,
                view.len().min(usize::try_from(expected_words).unwrap()),
                0,
                3,
                expected_words,
                i32::try_from(view.len()).unwrap(),
            ),
            instruction_count,
            result_count,
            0,
            Vec::new(),
        );
    }

    let function_node = view[12];
    for (index, expected) in [
        (9, 1),
        (10, 1),
        (13, 0),
        (14, 1),
        (15, 1),
        (16, 1),
        (17, instruction_count),
    ] {
        if view[index] != expected {
            return finish(
                &view,
                failure(2, index, 0, 1, expected, view[index]),
                instruction_count,
                result_count,
                0,
                Vec::new(),
            );
        }
    }
    if view[11] <= 0 {
        return finish(
            &view,
            failure(2, 11, 0, 1, 1, view[11]),
            instruction_count,
            result_count,
            0,
            Vec::new(),
        );
    }
    if !(3..=512).contains(&function_node) {
        return finish(
            &view,
            failure(2, 12, 0, 1, 512, function_node),
            instruction_count,
            result_count,
            0,
            Vec::new(),
        );
    }
    for (index, expected) in [
        (18, 2),
        (19, 1),
        (20, 1),
        (21, 1),
        (22, 0),
        (23, 1),
        (24, instruction_count),
    ] {
        if view[index] != expected {
            return finish(
                &view,
                failure(2, index, 0, 2, expected, view[index]),
                instruction_count,
                result_count,
                0,
                Vec::new(),
            );
        }
    }

    let instruction_count_usize = usize::try_from(instruction_count).unwrap();
    let mut values = Vec::with_capacity(usize::try_from(result_count).unwrap());
    let mut origins = Vec::with_capacity(usize::try_from(result_count).unwrap());
    let mut previous_origin = 0;
    let mut return_operand = (0, 0);
    let mut return_value = 0;
    for index in 0..instruction_count_usize {
        let id = i32::try_from(index + 1).unwrap();
        let base = 25 + index * 11;
        let last = index + 1 == instruction_count_usize;
        for (field, expected) in [(0, 3), (1, id), (10, 1)] {
            if view[base + field] != expected {
                return finish(
                    &view,
                    failure(3, base + field, id, 1, expected, view[base + field]),
                    instruction_count,
                    result_count,
                    0,
                    values,
                );
            }
        }
        let opcode = view[base + 2];
        let expected_opcode = if last { 6 } else { opcode };
        if (last && opcode != 6) || (!last && !(1..=5).contains(&opcode)) {
            return finish(
                &view,
                failure(3, base + 2, id, 2, expected_opcode, opcode),
                instruction_count,
                result_count,
                0,
                values,
            );
        }
        let expected_result = if last { 0 } else { id };
        let expected_type = if last { 0 } else { 1 };
        for (field, expected) in [(3, expected_result), (4, expected_type)] {
            if view[base + field] != expected {
                return finish(
                    &view,
                    failure(3, base + field, id, 2, expected, view[base + field]),
                    instruction_count,
                    result_count,
                    0,
                    values,
                );
            }
        }
        let origin = view[base + 9];
        let valid_origin = if last {
            origin == function_node - 1 && origin > previous_origin
        } else {
            origin > previous_origin && origin < function_node - 1
        };
        if !valid_origin {
            return finish(
                &view,
                failure(3, base + 9, id, 3, previous_origin + 1, origin),
                instruction_count,
                result_count,
                0,
                values,
            );
        }

        let left = match resolve_operand(
            view[base + 5],
            view[base + 6],
            &values,
            id,
            base + 5,
            base + 6,
        ) {
            Ok(value) => value,
            Err(diagnostic) => {
                return finish(
                    &view,
                    diagnostic,
                    instruction_count,
                    result_count,
                    0,
                    values,
                );
            }
        };
        let unary = opcode == 5 || opcode == 6;
        let right = if unary {
            if view[base + 7] != 0 || view[base + 8] != 0 {
                return finish(
                    &view,
                    failure(
                        4,
                        if view[base + 7] != 0 {
                            base + 7
                        } else {
                            base + 8
                        },
                        id,
                        2,
                        0,
                        if view[base + 7] != 0 {
                            view[base + 7]
                        } else {
                            view[base + 8]
                        },
                    ),
                    instruction_count,
                    result_count,
                    0,
                    values,
                );
            }
            0
        } else {
            match resolve_operand(
                view[base + 7],
                view[base + 8],
                &values,
                id,
                base + 7,
                base + 8,
            ) {
                Ok(value) => value,
                Err(diagnostic) => {
                    return finish(
                        &view,
                        diagnostic,
                        instruction_count,
                        result_count,
                        0,
                        values,
                    );
                }
            }
        };

        if last {
            return_operand = (view[base + 5], view[base + 6]);
            return_value = left;
        } else {
            let evaluated = match opcode {
                1 => left.checked_add(right),
                2 => left.checked_sub(right),
                3 => left.checked_mul(right),
                4 if right == 0 => None,
                4 => left.checked_div(right),
                5 => left.checked_neg(),
                _ => unreachable!("opcode checked above"),
            };
            let Some(evaluated) = evaluated else {
                return finish(
                    &view,
                    failure(5, base + 2, id, opcode, 0, 0),
                    instruction_count,
                    result_count,
                    0,
                    values,
                );
            };
            values.push(evaluated);
            origins.push(origin);
        }
        previous_origin = origin;
    }

    let results_base = 25 + instruction_count_usize * 11;
    for index in 0..usize::try_from(result_count).unwrap() {
        let id = i32::try_from(index + 1).unwrap();
        let base = results_base + index * 6;
        for (field, expected) in [(0, 4), (1, 1), (2, id), (3, 1)] {
            if view[base + field] != expected {
                return finish(
                    &view,
                    failure(6, base + field, id, 1, expected, view[base + field]),
                    instruction_count,
                    result_count,
                    0,
                    values,
                );
            }
        }
        if view[base + 4] != id {
            return finish(
                &view,
                failure(6, base + 4, id, 2, id, view[base + 4]),
                instruction_count,
                result_count,
                0,
                values,
            );
        }
        let expected_origin = origins[index];
        if view[base + 5] != expected_origin {
            return finish(
                &view,
                failure(6, base + 5, id, 3, expected_origin, view[base + 5]),
                instruction_count,
                result_count,
                0,
                values,
            );
        }
    }

    let root_operand = (view[6], view[7]);
    if root_operand != return_operand {
        let index = if root_operand.0 != return_operand.0 {
            6
        } else {
            7
        };
        return finish(
            &view,
            failure(
                7,
                index,
                instruction_count,
                2,
                if index == 6 {
                    return_operand.0
                } else {
                    return_operand.1
                },
                view[index],
            ),
            instruction_count,
            result_count,
            0,
            values,
        );
    }
    let root_value = match resolve_operand(
        root_operand.0,
        root_operand.1,
        &values,
        instruction_count + 1,
        6,
        7,
    ) {
        Ok(value) => value,
        Err(_) => {
            return finish(
                &view,
                failure(7, 6, instruction_count, 2, return_operand.0, root_operand.0),
                instruction_count,
                result_count,
                0,
                values,
            );
        }
    };
    if root_value != return_value {
        return finish(
            &view,
            failure(7, 7, instruction_count, 3, return_value, root_value),
            instruction_count,
            result_count,
            0,
            values,
        );
    }
    finish(
        &view,
        Diagnostic::success(),
        instruction_count,
        result_count,
        root_value,
        values,
    )
}

fn module(function_node: i32, instructions: &[Instruction], root: (i32, i32)) -> Vec<i32> {
    assert!(!instructions.is_empty());
    assert_eq!(instructions.last().unwrap().opcode, 6);
    let instruction_count = i32::try_from(instructions.len()).unwrap();
    let result_count = instruction_count - 1;
    let mut words = vec![
        1,
        1,
        1,
        instruction_count,
        result_count,
        1,
        root.0,
        root.1,
        1,
        1,
        1,
        1,
        function_node,
        0,
        1,
        1,
        1,
        instruction_count,
        2,
        1,
        1,
        1,
        0,
        1,
        instruction_count,
    ];
    for (index, instruction) in instructions.iter().enumerate() {
        let id = i32::try_from(index + 1).unwrap();
        let is_return = instruction.opcode == 6;
        words.extend([
            3,
            id,
            instruction.opcode,
            if is_return { 0 } else { id },
            if is_return { 0 } else { 1 },
            instruction.left_kind,
            instruction.left_payload,
            instruction.right_kind,
            instruction.right_payload,
            instruction.origin,
            1,
        ]);
    }
    for (index, instruction) in instructions[..instructions.len() - 1].iter().enumerate() {
        let id = i32::try_from(index + 1).unwrap();
        words.extend([4, 1, id, 1, id, instruction.origin]);
    }
    words
}

fn canonical_words() -> Vec<i32> {
    module(
        11,
        &[
            Instruction {
                opcode: 3,
                left_kind: 1,
                left_payload: 2,
                right_kind: 1,
                right_payload: 3,
                origin: 4,
            },
            Instruction {
                opcode: 1,
                left_kind: 1,
                left_payload: 1,
                right_kind: 2,
                right_payload: 1,
                origin: 5,
            },
            Instruction {
                opcode: 4,
                left_kind: 1,
                left_payload: 4,
                right_kind: 1,
                right_payload: 2,
                origin: 8,
            },
            Instruction {
                opcode: 2,
                left_kind: 2,
                left_payload: 2,
                right_kind: 2,
                right_payload: 3,
                origin: 9,
            },
            Instruction {
                opcode: 6,
                left_kind: 2,
                left_payload: 4,
                right_kind: 0,
                right_payload: 0,
                origin: 10,
            },
        ],
        (2, 4),
    )
}

#[test]
fn independent_b1a_oracle_freezes_the_canonical_seal() {
    let words = canonical_words();
    assert_eq!(words.len(), 104);
    let seal = verify(&words, None);
    assert_eq!(seal.attempted, 1);
    assert_eq!(seal.diagnostic, Diagnostic::success());
    assert_eq!((seal.instruction_count, seal.result_count), (5, 4));
    assert_eq!(seal.verified_results, vec![6, 7, 2, 5]);
    assert_eq!(seal.root_value, 5);
    assert_eq!(seal.checksum, 592_819);
    assert_eq!(
        seal,
        verify(&words, None),
        "verification must be deterministic"
    );
}

#[test]
fn independent_b1a_oracle_accepts_every_opcode_boundaries_and_maximum_counts() {
    let literal = module(
        3,
        &[Instruction {
            opcode: 6,
            left_kind: 1,
            left_payload: i32::MAX,
            right_kind: 0,
            right_payload: 0,
            origin: 2,
        }],
        (1, i32::MAX),
    );
    let literal_seal = verify(&literal, None);
    assert_eq!(literal_seal.diagnostic, Diagnostic::success());
    assert_eq!(literal_seal.root_value, i32::MAX);
    assert!(literal_seal.verified_results.is_empty());

    let all_opcodes = module(
        8,
        &[
            Instruction {
                opcode: 1,
                left_kind: 1,
                left_payload: 7,
                right_kind: 1,
                right_payload: 5,
                origin: 2,
            },
            Instruction {
                opcode: 2,
                left_kind: 2,
                left_payload: 1,
                right_kind: 1,
                right_payload: 2,
                origin: 3,
            },
            Instruction {
                opcode: 3,
                left_kind: 2,
                left_payload: 2,
                right_kind: 1,
                right_payload: 3,
                origin: 4,
            },
            Instruction {
                opcode: 4,
                left_kind: 2,
                left_payload: 3,
                right_kind: 1,
                right_payload: 5,
                origin: 5,
            },
            Instruction {
                opcode: 5,
                left_kind: 2,
                left_payload: 4,
                right_kind: 0,
                right_payload: 0,
                origin: 6,
            },
            Instruction {
                opcode: 6,
                left_kind: 2,
                left_payload: 5,
                right_kind: 0,
                right_payload: 0,
                origin: 7,
            },
        ],
        (2, 5),
    );
    let all_opcode_seal = verify(&all_opcodes, None);
    assert_eq!(all_opcode_seal.diagnostic, Diagnostic::success());
    assert_eq!(all_opcode_seal.verified_results, vec![12, 10, 30, 6, -6]);
    assert_eq!(all_opcode_seal.root_value, -6);

    let minimum = module(
        6,
        &[
            Instruction {
                opcode: 2,
                left_kind: 1,
                left_payload: 0,
                right_kind: 1,
                right_payload: i32::MAX,
                origin: 2,
            },
            Instruction {
                opcode: 2,
                left_kind: 2,
                left_payload: 1,
                right_kind: 1,
                right_payload: 1,
                origin: 3,
            },
            Instruction {
                opcode: 6,
                left_kind: 2,
                left_payload: 2,
                right_kind: 0,
                right_payload: 0,
                origin: 5,
            },
        ],
        (2, 2),
    );
    let minimum_seal = verify(&minimum, None);
    assert_eq!(minimum_seal.diagnostic, Diagnostic::success());
    assert_eq!(minimum_seal.root_value, i32::MIN);

    let mut maximum_instructions = Vec::with_capacity(510);
    for index in 0..509 {
        maximum_instructions.push(Instruction {
            opcode: 5,
            left_kind: if index == 0 { 1 } else { 2 },
            left_payload: if index == 0 {
                1
            } else {
                i32::try_from(index).unwrap()
            },
            right_kind: 0,
            right_payload: 0,
            origin: i32::try_from(index + 2).unwrap(),
        });
    }
    maximum_instructions.push(Instruction {
        opcode: 6,
        left_kind: 2,
        left_payload: 509,
        right_kind: 0,
        right_payload: 0,
        origin: 511,
    });
    let maximum = module(512, &maximum_instructions, (2, 509));
    let maximum_seal = verify(&maximum, None);
    assert_eq!(maximum_seal.diagnostic, Diagnostic::success());
    assert_eq!(maximum_seal.instruction_count, 510);
    assert_eq!(maximum_seal.result_count, 509);
    assert_eq!(maximum_seal.verified_results.len(), 509);
    assert_eq!(maximum_seal.root_value, -1);
}

#[test]
fn independent_b1a_oracle_rejects_each_corruption_family_deterministically() {
    let canonical = canonical_words();
    let mut mutations = Vec::new();
    let mut push = |label: &'static str, index: usize, value: i32, status: i32| {
        let mut words = canonical.clone();
        words[index] = value;
        mutations.push((label, words, status));
    };
    push("format", 0, 2, 1);
    push("function count", 1, 2, 1);
    push("block count", 2, 2, 1);
    push("instruction count", 3, 4, 1);
    push("result count", 4, 3, 1);
    push("entry function", 5, 2, 1);
    push("root type", 8, 0, 1);
    push("function tag", 9, 2, 2);
    push("function name", 11, 0, 2);
    push("function node", 12, 2, 2);
    push("function span", 17, 4, 2);
    push("block tag", 18, 1, 2);
    push("block reachability", 21, 0, 2);
    push("block successors", 22, 1, 2);
    push("instruction tag", 25, 4, 3);
    push("instruction id", 26, 2, 3);
    push("opcode", 27, 99, 3);
    push("instruction result", 28, 0, 3);
    push("instruction type", 29, 0, 3);
    push("instruction origin", 34, 0, 3);
    push("function identity", 35, 2, 3);
    push("operand kind", 30, 9, 4);
    push("forward result use", 44, 4, 4);
    push("unused unary lane", 76, 1, 4);
    push("divide by zero", 55, 0, 5);
    push("result tag", 80, 3, 6);
    push("result function", 81, 2, 6);
    push("result id", 82, 2, 6);
    push("result type", 83, 0, 6);
    push("definition id", 84, 2, 6);
    push("result origin", 85, 5, 6);
    push("root kind", 6, 1, 7);
    push("root payload", 7, 3, 7);
    push("return root", 75, 3, 7);

    let mut truncated = canonical.clone();
    truncated.pop();
    mutations.push(("truncated", truncated, 1));
    let mut trailing = canonical.clone();
    trailing.push(0);
    mutations.push(("trailing", trailing, 1));

    let overflow = module(
        5,
        &[
            Instruction {
                opcode: 1,
                left_kind: 1,
                left_payload: i32::MAX,
                right_kind: 1,
                right_payload: 1,
                origin: 2,
            },
            Instruction {
                opcode: 6,
                left_kind: 2,
                left_payload: 1,
                right_kind: 0,
                right_payload: 0,
                origin: 4,
            },
        ],
        (2, 1),
    );
    mutations.push(("numeric overflow", overflow, 5));

    for (label, words, expected_status) in mutations {
        let first = verify(&words, None);
        let second = verify(&words, None);
        assert_eq!(
            first, second,
            "{label} diagnostic/seal was nondeterministic"
        );
        assert_eq!(
            first.diagnostic.status, expected_status,
            "{label} reached the wrong verifier boundary: {first:?}"
        );
        assert_ne!(first.diagnostic, Diagnostic::success(), "{label}");
    }

    let faulted = verify(&canonical, Some((0, 2)));
    assert_eq!(faulted.diagnostic.status, 1);
    assert_ne!(faulted.checksum, verify(&canonical, None).checksum);
    let outside = verify(&canonical, Some((canonical.len(), 0)));
    assert_eq!(outside.diagnostic.status, 1);
}

#[test]
fn tracked_runtime_ascii_checked_ir_verifier_is_structurally_complete() {
    let product_path = repository_path(PRODUCT_RELATIVE_PATH);
    assert!(product_path.is_file(), "{INTENTIONAL_PRODUCT_RED}");

    let product = fs::read_to_string(&product_path).expect("read tracked CAP-045 product");
    let m1b = fs::read_to_string(repository_path(M1B_RELATIVE_PATH))
        .expect("read accepted CAP-044 product");
    assert!(m1b.contains("// CAP-044 TRACKED SELF-TEST"));
    assert!(m1b.contains("9, 5, 4, 104, 2, 4, 1, 355067"));

    assert!(product.is_ascii(), "CAP-045 product must remain raw ASCII");
    assert!(product.contains("// CAP-045 B1A VERIFIER BEGIN"));
    assert!(product.contains("// CAP-045 B1A VERIFIER END"));
    assert!(product.contains("// CAP-045 TRACKED SELF-TEST"));
    assert!(product.contains("let mut verified_results: ByteBuffer = bytes_new();"));
    assert_eq!(
        product.matches(": ByteBuffer = bytes_new();").count(),
        13,
        "CAP-045 must own exactly the accepted twelve owners plus verified_results"
    );
    for anchor in [
        "verification_fault_word: int",
        "verification_fault_value: int",
        "verified_attempted",
        "verified_status",
        "verified_word_index",
        "verified_record_id",
        "verified_code",
        "verified_expected",
        "verified_actual",
        "verified_instruction_count",
        "verified_result_count",
        "verified_root_value",
        "verified_checksum",
        "verified_checksum = 29",
        "checksum_step(verified_checksum, 995)",
        "5, 4, 5, 4, 592819",
        "-1, 0",
    ] {
        assert!(
            product.contains(anchor),
            "CAP-045 product omitted `{anchor}`"
        );
    }

    let verifier = product
        .split_once("// CAP-045 B1A VERIFIER BEGIN")
        .and_then(|(_, suffix)| suffix.split_once("// CAP-045 B1A VERIFIER END"))
        .map(|(section, _)| section)
        .expect("isolate CAP-045 verifier section");
    for forbidden in [
        "bytes_get(&source",
        "bytes_get(&names",
        "bytes_get(&tokens",
        "bytes_get(&nodes",
        "bytes_get(&values",
        "bytes_get(&operators",
        "bytes_get(&origins",
        "bytes_get(&symbols",
        "bytes_get(&facts",
        "bytes_get(&checked_values",
        "bytes_get(&checked_instructions",
        "semantic_checksum",
        "checked_checksum",
        "checked_root_",
        "checked_candidate_root_",
        "bytes_push(&mut checked_ir",
        "println!",
    ] {
        assert!(
            !verifier.contains(forbidden),
            "CAP-045 verifier crossed its independence boundary via `{forbidden}`"
        );
    }
    assert!(verifier.contains("bytes_get(&checked_ir"));
    assert!(verifier.contains("bytes_push(&mut verified_results"));

    let workflow = fs::read_to_string(repository_path(WORKFLOW_RELATIVE_PATH))
        .expect("read protected workflow");
    for step in [
        "Test runtime ASCII checked IR verifier at O0 and O2",
        "Test runtime ASCII checked IR verifier on Windows at O0 and O2",
    ] {
        assert!(
            workflow.contains(step),
            "protected workflow omitted `{step}`"
        );
    }
}
