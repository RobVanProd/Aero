//! H1-AST-001 observes actual native arenas without changing the product ABI.
//! Exit 80 is deliberately parser-only: it claims no semantic authentication.

use super::*;
use std::io::Read;

const COMPACT: &[u8] = b"fn f()->int{return 1;}";

/// Additive, frozen source transform over the complete reproducible predecessor.
/// This is source identity evidence, not the structural/provenance oracle above.
pub(super) fn expected_successor_source(predecessor: &str) -> String {
    assert_eq!(predecessor.len(), 334106);
    assert_eq!(
        format!("{:x}", md5::compute(predecessor.as_bytes())),
        "13c3d98476f74fdedb211d6765f1485d",
        "historical source is unchanged"
    );
    let mut lines: Vec<&str> = predecessor.split_inclusive('\n').collect();
    for &(start, count, old, replacement) in CONNECTED_SOURCE_EDITS.iter().rev() {
        assert_eq!(
            lines[start..start + count].concat(),
            old,
            "frozen source transform old lines at {start}"
        );
        lines.splice(start..start + count, replacement.split_inclusive('\n'));
    }
    lines.concat()
}

const CONNECTED_SOURCE_EDITS: &[(usize, usize, &str, &str)] = &[
    (
        1309,
        0,
        r####""####,
        r####"    let mut parser_record_5: int = 0;
    let mut parser_record_6: int = 0;
    let mut parser_record_7: int = 0;
    let mut parser_record_8: int = 0;
    let mut parser_record_9: int = 0;
    let mut parser_record_10: int = 0;
    let mut parser_record_11: int = 0;
    let mut parser_record_12: int = 0;
    let mut parser_record_13: int = 0;
    let mut parser_record_14: int = 0;
    let mut parser_record_15: int = 0;
    let mut parser_record_16: int = 0;
    let mut parser_record_17: int = 0;
    let mut parser_record_18: int = 0;
    let mut parser_record_19: int = 0;
"####,
    ),
    (
        1321,
        0,
        r####""####,
        r####"    let mut parser_append_5: int = 0;
    let mut parser_append_6: int = 0;
    let mut parser_append_7: int = 0;
    let mut parser_append_8: int = 0;
    let mut parser_append_9: int = 0;
    let mut parser_append_10: int = 0;
    let mut parser_append_11: int = 0;
    let mut parser_append_12: int = 0;
    let mut parser_append_13: int = 0;
    let mut parser_append_14: int = 0;
    let mut parser_append_15: int = 0;
    let mut parser_append_16: int = 0;
    let mut parser_append_17: int = 0;
    let mut parser_append_18: int = 0;
    let mut parser_append_19: int = 0;
"####,
    ),
    (
        1332,
        0,
        r####""####,
        r####"    // H1-AST-001 retains connected occurrence structure without adding meaning.
    let mut ast_after: int = 0;
    let mut ast_actual: int = 0;
    let mut ast_resume: int = 0;
    let mut ast_statement_after: int = 0;
    let mut ast_statement_node: int = 0;
    let mut ast_block_node: int = 0;
    let mut ast_function_body: int = 0;
    let mut ast_function_parameters: int = 0;
    let mut ast_parameter_annotation: int = 0;
    let mut ast_parameter_start: int = 0;
    let mut ast_parameter_line: int = 0;
    let mut ast_parameter_column: int = 0;
    let mut ast_parameter_type_start: int = 0;
    let mut ast_parameter_type_line: int = 0;
    let mut ast_parameter_type_column: int = 0;
    let mut ast_statement_start: int = 0;
    let mut ast_statement_line: int = 0;
    let mut ast_statement_column: int = 0;
    let mut ast_statement_name: int = 0;
    let mut ast_statement_mutable: int = 0;
    let mut ast_statement_type: int = 0;
    let mut ast_statement_annotation: int = 0;
    let mut ast_statement_type_start: int = 0;
    let mut ast_statement_type_line: int = 0;
    let mut ast_statement_type_column: int = 0;
    let mut ast_block_statements: int = 0;
    let mut ast_block_first: int = 0;
    let mut ast_block_cells: int = 0;
    let mut ast_block_start: int = 0;
    let mut ast_block_line: int = 0;
    let mut ast_block_column: int = 0;
    let mut ast_close_after: int = 0;
    let mut ast_control_resume: int = 0;
    let mut ast_control_kind: int = 0;
    let mut ast_control_phase: int = 0;
    let mut ast_control_previous: int = 0;
    let mut ast_control_condition: int = 0;
    let mut ast_control_then: int = 0;
    let mut ast_control_start: int = 0;
    let mut ast_control_line: int = 0;
    let mut ast_control_column: int = 0;
    let mut ast_control_node: int = 0;
    let mut ast_control_else: int = 0;
    let mut ast_control_depth_push: int = 0;
    let mut ast_control_after: int = 0;
    let mut ast_control_lookahead: int = 0;
    let mut ast_match_start: int = 0;
    let mut ast_match_line: int = 0;
    let mut ast_match_column: int = 0;
    let mut ast_match_subject: int = 0;
    let mut ast_match_constructor: int = 0;
    let mut ast_match_constructor_start: int = 0;
    let mut ast_match_constructor_line: int = 0;
    let mut ast_match_constructor_column: int = 0;
    let mut ast_match_binder: int = 0;
    let mut ast_match_first: int = 0;
    let mut ast_match_second: int = 0;
"####,
    ),
    (
        1558,
        0,
        r####""####,
        r####"                    ast_parameter_start = current_start;
                    ast_parameter_line = current_line;
                    ast_parameter_column = current_column;
"####,
    ),
    (
        1580,
        0,
        r####""####,
        r####"                    if param_cycle_mode == 2 {
                        ast_parameter_type_start = current_start;
                        ast_parameter_type_line = current_line;
                        ast_parameter_type_column = current_column;
                    }
"####,
    ),
    (
        1707,
        0,
        r####""####,
        r####"                ast_function_parameters = 0;
                ast_block_statements = 0;
                ast_block_first = 0;
                ast_block_cells = 0;
                ast_control_resume = 0;
"####,
    ),
    (
        1715,
        0,
        r####""####,
        r####"            if parser_running == 1 && skeleton_step == 6 {
                ast_block_start = current_start;
                ast_block_line = current_line;
                ast_block_column = current_column;
            }
"####,
    ),
    (
        1721,
        0,
        r####""####,
        r####"                if param_store == 1 {
                    pending_node_kind = 24;
                    pending_node_payload = param_type_code;
                    pending_node_left = 0;
                    pending_node_right = 0;
                    pending_node_offset = ast_parameter_type_start;
                    pending_node_line = ast_parameter_type_line;
                    pending_node_column = ast_parameter_type_column;
                    ast_actual = 1;
                    ast_after = 62;
                    parser_state = 60;
                }
"####,
    ),
    (
        1811,
        3,
        r####"                        parser_record_target = 3;
                        parser_record_width = 3;
                        parser_record_index = block_top;
"####,
        r####"                        parse_index = parse_index + 1;
                        ast_statement_after = 67;
                        parser_state = 67;
                        if block_state == 2 {
                            ast_statement_start = return_start;
                            ast_statement_line = return_line;
                            ast_statement_column = return_column;
                            pending_node_kind = 18;
                            pending_node_payload = 0;
                            pending_node_left = body_root;
                            pending_node_right = 0;
                            pending_node_offset = return_start;
                            pending_node_line = return_line;
                            pending_node_column = return_column;
                            ast_actual = 6;
                            ast_after = 66;
                            parser_state = 60;
                        }
                    }
                }
                if parser_running == 1 && statement_mode > 0 {
                    if ast_block_statements == 1 && ast_block_cells == 0 {
                        ast_resume = 45;
                        parser_record_target = 5;
                        parser_record_width = 5;
                        parser_record_index = ast_block_first;
"####,
    ),
    (
        1817,
        6,
        r####"                        parser_record_0 = 0;
                        parser_record_1 = 0;
                        parser_record_2 = 0;
                        parser_record_3 = 0;
                        parser_record_4 = 0;
                        parser_record_after = 54;
"####,
        r####"                        parser_record_after = 64;
"####,
    ),
    (
        1824,
        14,
        r####"                    }
                }
                if parser_running == 1 && statement_mode > 0 {
                    block_state = 1;
                    parse_index = parse_index + 1;
                    value_top = 0;
                    value_depth = 0;
                    expecting_operand = 1;
                    parser_state = 46;
                    if statement_mode == 3 {
                        parser_state = 40;
                    }
                    if statement_mode >= 4 {
                        parser_state = 3;
"####,
        r####"                    } else {
                        ast_statement_start = current_start;
                        ast_statement_line = current_line;
                        ast_statement_column = current_column;
                        ast_statement_name = current_name_id;
                        ast_statement_mutable = 0;
                        ast_statement_type = 0;
                        ast_statement_annotation = 0;
                        ast_control_resume = 0;
                        block_state = 1;
                        parse_index = parse_index + 1;
                        value_top = 0;
                        value_depth = 0;
                        expecting_operand = 1;
                        parser_state = 46;
                        if statement_mode == 3 {
                            parser_state = 40;
                        }
                        if statement_mode >= 4 {
                            parser_state = 3;
                        }
"####,
    ),
    (
        1899,
        0,
        r####""####,
        r####"                if stmt_cycle_step == 3 {
                    ast_statement_type_start = current_start;
                    ast_statement_type_line = current_line;
                    ast_statement_type_column = current_column;
                }
"####,
    ),
    (
        1953,
        0,
        r####""####,
        r####"                if stmt_cycle_step == 0 && current_kind == 5 {
                    ast_statement_mutable = 1;
                }
                if (stmt_cycle_step == 0 && current_kind == 1)
                    || stmt_cycle_step == 1 {
                    ast_statement_name = current_name_id;
                }
"####,
    ),
    (
        1968,
        0,
        r####""####,
        r####"                ast_statement_type = 0;
                if stmt_cycle_step == 3 && stmt_is_int == 1 {
                    ast_statement_type = 1;
                }
                if stmt_cycle_step == 3 && stmt_is_buffer == 1 {
                    ast_statement_type = 3;
                }
                if stmt_cycle_step == 9 {
                    ast_statement_type = 2;
                }
                if ast_statement_type > 0 {
                    pending_node_kind = 24;
                    pending_node_payload = ast_statement_type;
                    pending_node_left = 0;
                    pending_node_right = 0;
                    pending_node_offset = ast_statement_type_start;
                    pending_node_line = ast_statement_type_line;
                    pending_node_column = ast_statement_type_column;
                    ast_actual = 1;
                    ast_after = 74;
                    parser_state = 60;
                }
"####,
    ),
    (
        1971,
        4,
        r####"        // ';' terminates the statement rather than closing the body. It is the
        // one rule the ordinary return expression and the CAP-051 match
        // construct both return to, so their two entry points into the closing
        // sequence collapse into this one.
"####,
        r####"        // Completing a statement retains its declaration/target and expression.
        // A Return stays pending until this block's closing brace validates.
"####,
    ),
    (
        1996,
        4,
        r####"                // 'body_root' stays one register and the last write wins. Every
                // block's 'return' is its last statement and every function
                // body ends in one, so the last 'return' completed in token
                // order within a function is always the function body's own.
"####,
        r####""####,
    ),
    (
        2003,
        12,
        r####"                }
                block_kind_push = 0;
                if statement_mode == 4 {
                    block_kind_push = 1;
                }
                if statement_mode == 5 {
                    block_kind_push = 2;
                }
                statement_mode = 0;
                parser_state = 44;
                if block_kind_push > 0 {
                    parser_state = 52;
"####,
        r####"                    statement_mode = 0;
                    parser_state = 44;
                } else {
                    if statement_mode == 1 || statement_mode == 2 {
                        pending_node_kind = 28;
                        pending_node_payload = ast_statement_name;
                        pending_node_left = expression_root;
                        pending_node_right = 0;
                        ast_actual = 1;
                        if statement_mode == 1 {
                            pending_node_kind = 26 + ast_statement_mutable;
                            pending_node_left = ast_statement_annotation;
                            pending_node_right = expression_root;
                            ast_actual = 4;
                        }
                        pending_node_offset = ast_statement_start;
                        pending_node_line = ast_statement_line;
                        pending_node_column = ast_statement_column;
                        ast_statement_after = 44;
                        ast_after = 66;
                        parser_state = 60;
                    } else {
                        ast_control_kind = 1;
                        if statement_mode == 5 {
                            ast_control_kind = 2;
                        }
                        ast_control_phase = 1;
                        ast_control_previous = block_top;
                        ast_control_condition = expression_root;
                        ast_control_then = 0;
                        ast_control_start = ast_statement_start;
                        ast_control_line = ast_statement_line;
                        ast_control_column = ast_statement_column;
                        parser_append_0 = ast_control_kind;
                        parser_append_1 = ast_control_previous;
                        parser_append_2 = ast_control_resume;
                        parser_append_3 = ast_control_phase;
                        parser_append_4 = block_state;
                        parser_append_5 = ast_block_statements;
                        parser_append_6 = ast_block_first;
                        parser_append_7 = ast_block_cells;
                        parser_append_8 = ast_block_start;
                        parser_append_9 = ast_block_line;
                        parser_append_10 = ast_block_column;
                        parser_append_11 = body_root;
                        parser_append_12 = return_start;
                        parser_append_13 = return_line;
                        parser_append_14 = return_column;
                        parser_append_15 = ast_control_condition;
                        parser_append_16 = ast_control_then;
                        parser_append_17 = ast_control_start;
                        parser_append_18 = ast_control_line;
                        parser_append_19 = ast_control_column;
                        ast_control_depth_push = 1;
                        ast_control_after = 76;
                        parser_state = 52;
                    }
"####,
    ),
    (
        2019,
        2,
        r####"        // An 'else' continues the chain with another 'if', or opens the final
        // 'else' body. Nothing else is admissible after it.
"####,
        r####"        // Immutable twenty-word control continuations preserve the complete
        // parent block, pending Return, condition, branch and source anchors.
"####,
    ),
    (
        2022,
        6,
        r####"            if current_kind < 0 || current_start < 0 || current_length < 0
                || current_line <= 0 || current_column <= 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && current_kind != 12 && current_kind != 7 {
"####,
        r####"            if current_kind != 12 && current_kind != 7 {
"####,
    ),
    (
        2038,
        1,
        r####"                block_state = 1;
"####,
        r####"                ast_control_phase = 3;
                ast_control_after = 76;
"####,
    ),
    (
        2040,
        10,
        r####"                    statement_mode = 4;
                    value_top = 0;
                    value_depth = 0;
                    expecting_operand = 1;
                    parser_state = 3;
                } else {
                    statement_mode = 0;
                    block_kind_push = 3;
                    parser_state = 52;
                }
"####,
        r####"                    ast_control_phase = 4;
                    ast_control_after = 77;
                }
                parser_append_0 = ast_control_kind;
                parser_append_1 = ast_control_previous;
                parser_append_2 = ast_control_resume;
                parser_append_3 = ast_control_phase;
                parser_append_4 = block_state;
                parser_append_5 = ast_block_statements;
                parser_append_6 = ast_block_first;
                parser_append_7 = ast_block_cells;
                parser_append_8 = ast_block_start;
                parser_append_9 = ast_block_line;
                parser_append_10 = ast_block_column;
                parser_append_11 = body_root;
                parser_append_12 = return_start;
                parser_append_13 = return_line;
                parser_append_14 = return_column;
                parser_append_15 = ast_control_condition;
                parser_append_16 = ast_control_then;
                parser_append_17 = ast_control_start;
                parser_append_18 = ast_control_line;
                parser_append_19 = ast_control_column;
                ast_control_depth_push = 0;
                parser_state = 52;
"####,
    ),
    (
        2052,
        8,
        r####"
        // Push one block record: the block's kind - 1 an 'if' body, 2 a 'while'
        // body, 3 an 'else' body - the enclosing block's statement state, and
        // the link to the enclosing record. Only a nested block pushes, so a
        // 'block_top' of zero is the function body and its closing rule is
        // unchanged. The store is a fourth monotonic parse-group counter and
        // carries the same bound and the same exhaustion diagnostic as the
        // value and operator stores.
"####,
        r####""####,
    ),
    (
        2072,
        6,
        r####"                parser_append_width = 3;
                parser_append_0 = block_kind_push;
                parser_append_1 = block_state;
                parser_append_2 = block_top;
                parser_append_3 = 0;
                parser_append_4 = 0;
"####,
        r####"                parser_append_width = 20;
"####,
    ),
    (
        2090,
        1,
        r####"            block_depth = block_depth + 1;
"####,
        r####"            block_depth = block_depth + ast_control_depth_push;
            parser_state = ast_control_after;
        }
        if parser_cycle_state == 76 {
            ast_block_start = current_start;
            ast_block_line = current_line;
            ast_block_column = current_column;
"####,
    ),
    (
        2092,
        0,
        r####""####,
        r####"            ast_block_statements = 0;
            ast_block_first = 0;
            ast_block_cells = 0;
            body_root = 0;
            return_start = 0;
            return_line = 0;
            return_column = 0;
            statement_mode = 0;
            match_active = 0;
            ast_control_lookahead = 0;
"####,
    ),
    (
        2094,
        14,
        r####"
        // The block record is decoded; pop it, restore the enclosing block's
        // statement state, and admit 'else' only after an 'if' body.
        if parser_cycle_state == 54 {
            block_kind_popped = parser_record_0;
            block_state_saved = parser_record_1;
            block_previous = parser_record_2;
            if block_kind_popped <= 0 || block_kind_popped > 3
                || block_state_saved < 0 || block_state_saved > 2 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && (block_previous < 0 || block_depth <= 0
                || block_previous >= block_top) {
"####,
        r####"        if parser_cycle_state == 77 {
            ast_statement_start = current_start;
            ast_statement_line = current_line;
            ast_statement_column = current_column;
            ast_control_resume = 1;
            statement_mode = 4;
            value_top = 0;
            value_depth = 0;
            operator_top = 0;
            operator_depth = 0;
            paren_depth = 0;
            call_top = 0;
            held_active = 0;
            ref_pending = 0;
            arg_leading = 0;
            expecting_operand = 1;
            ast_control_lookahead = 0;
            parser_state = 3;
        }
        if parser_cycle_state == 72 {
            ast_control_kind = parser_record_0;
            ast_control_previous = parser_record_1;
            ast_control_resume = parser_record_2;
            ast_control_phase = parser_record_3;
            block_state = parser_record_4;
            ast_block_statements = parser_record_5;
            ast_block_first = parser_record_6;
            ast_block_cells = parser_record_7;
            ast_block_start = parser_record_8;
            ast_block_line = parser_record_9;
            ast_block_column = parser_record_10;
            body_root = parser_record_11;
            return_start = parser_record_12;
            return_line = parser_record_13;
            return_column = parser_record_14;
            ast_control_condition = parser_record_15;
            ast_control_then = parser_record_16;
            ast_control_start = parser_record_17;
            ast_control_line = parser_record_18;
            ast_control_column = parser_record_19;
            if ast_control_kind < 1 || ast_control_kind > 2
                || ast_control_previous < 0 || ast_control_previous >= block_top
                || ast_control_resume < 0 || ast_control_resume > 1
                || ast_control_phase < 1 || ast_control_phase > 4
                || block_state < 0 || block_state > 2 || block_depth <= 0 {
"####,
    ),
    (
        2112,
        6,
        r####"                block_top = block_previous;
                block_depth = block_depth - 1;
                block_state = block_state_saved;
                block_else = 0;
                if block_kind_popped == 1 {
                    block_else = 1;
"####,
        r####"                statement_mode = 0;
                match_active = 0;
                value_top = 0;
                value_depth = 0;
                operator_top = 0;
                operator_depth = 0;
                paren_depth = 0;
                call_top = 0;
                held_active = 0;
                ref_pending = 0;
                arg_leading = 0;
                expecting_operand = 1;
                if ast_control_kind == 1 && ast_control_phase == 1 {
                    ast_control_then = ast_block_node;
                    ast_control_phase = 2;
                    parser_append_0 = ast_control_kind;
                    parser_append_1 = ast_control_previous;
                    parser_append_2 = ast_control_resume;
                    parser_append_3 = ast_control_phase;
                    parser_append_4 = block_state;
                    parser_append_5 = ast_block_statements;
                    parser_append_6 = ast_block_first;
                    parser_append_7 = ast_block_cells;
                    parser_append_8 = ast_block_start;
                    parser_append_9 = ast_block_line;
                    parser_append_10 = ast_block_column;
                    parser_append_11 = body_root;
                    parser_append_12 = return_start;
                    parser_append_13 = return_line;
                    parser_append_14 = return_column;
                    parser_append_15 = ast_control_condition;
                    parser_append_16 = ast_control_then;
                    parser_append_17 = ast_control_start;
                    parser_append_18 = ast_control_line;
                    parser_append_19 = ast_control_column;
                    ast_control_depth_push = 0;
                    ast_control_after = 80;
                    parser_state = 52;
                } else {
                    block_top = ast_control_previous;
                    block_depth = block_depth - 1;
                    pending_node_kind = 33;
                    pending_node_payload = 0;
                    pending_node_left = ast_control_condition;
                    pending_node_right = ast_block_node;
                    pending_node_offset = ast_control_start;
                    pending_node_line = ast_control_line;
                    pending_node_column = ast_control_column;
                    ast_actual = 9;
                    ast_after = 73;
                    if ast_control_kind == 1 {
                        pending_node_kind = 31;
                        pending_node_left = ast_control_then;
                        pending_node_right = ast_block_node;
                        ast_actual = 7;
                        ast_after = 78;
                    }
                    parser_state = 60;
"####,
    ),
    (
        2119,
        0,
        r####""####,
        r####"            }
        }
        if parser_cycle_state == 80 {
            parser_token_after = 81;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }
        if parser_cycle_state == 81 {
            if current_kind == 8 {
"####,
    ),
    (
        2120,
        1,
        r####"                parser_state = 44;
"####,
        r####"                ast_control_lookahead = 0;
                parser_state = 50;
            } else {
                ast_control_lookahead = 1;
                block_top = ast_control_previous;
                block_depth = block_depth - 1;
                pending_node_kind = 31;
                pending_node_payload = 0;
                pending_node_left = ast_control_then;
                pending_node_right = 0;
                pending_node_offset = ast_control_start;
                pending_node_line = ast_control_line;
                pending_node_column = ast_control_column;
                ast_actual = 7;
                ast_after = 78;
                parser_state = 60;
            }
        }
        if parser_cycle_state == 78 {
            pending_node_kind = 32;
            pending_node_payload = 0;
            pending_node_left = ast_control_condition;
            pending_node_right = node_id;
            pending_node_offset = ast_control_start;
            pending_node_line = ast_control_line;
            pending_node_column = ast_control_column;
            ast_actual = 7;
            ast_after = 73;
            parser_state = 60;
        }
        if parser_cycle_state == 73 {
            if ast_control_resume == 1 {
                ast_block_node = node_id;
                parser_record_target = 3;
                parser_record_width = 20;
                parser_record_index = block_top;
                parser_record_field = 0;
                parser_record_byte = 0;
                parser_record_word = 0;
                parser_record_after = 72;
                parser_state = 31;
            } else {
                ast_statement_start = ast_control_start;
                ast_statement_line = ast_control_line;
                ast_statement_column = ast_control_column;
                ast_statement_after = 44;
                if ast_control_lookahead == 1 {
                    ast_statement_after = 45;
                }
                ast_after = 66;
                parser_state = 66;
"####,
    ),
    (
        2154,
        0,
        r####""####,
        r####"                    ast_match_start = current_start;
                    ast_match_line = current_line;
                    ast_match_column = current_column;
                    ast_match_first = 0;
                    ast_match_second = 0;
"####,
    ),
    (
        2207,
        0,
        r####""####,
        r####"                if match_cycle_step == 0 || match_cycle_step == 4
                    || match_cycle_step == 10 {
                    pending_node_kind = 34;
                    if match_cycle_step == 0 {
                        pending_node_kind = 2;
                    }
                    pending_node_payload = current_name_id;
                    pending_node_left = 0;
                    pending_node_right = 0;
                    pending_node_offset = current_start;
                    pending_node_line = current_line;
                    pending_node_column = current_column;
                    ast_actual = 1;
                    ast_after = 84;
                    if match_cycle_step == 0 {
                        ast_after = 83;
                    }
                    parser_state = 60;
                }
                if match_cycle_step == 2 || match_cycle_step == 8 {
                    ast_match_constructor = current_name_id;
                    ast_match_constructor_start = current_start;
                    ast_match_constructor_line = current_line;
                    ast_match_constructor_column = current_column;
                }
"####,
    ),
    (
        2211,
        0,
        r####""####,
        r####"                if match_cycle_step == 7 || match_cycle_step == 13 {
                    pending_node_kind = 35;
                    pending_node_payload = ast_match_constructor;
                    pending_node_left = ast_match_binder;
                    pending_node_right = expression_root;
                    pending_node_offset = ast_match_constructor_start;
                    pending_node_line = ast_match_constructor_line;
                    pending_node_column = ast_match_constructor_column;
                    ast_actual = 1;
                    ast_after = 85;
                    parser_state = 60;
                }
"####,
    ),
    (
        2212,
        2,
        r####"                    match_active = 0;
                    parser_state = 48;
"####,
        r####"                    pending_node_kind = 36;
                    pending_node_payload = 0;
                    pending_node_left = ast_match_first;
                    pending_node_right = ast_match_second;
                    pending_node_offset = ast_match_start;
                    pending_node_line = ast_match_line;
                    pending_node_column = ast_match_column;
                    ast_actual = 1;
                    ast_after = 86;
                    parser_state = 60;
"####,
    ),
    (
        2996,
        33,
        r####"                if node_count >= 65536 {
                    status = 14;
                    error_offset = return_start;
                    error_line = return_line;
                    error_column = return_column;
                    diagnostic_code = 65536;
                    diagnostic_actual = 6;
                    parser_running = 0;
                }
                if parser_running == 1 {
                    pending_node_kind = 18;
                    pending_node_payload = 0;
                    pending_node_left = body_root;
                    pending_node_right = 0;
                    pending_node_after = 22;
                    pending_node_offset = return_start;
                    pending_node_line = return_line;
                    pending_node_column = return_column;
                    parser_append_target = 4;
                    parser_append_width = 5;
                    parser_append_0 = node_count + 1;
                    parser_append_1 = return_start;
                    parser_append_2 = return_line;
                    parser_append_3 = return_column;
                    parser_append_4 = 6;
                    parser_append_field = 0;
                    parser_append_byte = 0;
                    parser_append_after = 33;
                    parser_append_offset = return_start;
                    parser_append_line = return_line;
                    parser_append_column = return_column;
                    parser_state = 32;
                }
"####,
        r####"                ast_statement_start = return_start;
                ast_statement_line = return_line;
                ast_statement_column = return_column;
                pending_node_kind = 18;
                pending_node_payload = 0;
                pending_node_left = body_root;
                pending_node_right = 0;
                pending_node_offset = return_start;
                pending_node_line = return_line;
                pending_node_column = return_column;
                ast_actual = 6;
                ast_statement_after = 67;
                ast_after = 66;
                parser_state = 60;
"####,
    ),
    (
        3381,
        0,
        r####""####,
        r####"        // Structural appends share the accepted origin-then-node transaction.
        // They never manufacture a value-stack entry or a scalar semantic fact.
        if parser_cycle_state == 60 {
            if node_count >= 65536 {
                status = 14;
                error_offset = pending_node_offset;
                error_line = pending_node_line;
                error_column = pending_node_column;
                diagnostic_code = 65536;
                diagnostic_actual = ast_actual;
                parser_running = 0;
            }
            if parser_running == 1 {
                pending_node_after = 61;
                parser_append_target = 4;
                parser_append_width = 5;
                parser_append_0 = node_count + 1;
                parser_append_1 = pending_node_offset;
                parser_append_2 = pending_node_line;
                parser_append_3 = pending_node_column;
                parser_append_4 = ast_actual;
                parser_append_field = 0;
                parser_append_byte = 0;
                parser_append_after = 33;
                parser_append_offset = pending_node_offset;
                parser_append_line = pending_node_line;
                parser_append_column = pending_node_column;
                parser_state = 32;
            }
        }
        if parser_cycle_state == 61 {
            node_count = node_count + 1;
            node_id = node_count;
            parser_state = ast_after;
        }
        if parser_cycle_state == 62 {
            ast_parameter_annotation = node_id;
            pending_node_kind = 25;
            pending_node_payload = param_name_id;
            pending_node_left = ast_parameter_annotation;
            pending_node_right = ast_function_parameters;
            pending_node_offset = ast_parameter_start;
            pending_node_line = ast_parameter_line;
            pending_node_column = ast_parameter_column;
            ast_actual = 1;
            ast_after = 63;
            parser_state = 60;
        }
        if parser_cycle_state == 63 {
            ast_function_parameters = node_id;
            parser_state = 1;
        }
        // Promote a held first statement only when it is really needed.
        if parser_cycle_state == 64 {
            pending_node_kind = 29;
            pending_node_payload = 0;
            pending_node_left = 0;
            pending_node_right = ast_block_first;
            pending_node_offset = parser_record_1;
            pending_node_line = parser_record_2;
            pending_node_column = parser_record_3;
            ast_actual = parser_record_4;
            ast_after = 65;
            parser_state = 60;
        }
        if parser_cycle_state == 65 {
            ast_block_cells = node_id;
            parser_state = ast_resume;
        }
        if parser_cycle_state == 66 {
            ast_statement_node = node_id;
            if ast_block_statements == 0 {
                ast_block_first = ast_statement_node;
                ast_block_statements = 1;
                statement_mode = 0;
                parser_state = ast_statement_after;
            } else {
                pending_node_kind = 29;
                pending_node_payload = 0;
                pending_node_left = ast_block_cells;
                pending_node_right = ast_statement_node;
                pending_node_offset = ast_statement_start;
                pending_node_line = ast_statement_line;
                pending_node_column = ast_statement_column;
                ast_actual = 1;
                if pending_node_kind == 29 {
                    parser_record_target = 5;
                    parser_record_width = 5;
                    parser_record_index = ast_statement_node;
                    parser_record_field = 0;
                    parser_record_byte = 0;
                    parser_record_word = 0;
                    parser_record_after = 75;
                    parser_state = 31;
                }
            }
        }
        if parser_cycle_state == 75 {
            ast_actual = parser_record_4;
            ast_after = 68;
            parser_state = 60;
        }
        if parser_cycle_state == 68 {
            ast_block_cells = node_id;
            ast_block_statements = ast_block_statements + 1;
            statement_mode = 0;
            parser_state = ast_statement_after;
        }
        if parser_cycle_state == 67 {
            if block_top == 0 && ast_function_parameters == 0
                && ast_block_statements == 1 {
                parser_record_target = 6;
                parser_record_width = 4;
                parser_record_index = ast_block_first;
                parser_record_field = 0;
                parser_record_byte = 0;
                parser_record_word = 0;
                parser_record_after = 79;
                parser_state = 31;
            } else {
                parser_state = 82;
            }
        }
        if parser_cycle_state == 79 {
            if parser_record_0 == 18 {
                ast_function_body = ast_block_first;
                pending_node_kind = 19;
                pending_node_payload = function_name_id;
                pending_node_left = ast_function_body;
                pending_node_right = item_previous;
                pending_node_offset = function_start;
                pending_node_line = function_line;
                pending_node_column = function_column;
                ast_actual = 3;
                ast_after = 69;
                parser_state = 60;
            } else {
                parser_state = 82;
            }
        }
        if parser_cycle_state == 82 {
                if ast_block_cells == 0 {
                    ast_resume = 82;
                    parser_record_target = 5;
                    parser_record_width = 5;
                    parser_record_index = ast_block_first;
                    parser_record_field = 0;
                    parser_record_byte = 0;
                    parser_record_word = 0;
                    parser_record_after = 64;
                    parser_state = 31;
                } else {
                    pending_node_kind = 30;
                    pending_node_payload = 0;
                    pending_node_left = ast_block_cells;
                    pending_node_right = 0;
                    pending_node_offset = ast_block_start;
                    pending_node_line = ast_block_line;
                    pending_node_column = ast_block_column;
                    ast_actual = 12;
                    ast_after = 70;
                    parser_state = 60;
                }
        }
        if parser_cycle_state == 69 {
            item_previous = node_id;
            parser_state = 20;
        }
        if parser_cycle_state == 70 {
            ast_block_node = node_id;
            // This real Block consumed its closing brace. A no-else child's
            // earlier lookahead is no longer available for parent dispatch.
            ast_control_lookahead = 0;
            if block_top == 0 {
                pending_node_kind = 38;
                pending_node_payload = 1;
                pending_node_left = ast_function_parameters;
                pending_node_right = ast_block_node;
                pending_node_offset = function_start;
                pending_node_line = function_line;
                pending_node_column = function_column;
                ast_actual = 3;
                ast_after = 71;
                parser_state = 60;
            } else {
                parser_record_target = 3;
                parser_record_width = 20;
                parser_record_index = block_top;
                parser_record_field = 0;
                parser_record_byte = 0;
                parser_record_word = 0;
                parser_record_after = 72;
                parser_state = 31;
            }
        }
        if parser_cycle_state == 71 {
            ast_function_body = node_id;
            pending_node_kind = 19;
            pending_node_payload = function_name_id;
            pending_node_left = ast_function_body;
            pending_node_right = item_previous;
            pending_node_offset = function_start;
            pending_node_line = function_line;
            pending_node_column = function_column;
            ast_actual = 3;
            ast_after = 69;
            parser_state = 60;
        }
        if parser_cycle_state == 74 {
            ast_statement_annotation = node_id;
            parser_state = 46;
        }
        if parser_cycle_state == 83 {
            ast_match_subject = node_id;
            parser_state = 42;
        }
        if parser_cycle_state == 84 {
            ast_match_binder = node_id;
            parser_state = 42;
        }
        if parser_cycle_state == 85 {
            if match_step == 8 {
                ast_match_first = node_id;
            } else {
                ast_match_second = node_id;
            }
            parser_state = 42;
        }
        if parser_cycle_state == 86 {
            pending_node_kind = 37;
            pending_node_payload = 0;
            pending_node_left = ast_match_subject;
            pending_node_right = node_id;
            pending_node_offset = ast_match_start;
            pending_node_line = ast_match_line;
            pending_node_column = ast_match_column;
            ast_actual = 1;
            ast_after = 87;
            parser_state = 60;
        }
        if parser_cycle_state == 87 {
            expression_root = node_id;
            match_active = 0;
            parser_state = 48;
        }

"####,
    ),
    (
        3429,
        0,
        r####""####,
        r####"                if parser_record_target == 5 {
                    parser_read_byte_value = result_value(bytes_get(&origins,
                        parser_read_offset));
                }
                if parser_record_target == 6 {
                    parser_read_byte_value = result_value(bytes_get(&nodes,
                        parser_read_offset));
                }
"####,
    ),
    (
        3430,
        1,
        r####"                    && parser_record_target != 3 && parser_record_target != 4 {
"####,
        r####"                    && parser_record_target != 3 && parser_record_target != 4
                    && parser_record_target != 5 && parser_record_target != 6 {
"####,
    ),
    (
        3478,
        0,
        r####""####,
        r####"                if parser_record_field == 5 {
                    parser_record_5 = parser_record_word;
                }
                if parser_record_field == 6 {
                    parser_record_6 = parser_record_word;
                }
                if parser_record_field == 7 {
                    parser_record_7 = parser_record_word;
                }
                if parser_record_field == 8 {
                    parser_record_8 = parser_record_word;
                }
                if parser_record_field == 9 {
                    parser_record_9 = parser_record_word;
                }
                if parser_record_field == 10 {
                    parser_record_10 = parser_record_word;
                }
                if parser_record_field == 11 {
                    parser_record_11 = parser_record_word;
                }
                if parser_record_field == 12 {
                    parser_record_12 = parser_record_word;
                }
                if parser_record_field == 13 {
                    parser_record_13 = parser_record_word;
                }
                if parser_record_field == 14 {
                    parser_record_14 = parser_record_word;
                }
                if parser_record_field == 15 {
                    parser_record_15 = parser_record_word;
                }
                if parser_record_field == 16 {
                    parser_record_16 = parser_record_word;
                }
                if parser_record_field == 17 {
                    parser_record_17 = parser_record_word;
                }
                if parser_record_field == 18 {
                    parser_record_18 = parser_record_word;
                }
                if parser_record_field == 19 {
                    parser_record_19 = parser_record_word;
                }
"####,
    ),
    (
        3503,
        0,
        r####""####,
        r####"            if parser_append_field == 5 {
                parser_append_word = parser_append_5;
            }
            if parser_append_field == 6 {
                parser_append_word = parser_append_6;
            }
            if parser_append_field == 7 {
                parser_append_word = parser_append_7;
            }
            if parser_append_field == 8 {
                parser_append_word = parser_append_8;
            }
            if parser_append_field == 9 {
                parser_append_word = parser_append_9;
            }
            if parser_append_field == 10 {
                parser_append_word = parser_append_10;
            }
            if parser_append_field == 11 {
                parser_append_word = parser_append_11;
            }
            if parser_append_field == 12 {
                parser_append_word = parser_append_12;
            }
            if parser_append_field == 13 {
                parser_append_word = parser_append_13;
            }
            if parser_append_field == 14 {
                parser_append_word = parser_append_14;
            }
            if parser_append_field == 15 {
                parser_append_word = parser_append_15;
            }
            if parser_append_field == 16 {
                parser_append_word = parser_append_16;
            }
            if parser_append_field == 17 {
                parser_append_word = parser_append_17;
            }
            if parser_append_field == 18 {
                parser_append_word = parser_append_18;
            }
            if parser_append_field == 19 {
                parser_append_word = parser_append_19;
            }
"####,
    ),
    (
        3591,
        1,
        r####"        || bytes_len(&blocks) < block_records * 12
"####,
        r####"        || bytes_len(&blocks) < block_records * 80
"####,
    ),
    (
        3598,
        1,
        r####"        || bytes_len(&blocks) != block_records * 12
"####,
        r####"        || bytes_len(&blocks) != block_records * 80
"####,
    ),
    (
        3743,
        0,
        r####""####,
        r####"    let mut validate_left_kind: int = 0;
    let mut validate_right_kind: int = 0;
    let mut validate_left_expression: int = 0;
    let mut validate_right_expression: int = 0;
    let mut validate_child_payload: int = 0;
    let mut validate_shape: int = 0;
"####,
    ),
    (
        3772,
        1,
        r####"        if validate_node_kind <= 0 || validate_node_kind > 23 {
"####,
        r####"        if validate_node_kind <= 0 || validate_node_kind > 38
            || validate_left < 0 || validate_left >= node_id
            || validate_right < 0 || validate_right >= node_id {
"####,
    ),
    (
        3775,
        2,
        r####"        if validate_node_kind == 1 {
            if validate_left != 0 || validate_right != 0 {
"####,
        r####"        validate_left_kind = 0;
        validate_right_kind = 0;
        validate_left_expression = 0;
        validate_right_expression = 0;
        if validate_left > 0 {
            word_offset = (validate_left - 1) * 16;
            byte_0 = result_value(bytes_get(&nodes, word_offset));
            byte_1 = result_value(bytes_get(&nodes, word_offset + 1));
            byte_2 = result_value(bytes_get(&nodes, word_offset + 2));
            byte_3 = result_value(bytes_get(&nodes, word_offset + 3));
            if byte_0 < 0 || byte_1 < 0 || byte_2 < 0 || byte_3 < 0 || byte_3 > 127 {
"####,
    ),
    (
        3779,
        3,
        r####"        } else if validate_node_kind == 2 {
            if validate_payload <= 0 || validate_payload > name_count
                || validate_left != 0 || validate_right != 0 {
"####,
        r####"            validate_left_kind = byte_0 + byte_1 * 256 + byte_2 * 65536 + byte_3 * 16777216;
        }
        if validate_right > 0 {
            word_offset = (validate_right - 1) * 16;
            byte_0 = result_value(bytes_get(&nodes, word_offset));
            byte_1 = result_value(bytes_get(&nodes, word_offset + 1));
            byte_2 = result_value(bytes_get(&nodes, word_offset + 2));
            byte_3 = result_value(bytes_get(&nodes, word_offset + 3));
            if byte_0 < 0 || byte_1 < 0 || byte_2 < 0 || byte_3 < 0 || byte_3 > 127 {
"####,
    ),
    (
        3784,
        6,
        r####"        } else if validate_node_kind == 3 || validate_node_kind == 4
            || validate_node_kind == 18 || validate_node_kind == 22
            || validate_node_kind == 23 {
            if validate_payload != 0 || validate_left <= 0
                || validate_left >= node_id || validate_right != 0 {
                return 78;
"####,
        r####"            validate_right_kind = byte_0 + byte_1 * 256 + byte_2 * 65536 + byte_3 * 16777216;
        }
        if (validate_left_kind >= 1 && validate_left_kind <= 17)
            || validate_left_kind == 20 || validate_left_kind == 22
            || validate_left_kind == 23 || validate_left_kind == 37 {
            validate_left_expression = 1;
        }
        if (validate_right_kind >= 1 && validate_right_kind <= 17)
            || validate_right_kind == 20 || validate_right_kind == 22
            || validate_right_kind == 23 || validate_right_kind == 37 {
            validate_right_expression = 1;
        }
        validate_shape = 0;
        if validate_node_kind == 1 {
            if validate_left == 0 && validate_right == 0 {
                validate_shape = 1;
"####,
    ),
    (
        3791,
        4,
        r####"        } else if validate_node_kind >= 5 && validate_node_kind <= 17 {
            if validate_payload != 0 || validate_left <= 0 || validate_right <= 0
                || validate_left >= node_id || validate_right >= node_id {
                return 78;
"####,
        r####"        }
        if validate_node_kind == 2 || validate_node_kind == 34 {
            if validate_payload > 0 && validate_payload <= name_count && validate_left == 0 && validate_right == 0 {
                validate_shape = 1;
"####,
    ),
    (
        3796,
        5,
        r####"        } else if validate_node_kind == 20 {
            if validate_payload <= 0 || validate_payload > name_count
                || validate_left < 0 || validate_left >= node_id
                || validate_right != 0 {
                return 78;
"####,
        r####"        }
        if validate_node_kind == 3 || validate_node_kind == 4 || validate_node_kind == 18 || validate_node_kind == 22 || validate_node_kind == 23 {
            if validate_payload == 0 && validate_left_expression == 1 && validate_right == 0 {
                validate_shape = 1;
"####,
    ),
    (
        3802,
        5,
        r####"        } else if validate_node_kind == 21 {
            if validate_payload != 0 || validate_left <= 0
                || validate_left >= node_id || validate_right < 0
                || validate_right >= node_id {
                return 78;
"####,
        r####"        }
        if validate_node_kind >= 5 && validate_node_kind <= 17 {
            if validate_payload == 0 && validate_left_expression == 1 && validate_right_expression == 1 {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 19 {
            if validate_payload > 0 && validate_payload <= name_count && (validate_left_kind == 18 || validate_left_kind == 38) && (validate_right == 0 || validate_right_kind == 19) {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 20 {
            if validate_payload > 0 && validate_payload <= name_count && (validate_left == 0 || validate_left_kind == 21) && validate_right == 0 {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 21 {
            if validate_payload == 0 && validate_left_expression == 1 && (validate_right == 0 || validate_right_kind == 21) {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 24 {
            if validate_payload >= 1 && validate_payload <= 3 && validate_left == 0 && validate_right == 0 {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 25 {
            if validate_payload > 0 && validate_payload <= name_count && validate_left_kind == 24 && (validate_right == 0 || validate_right_kind == 25) {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 26 || validate_node_kind == 27 {
            if validate_payload > 0 && validate_payload <= name_count && validate_left_kind == 24 && validate_right_expression == 1 {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 28 {
            if validate_payload > 0 && validate_payload <= name_count && validate_left_expression == 1 && validate_right == 0 {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 29 {
            if validate_payload == 0 && (validate_left == 0 || validate_left_kind == 29) && (validate_right_kind == 18 || validate_right_kind == 26 || validate_right_kind == 27 || validate_right_kind == 28 || validate_right_kind == 32 || validate_right_kind == 33) {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 30 {
            if validate_payload == 0 && validate_left_kind == 29 && validate_right == 0 {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 31 {
            if validate_payload == 0 && validate_left_kind == 30 && (validate_right == 0 || validate_right_kind == 30 || validate_right_kind == 32) {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 32 {
            if validate_payload == 0 && validate_left_expression == 1 && validate_right_kind == 31 {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 33 {
            if validate_payload == 0 && validate_left_expression == 1 && validate_right_kind == 30 {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 35 {
            if validate_payload > 0 && validate_payload <= name_count && validate_left_kind == 34 && validate_right_expression == 1 {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 36 {
            if validate_payload == 0 && validate_left_kind == 35 && validate_right_kind == 35 {
                validate_shape = 1;
"####,
    ),
    (
        3808,
        4,
        r####"        } else if validate_node_kind == 19 {
            if validate_payload <= 0 || validate_payload > name_count
                || validate_left <= 0 || validate_left >= node_id
                || validate_right < 0 || validate_right >= node_id {
"####,
        r####"        }
        if validate_node_kind == 37 {
            if validate_payload == 0 && validate_left_kind == 2 && validate_right_kind == 36 {
                validate_shape = 1;
            }
        }
        if validate_node_kind == 38 {
            if validate_payload == 1 && (validate_left == 0 || validate_left_kind == 25) && validate_right_kind == 30 {
                validate_shape = 1;
            }
        }
        if validate_shape == 1 && validate_node_kind == 25 {
            word_offset = (validate_left - 1) * 16 + 4;
            byte_0 = result_value(bytes_get(&nodes, word_offset));
            byte_1 = result_value(bytes_get(&nodes, word_offset + 1));
            byte_2 = result_value(bytes_get(&nodes, word_offset + 2));
            byte_3 = result_value(bytes_get(&nodes, word_offset + 3));
            if byte_0 < 0 || byte_1 < 0 || byte_2 < 0 || byte_3 < 0 || byte_3 > 127 {
"####,
    ),
    (
        3814,
        1,
        r####"        } else {
"####,
        r####"            validate_child_payload = byte_0 + byte_1 * 256 + byte_2 * 65536 + byte_3 * 16777216;
            if validate_child_payload < 1 || validate_child_payload > 2 {
                validate_shape = 0;
            }
        }
        if validate_shape == 0 {
"####,
    ),
    (
        3880,
        1,
        r####"        validate_parameter = validate_parameter + 1;
"####,
        r####"        validate_parameter = validate_parameter + 1;
    }
    checksum = checksum_step(checksum, parameter_count);

    if status != expected_status || error_offset != expected_offset
        || error_line != expected_line || error_column != expected_column
        || diagnostic_code != expected_code || diagnostic_actual != expected_actual
        || name_count != expected_names || token_count != expected_tokens
        || node_count != expected_nodes || root != expected_root
        || parameter_count != expected_parameters
        || checksum != expected_checksum {
        return 80;
    }

    // Validate the parallel origin sidecar against source, tokens, and nodes.
    // One append-only interval start per occurrence proves complete ownership.
    // This structural authority precedes origin authentication and all meaning.
    let mut ast_starts: ByteBuffer = bytes_new();
    let mut ast_interval_id: int = 1;
    let mut ast_interval_left: int = 0;
    let mut ast_interval_right: int = 0;
    let mut ast_interval_low: int = 0;
    let mut ast_interval_high: int = 0;
    let mut ast_interval_start: int = 0;
    let mut ast_interval_later_start: int = 0;
    let mut ast_interval_byte: int = 0;
    while status == 0 && semantic_status == 0 && ast_interval_id <= node_count {
        semantic_node = ast_interval_id;
        semantic_offset = 0 - 1;
        semantic_line = 0;
        semantic_column = 0;
        semantic_code = 1;
        word_offset = (ast_interval_id - 1) * 16 + 8;
        byte_0 = result_value(bytes_get(&nodes, word_offset));
        byte_1 = result_value(bytes_get(&nodes, word_offset + 1));
        byte_2 = result_value(bytes_get(&nodes, word_offset + 2));
        byte_3 = result_value(bytes_get(&nodes, word_offset + 3));
        if byte_0 < 0 || byte_1 < 0 || byte_2 < 0 || byte_3 < 0 || byte_3 > 127 {
            semantic_status = 27;
        } else {
            ast_interval_left = byte_0 + byte_1 * 256 + byte_2 * 65536 + byte_3 * 16777216;
        }
        word_offset = (ast_interval_id - 1) * 16 + 12;
        byte_0 = result_value(bytes_get(&nodes, word_offset));
        byte_1 = result_value(bytes_get(&nodes, word_offset + 1));
        byte_2 = result_value(bytes_get(&nodes, word_offset + 2));
        byte_3 = result_value(bytes_get(&nodes, word_offset + 3));
        if byte_0 < 0 || byte_1 < 0 || byte_2 < 0 || byte_3 < 0 || byte_3 > 127 {
            semantic_status = 27;
        } else {
            ast_interval_right = byte_0 + byte_1 * 256 + byte_2 * 65536 + byte_3 * 16777216;
        }
        ast_interval_low = ast_interval_left;
        ast_interval_high = ast_interval_right;
        if ast_interval_low > ast_interval_high {
            ast_interval_low = ast_interval_right;
            ast_interval_high = ast_interval_left;
        }
        ast_interval_start = ast_interval_id;
        if semantic_status == 0 && ast_interval_high > 0 {
            if ast_interval_high != ast_interval_id - 1 {
                semantic_status = 27;
            } else {
                word_offset = (ast_interval_high - 1) * 4;
                byte_0 = result_value(bytes_get(&ast_starts, word_offset));
                byte_1 = result_value(bytes_get(&ast_starts, word_offset + 1));
                byte_2 = result_value(bytes_get(&ast_starts, word_offset + 2));
                byte_3 = result_value(bytes_get(&ast_starts, word_offset + 3));
                if byte_0 < 0 || byte_1 < 0 || byte_2 < 0 || byte_3 < 0 || byte_3 > 127 {
                    semantic_status = 27;
                } else {
                    ast_interval_later_start = byte_0 + byte_1 * 256 + byte_2 * 65536 + byte_3 * 16777216;
                }
                ast_interval_start = ast_interval_later_start;
                if semantic_status == 0 && ast_interval_low > 0 {
                    if ast_interval_low == ast_interval_high
                        || ast_interval_low != ast_interval_later_start - 1 {
                        semantic_status = 27;
                    } else {
                        word_offset = (ast_interval_low - 1) * 4;
                        byte_0 = result_value(bytes_get(&ast_starts, word_offset));
                        byte_1 = result_value(bytes_get(&ast_starts, word_offset + 1));
                        byte_2 = result_value(bytes_get(&ast_starts, word_offset + 2));
                        byte_3 = result_value(bytes_get(&ast_starts, word_offset + 3));
                        if byte_0 < 0 || byte_1 < 0 || byte_2 < 0 || byte_3 < 0 || byte_3 > 127 {
                            semantic_status = 27;
                        } else {
                            ast_interval_start = byte_0 + byte_1 * 256 + byte_2 * 65536 + byte_3 * 16777216;
                        }
                    }
                }
            }
        }
        ast_interval_byte = 0;
        while semantic_status == 0 && ast_interval_byte < 4 {
            byte = word_byte_0(ast_interval_start);
            if ast_interval_byte == 1 {
                byte = word_byte_1(ast_interval_start);
            }
            if ast_interval_byte == 2 {
                byte = word_byte_2(ast_interval_start);
            }
            if ast_interval_byte == 3 {
                byte = word_byte_3(ast_interval_start);
            }
            push_result = result_value(bytes_push(&mut ast_starts, byte));
            if push_result < 0 {
                semantic_status = 26;
                semantic_code = 4;
            }
            ast_interval_byte = ast_interval_byte + 1;
        }
        ast_interval_id = ast_interval_id + 1;
"####,
    ),
    (
        3882,
        10,
        r####"    checksum = checksum_step(checksum, parameter_count);

    if status != expected_status || error_offset != expected_offset
        || error_line != expected_line || error_column != expected_column
        || diagnostic_code != expected_code || diagnostic_actual != expected_actual
        || name_count != expected_names || token_count != expected_tokens
        || node_count != expected_nodes || root != expected_root
        || parameter_count != expected_parameters
        || checksum != expected_checksum {
        return 80;
"####,
        r####"    if status == 0 && semantic_status == 0 {
        if ast_interval_start != 1 || root != node_count
            || bytes_len(&ast_starts) != node_count * 4 {
            semantic_status = 27;
            semantic_node = root;
            semantic_offset = 0 - 1;
            semantic_line = 0;
            semantic_column = 0;
            semantic_code = 1;
        }
"####,
    ),
    (
        3893,
        2,
        r####"
    // Validate the parallel origin sidecar against source, tokens, and nodes.
"####,
        r####""####,
    ),
    (
        3896,
        0,
        r####""####,
        r####"    let mut origin_auth_attempted: int = 0;
"####,
    ),
    (
        3913,
        0,
        r####""####,
        r####"    let mut origin_search_low: int = 0;
    let mut origin_search_high: int = 0;
    let mut origin_search_middle: int = 0;
    let mut origin_node_right: int = 0;
"####,
    ),
    (
        3914,
        0,
        r####""####,
        r####"        origin_auth_attempted = 1;
"####,
    ),
    (
        3967,
        21,
        r####"        if semantic_status == 0 {
            location_index = 0;
            location_line = 1;
            location_column = 1;
            while semantic_status == 0 && location_index < origin_offset {
                byte = result_value(bytes_get(&source, location_index));
                if byte < 0 {
                    semantic_status = 27;
                } else if byte == 10 {
                    location_line = location_line + 1;
                    location_column = 1;
                } else {
                    location_column = location_column + 1;
                }
                location_index = location_index + 1;
            }
            if semantic_status == 0 && (location_line != origin_line
                || location_column != origin_column) {
                semantic_status = 27;
            }
        }
"####,
        r####""####,
    ),
    (
        4069,
        0,
        r####""####,
        r####"                if origin_node_kind == 24 || origin_node_kind == 25
                    || origin_node_kind == 28 || origin_node_kind == 34
                    || origin_node_kind == 35 || origin_node_kind == 36
                    || origin_node_kind == 37 {
                    origin_expected_kind = 1;
                }
                if origin_node_kind == 26 || origin_node_kind == 27 {
                    origin_expected_kind = 4;
                }
                if origin_node_kind == 30 {
                    origin_expected_kind = 12;
                }
                if origin_node_kind == 31 || origin_node_kind == 32 {
                    origin_expected_kind = 7;
                }
                if origin_node_kind == 33 {
                    origin_expected_kind = 9;
                }
                if origin_node_kind == 38 {
                    origin_expected_kind = 3;
                }
                if origin_node_kind == 29 {
                    word_offset = origin_index * 16 + 12;
                    origin_node_right = result_value(bytes_get(&nodes, word_offset))
                        + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                        + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                        + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
                    word_offset = (origin_node_right - 1) * 20 + 16;
                    origin_expected_kind = result_value(bytes_get(&origins, word_offset))
                        + result_value(bytes_get(&origins, word_offset + 1)) * 256
                        + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                        + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
                }
"####,
    ),
    (
        4075,
        1,
        r####"            origin_token_index = 0;
"####,
        r####"            // Token coordinates were authenticated once against source above.
            // Match the exact tuple with logarithmic token selection.
            origin_search_low = 0;
            origin_search_high = token_count;
            while semantic_status == 0 && origin_search_low < origin_search_high {
                origin_search_middle = origin_search_low
                    + signed_quotient(origin_search_high - origin_search_low, 2);
                word_offset = origin_search_middle * 24 + 4;
                origin_token_start = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if origin_token_start < origin_offset {
                    origin_search_low = origin_search_middle + 1;
                } else {
                    origin_search_high = origin_search_middle;
                }
            }
            origin_token_index = origin_search_low;
"####,
    ),
    (
        4077,
        1,
        r####"            while origin_token_index < token_count && origin_token_found == 0 {
"####,
        r####"            if origin_token_index < token_count {
"####,
    ),
    (
        4079,
        4,
        r####"                origin_token_record_kind = 0;
                origin_token_start = 0;
                origin_token_line = 0;
                origin_token_column = 0;
"####,
        r####""####,
    ),
    (
        4089,
        2,
        r####"                    if byte_0 < 0 || byte_1 < 0 || byte_2 < 0 || byte_3 < 0
                        || byte_3 > 127 {
"####,
        r####"                    if byte_0 < 0 || byte_1 < 0 || byte_2 < 0 || byte_3 < 0 || byte_3 > 127 {
"####,
    ),
    (
        4113,
        1,
        r####"                origin_token_index = origin_token_index + 1;
"####,
        r####""####,
    ),
    (
        4122,
        0,
        r####""####,
        r####"

    // Bounded brace ownership: every closing event is consumed once in AST
    // postorder. Coordinates are verified above; new resource failures remain
    // explicitly unlocated until complete source-role authentication.
    let mut ast_brace_history: ByteBuffer = bytes_new();
    let mut ast_ends: ByteBuffer = bytes_new();
    let mut map_index: int = 1;
    let mut map_kind: int = 0;
    let mut map_left: int = 0;
    let mut map_right: int = 0;
    let mut map_word: int = 0;
    let mut map_end: int = 0;
    let mut map_inherit: int = 0;
    let mut map_consume: int = 0;
    let mut map_token: int = 0;
    let mut map_token_kind: int = 0;
    let mut map_open: int = 0;
    let mut map_expected_open: int = 0;
    let mut map_previous: int = 0;
    let mut map_active: int = 0;
    let mut map_history_count: int = 0;
    let mut map_closures: int = 0;
    let mut map_found: int = 0;
    let mut map_scan: int = 0;
    let mut map_field: int = 0;
    let mut map_byte: int = 0;
    let mut map_append: int = 0;
    while status == 0 && semantic_status == 0 && map_index <= node_count {
        word_offset = (map_index - 1) * 16 + 0;
            map_kind = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (map_index - 1) * 16 + 8;
            map_left = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (map_index - 1) * 16 + 12;
            map_right = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        semantic_node = map_index;
        word_offset = (map_index - 1) * 20 + 4;
            semantic_offset = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (map_index - 1) * 20 + 8;
            semantic_line = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (map_index - 1) * 20 + 12;
            semantic_column = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        semantic_code = 1;
        map_end = 0;
        map_inherit = 0;
        map_consume = 0;
        if map_kind == 30 || map_kind == 37 { map_consume = 1; }
        if map_kind == 19 {
            word_offset = (map_left - 1) * 16 + 0;
            map_word = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
            if map_word == 18 { map_consume = 1; }
            else { map_inherit = map_left; }
        }
        if map_kind == 29 || map_kind == 32 || map_kind == 33
            || map_kind == 38 { map_inherit = map_right; }
        if map_kind == 18 { map_inherit = map_left; }
        if map_kind == 31 {
            map_inherit = map_left;
            if map_right > 0 { map_inherit = map_right; }
        }
        if map_inherit > 0 {
            word_offset = (map_inherit - 1) * 4;
            map_end = result_value(bytes_get(&ast_ends, word_offset))
                + result_value(bytes_get(&ast_ends, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_ends, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_ends, word_offset + 3)) * 16777216;
            if map_end < 0 || map_end > token_count { semantic_status = 27; }
        }
        if map_consume == 1 {
            origin_search_low = 0;
            origin_search_high = token_count;
            while origin_search_low < origin_search_high {
                origin_search_middle = origin_search_low
                    + signed_quotient(origin_search_high - origin_search_low, 2);
                word_offset = (origin_search_middle) * 24 + 4;
            map_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if map_word < semantic_offset {
                    origin_search_low = origin_search_middle + 1;
                } else { origin_search_high = origin_search_middle; }
            }
            map_expected_open = origin_search_low;
            if map_kind == 37 { map_expected_open = map_expected_open + 2; }
            if map_kind == 19 {
                map_scan = map_expected_open + 1;
                map_expected_open = 0 - 1;
                while map_scan < token_count && map_expected_open < 0 {
                    word_offset = (map_scan) * 24 + 0;
            map_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                    if map_word == 12 { map_expected_open = map_scan; }
                    map_scan = map_scan + 1;
                }
            }
            word_offset = (map_expected_open) * 24 + 0;
            map_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if map_expected_open < 0 || map_expected_open >= token_count
                || map_word != 12 { semantic_status = 27; }
            map_found = 0;
            while semantic_status == 0 && map_token < token_count && map_found == 0 {
                word_offset = (map_token) * 24 + 0;
            map_token_kind = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if map_token_kind == 12 {
                    if map_history_count >= node_count { semantic_status = 27; }

                map_field = 0;
                while semantic_status == 0 && map_field < 2 {
                    map_append = map_end;
                    map_append = map_active;
                    if map_field == 1 { map_append = map_token; }
                    map_byte = 0;
                    while semantic_status == 0 && map_byte < 4 {
                        byte = word_byte_0(map_append);
                        if map_byte == 1 { byte = word_byte_1(map_append); }
                        if map_byte == 2 { byte = word_byte_2(map_append); }
                        if map_byte == 3 { byte = word_byte_3(map_append); }
                        push_result = result_value(bytes_push(&mut ast_brace_history, byte));
                        if push_result < 0 {
                            semantic_status = 26;
                            semantic_code = 5;
                            semantic_offset = 0 - 1;
                            semantic_line = 0;
                            semantic_column = 0;
                        }
                        map_byte = map_byte + 1;
                    }
                    map_field = map_field + 1;
                }

                    if semantic_status == 0 {
                        map_history_count = map_history_count + 1;
                        map_active = map_history_count;
                    }
                }
                if map_token_kind == 13 {
                    if map_active <= 0 || map_active > map_history_count {
                        semantic_status = 27;
                    } else {
                        word_offset = (map_active - 1) * 8;
            map_previous = result_value(bytes_get(&ast_brace_history, word_offset))
                + result_value(bytes_get(&ast_brace_history, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_brace_history, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_brace_history, word_offset + 3)) * 16777216;
                        word_offset = (map_active - 1) * 8 + 4;
            map_open = result_value(bytes_get(&ast_brace_history, word_offset))
                + result_value(bytes_get(&ast_brace_history, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_brace_history, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_brace_history, word_offset + 3)) * 16777216;
                        if map_previous < 0 || map_previous >= map_active
                            || map_open != map_expected_open {
                            semantic_status = 27;
                        } else {
                            map_active = map_previous;
                            map_closures = map_closures + 1;
                            map_found = 1;
                            map_end = map_token + 1;
                        }
                    }
                }
                map_token = map_token + 1;
            }
            if semantic_status == 0 && map_found == 0 { semantic_status = 27; }
        }

                map_field = 0;
                while semantic_status == 0 && map_field < 1 {
                    map_append = map_end;

                    map_byte = 0;
                    while semantic_status == 0 && map_byte < 4 {
                        byte = word_byte_0(map_append);
                        if map_byte == 1 { byte = word_byte_1(map_append); }
                        if map_byte == 2 { byte = word_byte_2(map_append); }
                        if map_byte == 3 { byte = word_byte_3(map_append); }
                        push_result = result_value(bytes_push(&mut ast_ends, byte));
                        if push_result < 0 {
                            semantic_status = 26;
                            semantic_code = 6;
                            semantic_offset = 0 - 1;
                            semantic_line = 0;
                            semantic_column = 0;
                        }
                        map_byte = map_byte + 1;
                    }
                    map_field = map_field + 1;
                }

        map_index = map_index + 1;
    }
    if status == 0 && semantic_status == 0 {
        if map_active != 0 || map_closures != map_history_count
            || map_history_count > node_count
            || bytes_len(&ast_brace_history) != map_history_count * 8
            || bytes_len(&ast_ends) != node_count * 4 { semantic_status = 27; }
        while semantic_status == 0 && map_token < token_count {
            word_offset = (map_token) * 24 + 0;
            map_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if map_word == 12 || map_word == 13 { semantic_status = 27; }
            map_token = map_token + 1;
        }
    }
    // Authenticate source roles only after every origin tuple is verified.
    // Name identity does not substitute for the owned declaration occurrence.
    let mut auth_index: int = 0;
    let mut auth_field: int = 0;
    let mut auth_word: int = 0;
    let mut auth_kind: int = 0;
    let mut auth_payload: int = 0;
    let mut auth_left: int = 0;
    let mut auth_right: int = 0;
    let mut auth_offset: int = 0;
    let mut auth_token: int = 0;
    let mut auth_neighbor: int = 0;
    let mut auth_link: int = 0;
    let mut auth_other: int = 0;
    let mut auth_related: int = 0;
    let mut auth_type: int = 0;
    let mut auth_type_token: int = 0;
    let mut auth_type_count: int = 0;
    let mut auth_spelling: int = 0;
    let mut auth_spelling_start: int = 0;
    let mut auth_spelling_length: int = 0;
    let mut auth_spelling_index: int = 0;
    let mut auth_spelling_byte: int = 0;
    let mut auth_expected_byte: int = 0;
    let mut auth_open: int = 0;
    let mut auth_close: int = 0;
    let mut auth_depth: int = 0;
    let mut auth_scan: int = 0;
    let mut auth_name_token: int = 0;
    let mut auth_name_start: int = 0;
    let mut auth_name_length: int = 0;
    let mut auth_body: int = 0;
    let mut auth_else: int = 0;
    while status == 0 && semantic_status == 0 && auth_index < node_count {
        word_offset = auth_index * 16;
            auth_kind = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = auth_index * 16 + 4;
            auth_payload = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = auth_index * 16 + 8;
            auth_left = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = auth_index * 16 + 12;
            auth_right = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (auth_index + 1 - 1) * 20 + 4;
            auth_offset = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        semantic_node = auth_index + 1;
        semantic_offset = auth_offset;
        word_offset = (auth_index + 1 - 1) * 20 + 8;
            semantic_line = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (auth_index + 1 - 1) * 20 + 12;
            semantic_column = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        semantic_code = 1;
        origin_search_low = 0;
        origin_search_high = token_count;
        while origin_search_low < origin_search_high {
            origin_search_middle = origin_search_low
                + signed_quotient(origin_search_high - origin_search_low, 2);
            word_offset = (origin_search_middle) * 24 + 4;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word < auth_offset {
                origin_search_low = origin_search_middle + 1;
            } else {
                origin_search_high = origin_search_middle;
            }
        }
        auth_token = origin_search_low;
        if auth_kind == 2 || auth_kind == 25 || auth_kind == 28
            || auth_kind == 34 || auth_kind == 35 {
            word_offset = (auth_token) * 24 + 20;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != auth_payload {
                semantic_status = 27;
            }
        }
        if auth_kind == 19 {
            word_offset = (auth_token + 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_token + 1) * 24 + 20;
            auth_neighbor = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != 1 || auth_neighbor != auth_payload {
                semantic_status = 27;
            }
        }
        if auth_kind == 20 {
            word_offset = (auth_token - 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_token - 1) * 24 + 20;
            auth_neighbor = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != 1 || auth_neighbor != auth_payload {
                semantic_status = 27;
            }
        }


        // Authenticate occurrence bytes against the declared interner spelling,
        // rather than trusting an in-range token name ID alone.
        auth_name_token = 0 - 1;
        if auth_kind == 2 || auth_kind == 25 || auth_kind == 28
            || auth_kind == 34 || auth_kind == 35 {
            auth_name_token = auth_token;
        }
        if auth_kind == 19 { auth_name_token = auth_token + 1; }
        if auth_kind == 20 { auth_name_token = auth_token - 1; }
        if auth_kind == 26 { auth_name_token = auth_token + 1; }
        if auth_kind == 27 { auth_name_token = auth_token + 2; }
        if auth_name_token >= 0 {
            word_offset = (auth_payload - 1) * 8;
            auth_name_start = result_value(bytes_get(&names, word_offset))
                + result_value(bytes_get(&names, word_offset + 1)) * 256
                + result_value(bytes_get(&names, word_offset + 2)) * 65536
                + result_value(bytes_get(&names, word_offset + 3)) * 16777216;
            word_offset = (auth_payload - 1) * 8 + 4;
            auth_name_length = result_value(bytes_get(&names, word_offset))
                + result_value(bytes_get(&names, word_offset + 1)) * 256
                + result_value(bytes_get(&names, word_offset + 2)) * 65536
                + result_value(bytes_get(&names, word_offset + 3)) * 16777216;
            word_offset = (auth_name_token) * 24 + 4;
            auth_spelling_start = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_name_token) * 24 + 8;
            auth_spelling_length = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_name_length != auth_spelling_length { semantic_status = 27; }
            auth_spelling_index = 0;
            while semantic_status == 0 && auth_spelling_index < auth_name_length {
                if result_value(bytes_get(&source, auth_name_start + auth_spelling_index))
                    != result_value(bytes_get(&source,
                        auth_spelling_start + auth_spelling_index)) {
                    semantic_status = 27;
                }
                auth_spelling_index = auth_spelling_index + 1;
            }
        }
        // Authenticate the complete allowed annotation or contextual spelling.
        auth_type = 0;
        if auth_kind == 24 {
            auth_type = auth_payload;
        }
        if auth_kind == 36 || auth_kind == 37 {
            auth_type = 4;
        }
        auth_type_count = 0;
        auth_type_token = auth_token;
        while semantic_status == 0 && auth_type > 0 {
            word_offset = auth_type_token * 24;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != 1 { semantic_status = 27; }
            auth_spelling = auth_type;
            if auth_type_count > 0 {
                auth_spelling = 1;
            }
            word_offset = (auth_type_token) * 24 + 4;
            auth_spelling_start = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_type_token) * 24 + 8;
            auth_spelling_length = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            auth_other = 3;
            if auth_spelling == 2 {
                auth_other = 6;
            }
            if auth_spelling == 3 {
                auth_other = 10;
            }
            if auth_spelling == 4 {
                auth_other = 5;
            }
            if auth_spelling_length != auth_other {
                semantic_status = 27;
            }
            auth_spelling_index = 0;
            while semantic_status == 0 && auth_spelling_index < auth_other {
                auth_expected_byte = 0;
                if auth_spelling == 1 {
                    if auth_spelling_index == 0 { auth_expected_byte = 105; }
                    if auth_spelling_index == 1 { auth_expected_byte = 110; }
                    if auth_spelling_index == 2 { auth_expected_byte = 116; }
                }
                if auth_spelling == 2 {
                    if auth_spelling_index == 0 { auth_expected_byte = 82; }
                    if auth_spelling_index == 1 { auth_expected_byte = 101; }
                    if auth_spelling_index == 2 { auth_expected_byte = 115; }
                    if auth_spelling_index == 3 { auth_expected_byte = 117; }
                    if auth_spelling_index == 4 { auth_expected_byte = 108; }
                    if auth_spelling_index == 5 { auth_expected_byte = 116; }
                }
                if auth_spelling == 3 {
                    if auth_spelling_index == 0 { auth_expected_byte = 66; }
                    if auth_spelling_index == 1 { auth_expected_byte = 121; }
                    if auth_spelling_index == 2 { auth_expected_byte = 116; }
                    if auth_spelling_index == 3 { auth_expected_byte = 101; }
                    if auth_spelling_index == 4 { auth_expected_byte = 66; }
                    if auth_spelling_index == 5 { auth_expected_byte = 117; }
                    if auth_spelling_index == 6 { auth_expected_byte = 102; }
                    if auth_spelling_index == 7 { auth_expected_byte = 102; }
                    if auth_spelling_index == 8 { auth_expected_byte = 101; }
                    if auth_spelling_index == 9 { auth_expected_byte = 114; }
                }
                if auth_spelling == 4 {
                    if auth_spelling_index == 0 { auth_expected_byte = 109; }
                    if auth_spelling_index == 1 { auth_expected_byte = 97; }
                    if auth_spelling_index == 2 { auth_expected_byte = 116; }
                    if auth_spelling_index == 3 { auth_expected_byte = 99; }
                    if auth_spelling_index == 4 { auth_expected_byte = 104; }
                }
                auth_spelling_byte = result_value(bytes_get(&source,
                    auth_spelling_start + auth_spelling_index));
                if auth_spelling_byte != auth_expected_byte {
                    semantic_status = 27;
                }
                auth_spelling_index = auth_spelling_index + 1;
            }
            if auth_type == 2 && auth_type_count == 0 {
                word_offset = (auth_token + 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                word_offset = (auth_token + 3) * 24 + 0;
            auth_neighbor = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                word_offset = (auth_token + 5) * 24 + 0;
            auth_other = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if auth_token + 5 >= token_count || auth_word != 29
                    || auth_neighbor != 16 || auth_other != 31 {
                    semantic_status = 27;
                }
                auth_type_count = 1;
                auth_type_token = auth_token + 2;
            } else if auth_type == 2 && auth_type_count == 1 {
                auth_type_count = 2;
                auth_type_token = auth_token + 4;
            } else {
                auth_type = 0;
            }
        }

        // Declaration adjacency pins each annotation and binder to its owner.
        auth_neighbor = 0;
        if auth_kind == 25 { auth_neighbor = auth_token; }
        if auth_kind == 26 { auth_neighbor = auth_token + 1; }
        if auth_kind == 27 {
            word_offset = (auth_token + 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != 5 { semantic_status = 27; }
            auth_neighbor = auth_token + 2;
        }
        if auth_kind == 25 || auth_kind == 26 || auth_kind == 27 {
            word_offset = (auth_neighbor) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_neighbor) * 24 + 20;
            auth_other = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != 1 || auth_other != auth_payload {
                semantic_status = 27;
            }
            word_offset = (auth_neighbor + 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_neighbor + 2) * 24 + 4;
            auth_other = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_left - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            if auth_other != auth_word { semantic_status = 27; }
            word_offset = (auth_neighbor + 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != 17 { semantic_status = 27; }
            if auth_kind == 26 || auth_kind == 27 {
                word_offset = (auth_left - 1) * 16 + 4;
                auth_word = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
                auth_scan = auth_neighbor + 3;
                if auth_word == 2 { auth_scan = auth_neighbor + 8; }
                word_offset = auth_scan * 24;
                auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if auth_word != 25 { semantic_status = 27; }
            }
        }
        if auth_kind == 28 {
            word_offset = (auth_token + 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != 25 { semantic_status = 27; }
        }
        if auth_kind == 34 {
            word_offset = (auth_token - 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_token + 1) * 24 + 0;
            auth_neighbor = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_token + 2) * 24 + 0;
            auth_other = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != 10 || auth_neighbor != 11 || auth_other != 36 {
                semantic_status = 27;
            }
        }
        if auth_kind == 35 {
            word_offset = (auth_token + 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_token + 2) * 24 + 4;
            auth_neighbor = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_left - 1) * 20 + 4;
            auth_other = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            if auth_word != 10 || auth_neighbor != auth_other {
                semantic_status = 27;
            }
            word_offset = (auth_right - 1) * 20 + 4;
            auth_neighbor = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            if auth_neighbor <= auth_other || auth_other <= auth_offset {
                semantic_status = 27;
            }
        }
        if auth_kind == 22 || auth_kind == 23 {
            word_offset = (auth_token + 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if (auth_kind == 22 && auth_word == 5)
                || (auth_kind == 23 && auth_word != 5) {
                semantic_status = 27;
            }
        }

        // Structural wrappers must copy the exact origin of their own role.
        auth_related = 0;
        if auth_kind == 29 || auth_kind == 32 || auth_kind == 37 {
            auth_related = auth_right;
        }
        if auth_kind == 19 {
            word_offset = (auth_left - 1) * 16 + 0;
            auth_word = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
            if auth_word == 38 { auth_related = auth_left; }
        }
        auth_field = 1;
        while semantic_status == 0 && auth_related > 0 && auth_field < 5 {
            word_offset = (auth_index + 1 - 1) * 20 + auth_field * 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            word_offset = (auth_related - 1) * 20 + auth_field * 4;
            auth_neighbor = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            if auth_word != auth_neighbor { semantic_status = 27; }
            auth_field = auth_field + 1;
        }

        // Reverse-linked occurrence sequences preserve leading source order.
        auth_link = 0;
        if auth_kind == 19 || auth_kind == 25 { auth_link = auth_right; }
        if auth_kind == 29 { auth_link = auth_left; }
        if auth_link > 0 {
            word_offset = (auth_link - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            if auth_word >= auth_offset { semantic_status = 27; }
        }
        if auth_kind == 36 {
            word_offset = (auth_left - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            word_offset = (auth_right - 1) * 20 + 4;
            auth_neighbor = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            if auth_word >= auth_neighbor { semantic_status = 27; }
        }

        // Every statement occurrence lies inside its actual opening brace.
        if auth_kind == 30 {
            word_offset = auth_index * 4;
            auth_close = result_value(bytes_get(&ast_ends, word_offset))
                + result_value(bytes_get(&ast_ends, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_ends, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_ends, word_offset + 3)) * 16777216;
            if auth_close <= 0 { semantic_status = 27; }
            auth_close = auth_close - 1;
            word_offset = (auth_close) * 24 + 4;
            auth_other = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            auth_link = auth_left;
            while semantic_status == 0 && auth_link > 0 {
                word_offset = (auth_link - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
                if auth_word <= auth_offset || auth_word >= auth_other {
                    semantic_status = 27;
                }
                word_offset = (auth_link - 1) * 16 + 8;
            auth_link = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
            }
        }


        // Control bodies belong to this exact keyword occurrence. A same-kind
        // brace or another If's branch cannot substitute for its own body.
        if auth_kind == 32 || auth_kind == 33 {
            auth_body = auth_right;
            auth_else = 0;
            if auth_kind == 32 {
                word_offset = (auth_right - 1) * 16 + 8;
            auth_body = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
                word_offset = (auth_right - 1) * 16 + 12;
            auth_else = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
            }
            auth_scan = auth_token + 1;
            auth_open = 0 - 1;
            while auth_scan < token_count && auth_open < 0 {
                word_offset = (auth_scan) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if auth_word == 12 { auth_open = auth_scan; }
                auth_scan = auth_scan + 1;
            }
            if auth_open < 0 { semantic_status = 27; }
            word_offset = (auth_open) * 24 + 4;
            auth_other = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_body - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            if auth_word != auth_other { semantic_status = 27; }
            word_offset = (auth_left - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            if auth_word <= auth_offset || auth_word >= auth_other {
                semantic_status = 27;
            }
            if auth_kind == 32 {
                word_offset = (auth_body - 1) * 4;
                auth_close = result_value(bytes_get(&ast_ends, word_offset))
                + result_value(bytes_get(&ast_ends, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_ends, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_ends, word_offset + 3)) * 16777216;
                if auth_close <= 0 { semantic_status = 27; }
                auth_close = auth_close - 1;
                word_offset = (auth_close + 1) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if auth_else == 0 {
                    if auth_word == 8 { semantic_status = 27; }
                } else {
                    if auth_word != 8 { semantic_status = 27; }
                    word_offset = (auth_close + 2) * 24 + 4;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                    word_offset = (auth_else - 1) * 20 + 4;
            auth_neighbor = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
                    if auth_word != auth_neighbor { semantic_status = 27; }
                }
            }
        }
        // The descriptor's own header bounds its parameter declarations.
        if auth_kind == 38 {
            auth_scan = auth_token + 1;
            auth_open = 0 - 1;
            while auth_scan < token_count && auth_open < 0 {
                word_offset = (auth_scan) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if auth_word == 12 { auth_open = auth_scan; }
                auth_scan = auth_scan + 1;
            }
            if auth_open < 0 { semantic_status = 27; }
            word_offset = (auth_open) * 24 + 4;
            auth_other = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (auth_right - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            if auth_word != auth_other { semantic_status = 27; }
            auth_link = auth_left;
            while semantic_status == 0 && auth_link > 0 {
                word_offset = (auth_link - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
                if auth_word <= auth_offset || auth_word >= auth_other {
                    semantic_status = 27;
                }
                word_offset = (auth_link - 1) * 16 + 12;
            auth_link = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
            }
        }

        // Match owns its subject and first constructor token, not an interner
        // occurrence elsewhere. The second arm follows the first top-level comma.
        if auth_kind == 37 {
            word_offset = (auth_left - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            word_offset = (auth_token + 1) * 24 + 4;
            auth_neighbor = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != auth_neighbor { semantic_status = 27; }
            word_offset = (auth_token + 2) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != 12 { semantic_status = 27; }
            word_offset = (auth_right - 1) * 16 + 8;
            auth_link = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
            word_offset = (auth_link - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            word_offset = (auth_token + 3) * 24 + 4;
            auth_neighbor = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != auth_neighbor { semantic_status = 27; }
            auth_scan = auth_token + 3;
            auth_depth = 0;
            auth_close = 0 - 1;
            while auth_scan < token_count && auth_close < 0 {
                word_offset = (auth_scan) * 24 + 0;
            auth_word = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if auth_word == 10 { auth_depth = auth_depth + 1; }
                if auth_word == 11 { auth_depth = auth_depth - 1; }
                if auth_word == 16 && auth_depth == 0 { auth_close = auth_scan; }
                auth_scan = auth_scan + 1;
            }
            if auth_close < 0 { semantic_status = 27; }
            word_offset = (auth_right - 1) * 16 + 12;
            auth_link = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
            word_offset = (auth_link - 1) * 20 + 4;
            auth_word = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            word_offset = (auth_close + 1) * 24 + 4;
            auth_neighbor = result_value(bytes_get(&tokens, word_offset))
                    + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                    + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                    + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if auth_word != auth_neighbor { semantic_status = 27; }
        }
        auth_index = auth_index + 1;
    }

    // Exact source occurrences, not merely equal token spellings. Every range
    // is computed once from earlier child ranges and authenticated role anchors.
    let mut ast_ranges: ByteBuffer = bytes_new();
    let mut range_index: int = 1;
    let mut range_kind: int = 0;
    let mut range_payload: int = 0;
    let mut range_left: int = 0;
    let mut range_right: int = 0;
    let mut range_at: int = 0;
    let mut range_first: int = 0;
    let mut range_last: int = 0;
    let mut range_left_first: int = 0;
    let mut range_left_last: int = 0;
    let mut range_right_first: int = 0;
    let mut range_right_last: int = 0;
    let mut range_word: int = 0;
    let mut range_other: int = 0;
    let mut range_aux: int = 0;
    let mut range_set: int = 0;
    let mut range_expression: int = 0;
    let mut range_atom: int = 0;
    let mut range_last_atom: int = 0 - 1;
    let mut range_tmp: int = 0;
    let mut range_pos: int = 0;
    let mut range_length: int = 0;
    let mut range_digit_index: int = 0;
    let mut range_digit: int = 0;
    let mut range_literal: int = 0;
    let mut range_expand: int = 0;
    let mut range_field: int = 0;
    let mut range_byte: int = 0;
    let mut range_append: int = 0;
    while status == 0 && semantic_status == 0 && range_index <= node_count {
        word_offset = (range_index - 1) * 16 + 0;
            range_kind = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (range_index - 1) * 16 + 4;
            range_payload = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (range_index - 1) * 16 + 8;
            range_left = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (range_index - 1) * 16 + 12;
            range_right = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        semantic_node = range_index;
        word_offset = (range_index - 1) * 20 + 4;
            semantic_offset = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (range_index - 1) * 20 + 8;
            semantic_line = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (range_index - 1) * 20 + 12;
            semantic_column = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        semantic_code = 1;
        origin_search_low = 0;
        origin_search_high = token_count;
        while origin_search_low < origin_search_high {
            origin_search_middle = origin_search_low
                + signed_quotient(origin_search_high - origin_search_low, 2);
            word_offset = (origin_search_middle) * 24 + 4;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if range_word < semantic_offset { origin_search_low = origin_search_middle + 1; }
            else { origin_search_high = origin_search_middle; }
        }
        range_at = origin_search_low;
        range_left_first = 0 - 1;
        range_left_last = 0 - 1;
        range_right_first = 0 - 1;
        range_right_last = 0 - 1;
        if range_left > 0 {
            word_offset = (range_left - 1) * 8 + 0;
            range_left_first = result_value(bytes_get(&ast_ranges, word_offset))
                + result_value(bytes_get(&ast_ranges, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_ranges, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_ranges, word_offset + 3)) * 16777216;
            range_left_first = range_left_first - 1;
            word_offset = (range_left - 1) * 8 + 4;
            range_left_last = result_value(bytes_get(&ast_ranges, word_offset))
                + result_value(bytes_get(&ast_ranges, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_ranges, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_ranges, word_offset + 3)) * 16777216;
            range_left_last = range_left_last - 1;
        }
        if range_right > 0 {
            word_offset = (range_right - 1) * 8 + 0;
            range_right_first = result_value(bytes_get(&ast_ranges, word_offset))
                + result_value(bytes_get(&ast_ranges, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_ranges, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_ranges, word_offset + 3)) * 16777216;
            range_right_first = range_right_first - 1;
            word_offset = (range_right - 1) * 8 + 4;
            range_right_last = result_value(bytes_get(&ast_ranges, word_offset))
                + result_value(bytes_get(&ast_ranges, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_ranges, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_ranges, word_offset + 3)) * 16777216;
            range_right_last = range_right_last - 1;
        }
        range_first = range_at;
        range_last = range_at;
        range_set = 0;
        range_expression = 0;
        range_atom = 0;
        if (range_kind >= 1 && range_kind <= 17) || range_kind == 20
            || range_kind == 22 || range_kind == 23 || range_kind == 37 {
            range_expression = 1;
        }
        if range_kind == 1 || range_kind == 2
            || (range_kind == 20 && range_left == 0) { range_atom = 1; }
        if range_atom == 1 {
            if range_at <= range_last_atom { semantic_status = 27; }
            range_last_atom = range_at;
        }
        if range_kind == 1 || range_kind == 2 || range_kind == 34 { range_set = 1; }

        // The literal payload is an exact encoding of its own decimal token,
        // not a newly inferred scalar fact.
        if range_kind == 1 {
            word_offset = (range_at) * 24 + 4;
            range_pos = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_at) * 24 + 8;
            range_length = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            range_literal = 0;
            range_digit_index = 0;
            while semantic_status == 0 && range_digit_index < range_length {
                range_digit = result_value(bytes_get(&source, range_pos + range_digit_index)) - 48;
                if range_digit < 0 || range_digit > 9 {
                    semantic_status = 27;
                } else if range_literal > signed_quotient(2147483647 - range_digit, 10) {
                    semantic_status = 27;
                } else { range_literal = range_literal * 10 + range_digit; }
                range_digit_index = range_digit_index + 1;
            }
            if range_literal != range_payload { semantic_status = 27; }
        }

        if range_kind == 3 || range_kind == 4 || range_kind == 22 || range_kind == 23 {
            range_tmp = range_at + 1;
            if range_kind == 23 { range_tmp = range_at + 2; }
            if range_left_first != range_tmp { semantic_status = 27; }
            range_last = range_left_last;
            range_set = 1;
        }
        if range_kind >= 5 && range_kind <= 17 {
            if range_left_last + 1 != range_at || range_right_first != range_at + 1 {
                semantic_status = 27;
            }
            range_first = range_left_first;
            range_last = range_right_last;
            range_set = 1;
        }
        if range_kind == 21 {
            range_first = range_left_first;
            range_last = range_at;
            if range_right == 0 {
                if range_left_last + 1 != range_at { semantic_status = 27; }
            } else {
                word_offset = (range_left_last + 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if range_word != 16 || range_right_first != range_left_last + 2
                    || range_right_last != range_at { semantic_status = 27; }
            }
            range_set = 1;
        }
        if range_kind == 20 {
            range_first = range_at - 1;
            if range_left == 0 {
                range_last = range_at + 1;
                word_offset = (range_last) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if range_word != 11 { semantic_status = 27; }
            } else {
                range_last = range_left_last;
                word_offset = (range_last) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if range_left_first != range_at + 1 || range_word != 11 {
                    semantic_status = 27;
                }
            }
            range_set = 1;
        }
        if range_kind == 24 {
            if range_payload == 2 { range_last = range_at + 5; }
            range_set = 1;
        }
        if range_kind == 25 {
            if range_left_first != range_at + 2 { semantic_status = 27; }
            range_last = range_left_last;
            if range_right > 0 {
                word_offset = (range_right_last + 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if range_word != 16 || range_at != range_right_last + 2 {
                    semantic_status = 27;
                }
                range_first = range_right_first;
            }
            range_set = 1;
        }
        if range_kind == 26 || range_kind == 27 {
            word_offset = (range_left_last + 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_right_last + 1) * 24 + 0;
            range_other = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if range_word != 25 || range_right_first != range_left_last + 2
                || range_other != 18 { semantic_status = 27; }
            range_last = range_right_last + 1;
            range_set = 1;
        }
        if range_kind == 28 || range_kind == 18 {
            range_tmp = range_at + 1;
            if range_kind == 28 { range_tmp = range_at + 2; }
            word_offset = (range_left_last + 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if range_left_first != range_tmp || range_word != 18 { semantic_status = 27; }
            range_last = range_left_last + 1;
            range_set = 1;
        }
        if range_kind == 29 {
            range_first = range_right_first;
            range_last = range_right_last;
            if range_left > 0 {
                if range_left_last + 1 != range_right_first { semantic_status = 27; }
                range_first = range_left_first;
            }
            range_set = 1;
        }
        if range_kind == 30 {
            word_offset = (range_index - 1) * 4;
            range_last = result_value(bytes_get(&ast_ends, word_offset))
                + result_value(bytes_get(&ast_ends, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_ends, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_ends, word_offset + 3)) * 16777216;
            range_last = range_last - 1;
            if range_left_first != range_at + 1 || range_left_last + 1 != range_last {
                semantic_status = 27;
            }
            range_set = 1;
        }
        if range_kind == 31 {
            range_first = range_left_first;
            range_last = range_left_last;
            if range_right > 0 {
                word_offset = (range_left_last + 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                if range_word != 8 || range_right_first != range_left_last + 2 {
                    semantic_status = 27;
                }
                range_last = range_right_last;
            }
            range_set = 1;
        }
        if range_kind == 32 || range_kind == 33 {
            if range_left_first != range_at + 1 || range_left_last + 1 != range_right_first {
                semantic_status = 27;
            }
            range_last = range_right_last;
            range_set = 1;
        }
        if range_kind == 35 {
            if range_left_first != range_at + 2 || range_left_last != range_at + 2
                || range_right_first != range_at + 5 { semantic_status = 27; }
            word_offset = (range_right_last + 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if range_word != 16 { semantic_status = 27; }
            range_last = range_right_last + 1;
            range_set = 1;
        }
        if range_kind == 36 {
            if range_left_last + 1 != range_right_first { semantic_status = 27; }
            range_first = range_left_first;
            range_last = range_right_last;
            range_set = 1;
        }
        if range_kind == 37 {
            word_offset = (range_index - 1) * 4;
            range_last = result_value(bytes_get(&ast_ends, word_offset))
                + result_value(bytes_get(&ast_ends, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_ends, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_ends, word_offset + 3)) * 16777216;
            range_last = range_last - 1;
            if range_left_first != range_at + 1 || range_left_last != range_at + 1
                || range_right_first != range_at + 3 || range_right_last + 1 != range_last {
                semantic_status = 27;
            }
            range_set = 1;
        }
        if range_kind == 38 {
            range_tmp = range_right_first - 1;

            word_offset = (range_at + 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_at + 2) * 24 + 0;
            range_other = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_tmp - 2) * 24 + 0;
            range_aux = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if range_word != 1 || range_other != 10 || range_aux != 11 {
                semantic_status = 27;
            }
            word_offset = (range_tmp - 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_tmp) * 24 + 0;
            range_other = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_tmp) * 24 + 4;
            range_pos = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_tmp) * 24 + 8;
            range_length = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if range_word != 35 || range_other != 1 || range_length != 3
                || result_value(bytes_get(&source, range_pos)) != 105
                || result_value(bytes_get(&source, range_pos + 1)) != 110
                || result_value(bytes_get(&source, range_pos + 2)) != 116 {
                semantic_status = 27;
            }

            if range_left == 0 {
                if range_tmp - 2 != range_at + 3 { semantic_status = 27; }
            } else if range_left_first != range_at + 3
                || range_left_last + 1 != range_tmp - 2 { semantic_status = 27; }
            range_last = range_right_last;
            range_set = 1;
        }
        if range_kind == 19 {
            word_offset = (range_left - 1) * 16 + 0;
            range_word = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
            if range_word == 38 {
                if range_left_first != range_at { semantic_status = 27; }
                range_last = range_left_last;
            } else {
                range_tmp = range_at + 5;

            word_offset = (range_at + 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_at + 2) * 24 + 0;
            range_other = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_tmp - 2) * 24 + 0;
            range_aux = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if range_word != 1 || range_other != 10 || range_aux != 11 {
                semantic_status = 27;
            }
            word_offset = (range_tmp - 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_tmp) * 24 + 0;
            range_other = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_tmp) * 24 + 4;
            range_pos = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_tmp) * 24 + 8;
            range_length = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            if range_word != 35 || range_other != 1 || range_length != 3
                || result_value(bytes_get(&source, range_pos)) != 105
                || result_value(bytes_get(&source, range_pos + 1)) != 110
                || result_value(bytes_get(&source, range_pos + 2)) != 116 {
                semantic_status = 27;
            }

                word_offset = (range_at + 6) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
                word_offset = (range_index - 1) * 4;
            range_last = result_value(bytes_get(&ast_ends, word_offset))
                + result_value(bytes_get(&ast_ends, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_ends, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_ends, word_offset + 3)) * 16777216;
                range_last = range_last - 1;
                if range_word != 12 || range_left_first != range_at + 7
                    || range_left_last + 1 != range_last { semantic_status = 27; }
            }
            if range_right > 0 && range_right_last + 1 != range_at { semantic_status = 27; }
            range_set = 1;
        }

        // Grouping has no AST node. Immediate surrounding LP/RP belongs to
        // this expression unless LP is the actual bare-callee call delimiter.
        range_expand = range_expression;
        while semantic_status == 0 && range_expand == 1 && range_first > 0
            && range_last + 1 < token_count {
            word_offset = (range_first - 1) * 24 + 0;
            range_word = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            word_offset = (range_last + 1) * 24 + 0;
            range_other = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            range_aux = 0;
            if range_first > 1 {
                word_offset = (range_first - 2) * 24 + 0;
            range_aux = result_value(bytes_get(&tokens, word_offset))
                + result_value(bytes_get(&tokens, word_offset + 1)) * 256
                + result_value(bytes_get(&tokens, word_offset + 2)) * 65536
                + result_value(bytes_get(&tokens, word_offset + 3)) * 16777216;
            }
            if range_word == 10 && range_other == 11 && range_aux != 1 {
                range_first = range_first - 1;
                range_last = range_last + 1;
            } else { range_expand = 0; }
        }
        if range_set == 0 || range_first < 0 || range_last < range_first
            || range_last >= token_count - 1 { semantic_status = 27; }
        range_field = 0;
        while semantic_status == 0 && range_field < 2 {
            range_append = range_first + 1;
            if range_field == 1 { range_append = range_last + 1; }
            range_byte = 0;
            while semantic_status == 0 && range_byte < 4 {
                byte = word_byte_0(range_append);
                if range_byte == 1 { byte = word_byte_1(range_append); }
                if range_byte == 2 { byte = word_byte_2(range_append); }
                if range_byte == 3 { byte = word_byte_3(range_append); }
                push_result = result_value(bytes_push(&mut ast_ranges, byte));
                if push_result < 0 {
                    semantic_status = 26;
                    semantic_code = 7;
                    semantic_offset = 0 - 1;
                    semantic_line = 0;
                    semantic_column = 0;
                }
                range_byte = range_byte + 1;
            }
            range_field = range_field + 1;
        }
        range_index = range_index + 1;
    }
    if status == 0 && semantic_status == 0 && bytes_len(&ast_ranges) != node_count * 8 {
        semantic_status = 27;
        semantic_code = 1;
    }
"####,
    ),
    (
        4263,
        0,
        r####""####,
        r####"        && origin_auth_attempted == 1
        && (semantic_status != 26 || semantic_code < 4 || semantic_code > 7)
"####,
    ),
    (
        4326,
        0,
        r####""####,
        r####"    // Connected structure is authenticated syntax, not invented scalar meaning.
    // Keep the identifier-use refusal above first, then refuse the earliest new
    // structural kind with its fully authenticated origin and no facts.
    semantic_index = 0;
    while status == 0 && semantic_status == 0 && semantic_index < node_count {
        word_offset = semantic_index * 16;
        semantic_kind = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        if semantic_kind >= 24 && semantic_kind <= 38 {
            semantic_status = 28;
            semantic_node = semantic_index + 1;
            semantic_code = semantic_kind;
            word_offset = semantic_index * 20 + 4;
            semantic_offset = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            word_offset = semantic_index * 20 + 8;
            semantic_line = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            word_offset = semantic_index * 20 + 12;
            semantic_column = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        }
        semantic_index = semantic_index + 1;
    }

"####,
    ),
    (
        7481,
        0,
        r####""####,
        r####"
"####,
    ),
];

/// Successor representation model. The preserved predecessor expression
/// oracle remains the authority for arithmetic/calls and operational pushes;
/// this recursive structural visitor is independent of the product's flat
/// continuation states. Historical models and vectors are never rewritten.
pub(super) fn successor_parser(source: &[u8], caps: &oracle::Caps) -> oracle::Ingestion {
    struct Model<'a> {
        source: &'a [u8],
        data: oracle::Ingestion,
        index: usize,
        caps: &'a oracle::Caps,
        counts: oracle::Counts,
    }
    impl Model<'_> {
        fn current(&self) -> [i32; 6] {
            self.data.tokens[self.index]
        }
        fn text(&self, token: [i32; 6]) -> &[u8] {
            &self.source[token[1] as usize..(token[1] + token[2]) as usize]
        }
        fn reject(token: [i32; 6], status: i32, code: i32) -> oracle::Reject {
            oracle::Reject {
                offset: token[1],
                line: token[3],
                column: token[4],
                status,
                code,
                actual: token[0],
            }
        }
        fn take(&mut self, kind: i32) -> Result<[i32; 6], oracle::Reject> {
            let token = self.current();
            if token[0] != kind {
                return Err(Self::reject(token, 10, kind));
            }
            self.index += 1;
            Ok(token)
        }
        fn append(
            &mut self,
            kind: i32,
            payload: i32,
            left: i32,
            right: i32,
            token: [i32; 6],
        ) -> Result<i32, oracle::Reject> {
            if self.data.nodes.len() >= self.caps.nodes {
                return Err(Self::reject(token, 14, self.caps.nodes as i32));
            }
            self.data.nodes.push([kind, payload, left, right]);
            let id = self.data.nodes.len() as i32;
            self.data
                .origins
                .push([id, token[1], token[3], token[4], token[0]]);
            Ok(id)
        }
        fn origin_token(&self, id: i32) -> [i32; 6] {
            let origin = self.data.origins[(id - 1) as usize];
            self.data.tokens[self
                .data
                .tokens
                .binary_search_by_key(&origin[1], |t| t[1])
                .unwrap()]
        }
        fn annotation(&mut self, local: bool) -> Result<(i32, [i32; 6]), oracle::Reject> {
            let token = self.take(1)?;
            let code = match self.text(token) {
                b"int" => 1,
                b"ByteBuffer" if local => 3,
                b"Result" => {
                    self.take(29)?;
                    let first = self.take(1)?;
                    if self.text(first) != b"int" {
                        return Err(Self::reject(first, 12, 102));
                    }
                    self.take(16)?;
                    let second = self.take(1)?;
                    if self.text(second) != b"int" {
                        return Err(Self::reject(second, 12, 102));
                    }
                    self.take(31)?;
                    2
                }
                _ => return Err(Self::reject(token, 12, 102)),
            };
            Ok((code, token))
        }
        fn expression(&mut self) -> Result<i32, oracle::Reject> {
            oracle::parse_expression(
                &self.data.clone(),
                self.source,
                &mut self.index,
                &mut self.data.nodes,
                &mut self.data.origins,
                true,
                self.caps,
                &mut self.counts,
            )
        }
        fn returned_expression(&mut self) -> Result<i32, oracle::Reject> {
            let token = self.current();
            if token[0] != 1 || self.text(token) != b"match" {
                return self.expression();
            }
            self.index += 1;
            let subject = self.take(1)?;
            let subject_id = self.append(2, subject[5], 0, 0, subject)?;
            self.take(12)?;
            let mut arms = [0; 2];
            for arm in &mut arms {
                let constructor = self.take(1)?;
                self.take(10)?;
                let binder = self.take(1)?;
                let binder_id = self.append(34, binder[5], 0, 0, binder)?;
                self.take(11)?;
                self.take(36)?;
                let value = self.expression()?;
                self.take(16)?;
                *arm = self.append(35, constructor[5], binder_id, value, constructor)?;
            }
            self.take(13)?;
            let seq = self.append(36, 0, arms[0], arms[1], token)?;
            self.append(37, 0, subject_id, seq, token)
        }
        fn physical_control(&mut self, token: [i32; 6]) -> Result<(), oracle::Reject> {
            if self.counts.blocks >= self.caps.blocks {
                return Err(Self::reject(token, 15, self.caps.blocks as i32));
            }
            self.counts.blocks += 1;
            Ok(())
        }
        fn control(&mut self, keyword: [i32; 6]) -> Result<i32, oracle::Reject> {
            let condition = self.expression()?;
            let open = self.take(12)?;
            self.physical_control(open)?;
            let body = self.block(open, false, false)?;
            if keyword[0] == 9 {
                return self.append(33, 0, condition, body, keyword);
            }
            // Replacing the completed then continuation consumes a record,
            // regardless of whether an else token follows.
            self.physical_control(self.current())?;
            let mut otherwise = 0;
            if self.current()[0] == 8 {
                self.index += 1;
                let next = self.current();
                if next[0] != 7 && next[0] != 12 {
                    return Err(Self::reject(next, 10, 12));
                }
                self.index += 1;
                self.physical_control(next)?;
                otherwise = if next[0] == 7 {
                    self.control(next)?
                } else {
                    self.block(next, false, false)?
                };
            }
            let pair = self.append(31, 0, body, otherwise, keyword)?;
            self.append(32, 0, condition, pair, keyword)
        }
        fn block(
            &mut self,
            open: [i32; 6],
            function_body: bool,
            compact: bool,
        ) -> Result<i32, oracle::Reject> {
            let mut first = 0;
            let mut cells = 0;
            let mut count = 0;
            let mut pending_return = None;
            loop {
                let leading = self.current();
                if leading[0] == 13 {
                    if count == 0 && pending_return.is_none() {
                        return Err(Self::reject(leading, 10, 6));
                    }
                    if function_body && pending_return.is_none() {
                        return Err(Self::reject(leading, 10, 6));
                    }
                    self.index += 1;
                    if let Some((expression, token)) = pending_return {
                        let statement = self.append(18, 0, expression, 0, token)?;
                        if count == 0 {
                            first = statement;
                            count = 1;
                        } else {
                            cells = self.append(29, 0, cells, statement, token)?;
                            count += 1;
                        }
                    }
                    if compact && count == 1 && self.data.nodes[(first - 1) as usize][0] == 18 {
                        return Ok(first);
                    }
                    if cells == 0 {
                        cells = self.append(29, 0, 0, first, self.origin_token(first))?;
                    }
                    return self.append(30, 0, cells, 0, open);
                }
                if pending_return.is_some() {
                    return Err(Self::reject(leading, 10, 13));
                }
                if !matches!(leading[0], 1 | 4 | 6 | 7 | 9) {
                    return Err(Self::reject(
                        leading,
                        10,
                        if function_body || count == 0 { 6 } else { 13 },
                    ));
                }
                // Promotion precedes parsing the next statement, including
                // malformed initializers. It does not happen at an invalid opener.
                if count == 1 && cells == 0 {
                    cells = self.append(29, 0, 0, first, self.origin_token(first))?;
                }
                self.index += 1;
                let statement = match leading[0] {
                    6 => {
                        let expression = self.returned_expression()?;
                        self.take(18)?;
                        pending_return = Some((expression, leading));
                        continue;
                    }
                    7 | 9 => self.control(leading)?,
                    1 => {
                        self.take(25)?;
                        let expression = self.expression()?;
                        self.take(18)?;
                        self.append(28, leading[5], expression, 0, leading)?
                    }
                    4 => {
                        let mutable = self.current()[0] == 5;
                        if mutable {
                            self.index += 1;
                        }
                        let name = self.take(1)?;
                        self.take(17)?;
                        let (code, token) = self.annotation(true)?;
                        let annotation = self.append(24, code, 0, 0, token)?;
                        self.take(25)?;
                        let expression = self.expression()?;
                        self.take(18)?;
                        self.append(
                            if mutable { 27 } else { 26 },
                            name[5],
                            annotation,
                            expression,
                            leading,
                        )?
                    }
                    _ => unreachable!(),
                };
                if count == 0 {
                    first = statement;
                } else {
                    cells = self.append(29, 0, cells, statement, leading)?;
                }
                count += 1;
            }
        }
        fn module(&mut self) -> Result<(), oracle::Reject> {
            let mut previous = 0;
            loop {
                let keyword = self.take(3)?;
                let name = self.take(1)?;
                self.take(10)?;
                let mut parameters = 0;
                if self.current()[0] != 11 {
                    loop {
                        let declaration = self.take(1)?;
                        self.take(17)?;
                        let (code, token) = self.annotation(false)?;
                        self.data.parameters.push((declaration[5], code));
                        let annotation = self.append(24, code, 0, 0, token)?;
                        parameters =
                            self.append(25, declaration[5], annotation, parameters, declaration)?;
                        if self.current()[0] != 16 {
                            break;
                        }
                        self.index += 1;
                    }
                }
                self.take(11)?;
                self.take(35)?;
                let result = self.take(1)?;
                if self.text(result) != b"int" {
                    return Err(Self::reject(result, 12, 102));
                }
                let open = self.take(12)?;
                let mut body = self.block(open, true, parameters == 0)?;
                if self.data.nodes[(body - 1) as usize][0] == 30 {
                    body = self.append(38, 1, parameters, body, keyword)?;
                }
                previous = self.append(19, name[5], body, previous, keyword)?;
                if self.current()[0] == 3 {
                    continue;
                }
                self.take(0)?;
                self.data.root = previous;
                return Ok(());
            }
        }
    }
    let mut data = module_ingest(source);
    if data.status != 0 {
        return data;
    }
    data.signature_grammar = true;
    let mut model = Model {
        source,
        data,
        index: 0,
        caps,
        counts: oracle::Counts::default(),
    };
    if let Err(stop) = model.module() {
        model.data.status = stop.status;
        model.data.error_offset = stop.offset;
        model.data.error_line = stop.line;
        model.data.error_column = stop.column;
        model.data.diagnostic_code = stop.code;
        model.data.diagnostic_actual = stop.actual;
    }
    model.data.counts = (
        model.counts.values,
        model.counts.operators,
        model.counts.blocks,
        model.counts.calls,
    );
    model.data
}

pub(super) fn successor_refusal(stopped: &oracle::Ingestion) -> oracle::SemanticStop {
    assert_eq!(stopped.status, 0);
    let mut stop = oracle::SemanticStop {
        status: 0,
        node: 0,
        offset: -1,
        line: 0,
        column: 0,
        code: 0,
        expected: 0,
        actual: 0,
        symbols: 0,
        facts: 0,
        root_type: 0,
        symbol_words: Vec::new(),
        fact_words: Vec::new(),
    };
    for (index, node) in stopped.nodes.iter().enumerate() {
        if node[0] == 19 {
            stop.symbol_words
                .extend_from_slice(&[1, node[1], index as i32 + 1, 1]);
            stop.symbols += 1;
        }
    }
    let refusal = stopped
        .nodes
        .iter()
        .position(|node| node[0] == 2)
        .map(|index| (index, 17, 2))
        .or_else(|| {
            stopped
                .nodes
                .iter()
                .position(|node| (24..=38).contains(&node[0]))
                .map(|index| (index, 28, stopped.nodes[index][0]))
        });
    let (index, status, code) =
        refusal.expect("only rich or identifier-refused modules use this model");
    let origin = stopped.origins[index];
    stop.status = status;
    stop.node = index as i32 + 1;
    stop.offset = origin[1];
    stop.line = origin[2];
    stop.column = origin[3];
    stop.code = code;
    stop
}

pub(super) fn run_cached_vector(source: &[u8], expected: &[i32], optimization: &str) -> i32 {
    let workspace = TestWorkspace::new("connected-ast-full-vector");
    let product = native_product(optimization);
    let object = workspace.write("product.o", &product.object);
    let harness = workspace.write(
        "expectation.c",
        expectation_harness(expected, source, source.len() as i32),
    );
    let runtime = repository_path("../../src/compiler/runtime/aero_test_runtime.c");
    let executable = clang_link(
        &workspace,
        "full-vector",
        optimization,
        &[object.as_path(), runtime.as_path(), harness.as_path()],
    );
    let output = Command::new(executable)
        .output()
        .expect("run full authentication/refusal vector");
    assert!(
        output.stdout.is_empty() && output.stderr.is_empty(),
        "forbidden output: {output:?}"
    );
    output
        .status
        .code()
        .expect("native full-vector process exit")
}

/// Explicitly amended current-product grading; callers retain their complete
/// historical models/tables and opt into this successor instead of silently
/// switching a shared harness. Grammar diagnostics and expression work remain
/// exact; representation changes and the separately frozen scalar semantic
/// successor are graded explicitly. The predecessor refusal remains unchanged.
pub(super) fn run_successor_expectation(
    label: &str,
    source: &[u8],
    predecessor: &oracle::Ingestion,
    optimization: &str,
) -> i32 {
    let successor = successor_parser(source, &module_caps());
    assert_eq!(
        (
            successor.status,
            successor.error_offset,
            successor.error_line,
            successor.error_column,
            successor.diagnostic_code,
            successor.diagnostic_actual
        ),
        (
            predecessor.status,
            predecessor.error_offset,
            predecessor.error_line,
            predecessor.error_column,
            predecessor.diagnostic_code,
            predecessor.diagnostic_actual
        ),
        "{label}: connected representation must preserve grammar priority/location"
    );
    assert_eq!(
        successor.parameters, predecessor.parameters,
        "{label}: parameter ABI"
    );
    assert_eq!(
        (successor.counts.0, successor.counts.1, successor.counts.3),
        (
            predecessor.counts.0,
            predecessor.counts.1,
            predecessor.counts.3
        ),
        "{label}: independent expression work-store costs"
    );
    let expected = if successor.status == 0 {
        validate_origins(source, &successor.nodes, &successor.origins);
        if successor
            .nodes
            .iter()
            .all(|node| node[0] != 2 && node[0] < 24)
        {
            // Compact arithmetic/borrowed-call behavior remains graded by the
            // exact existing meaning/checked/verifier/emitter authority.
            let meaning = oracle::module_semantic_meaning(&successor);
            return run_meaning_expectation(label, source, &successor, &meaning, optimization);
        }
        oracle::refused_expectation_vector(
            source,
            &successor,
            &scoped_semantics::scalar_meaning(&successor),
        )
    } else {
        oracle::expectation_vector(source, &successor)
    };
    run_cached_vector(source, &expected, optimization)
}

pub(super) fn run_fault_vector(
    source: &[u8],
    expected: &[i32],
    optimization: &str,
    trigger_owner: u32,
    mutation: &str,
    fail_after: Option<(u64, bool)>,
) -> Capture {
    let workspace = TestWorkspace::new("connected-ast-authentication-fault");
    let product = native_product(optimization);
    let object = workspace.write("product.o", &product.object);
    let runtime = repository_path("../../src/compiler/runtime/aero_test_runtime.c");
    let include = runtime.to_string_lossy().replace('\\', "/");
    let observer=OBSERVER.replace("RUNTIME_PATH",&include).replace(
        "    void *pointer = ast_runtime_alloc(size);",
        &format!("    if (ast_owner_count + 1 == {trigger_owner}) {{ {mutation} }}\n    void *pointer = ast_runtime_alloc(size);"));
    let mut harness = expectation_harness(expected, source, source.len() as i32);
    if let Some((events, failed_initial)) = fail_after {
        harness = harness.replace(
            "aero_test_reset(UINT64_MAX)",
            &format!("aero_test_reset(UINT64_C({events}))"),
        );
        // A failed initial allocation is an attempted call with no owner to
        // release. Failed reallocations retain the original owner and bytes.
        if failed_initial {
            harness = harness.replace(
                "aero_test_alloc_calls() != aero_test_dealloc_calls()",
                "aero_test_alloc_calls() != aero_test_dealloc_calls() + 1",
            );
        }
    }
    let harness=harness.replace("int main(void) {",
        "int main(int argc, char **argv) {\n    if (argc != 2) return 58;")
        .replace("    if (wrote_output != 0) return 62;",
            "    if (!ast_write_capture(argv[1], result)) return 59;\n    if (wrote_output != 0) return 62;");
    let harness = workspace.write("fault.c", format!("{observer}\n{harness}"));
    let executable = clang_link(
        &workspace,
        "authentication-fault",
        optimization,
        &[object.as_path(), harness.as_path()],
    );
    let artifact = workspace.root.join("actual-authentication-fault.bin");
    let output = Command::new(executable)
        .arg(&artifact)
        .output()
        .expect("run exact native fault vector");
    assert_eq!(
        output.status.code(),
        Some(91),
        "full fault vector failed: {output:?}"
    );
    assert!(
        output.stdout.is_empty() && output.stderr.is_empty(),
        "forbidden channel output: {output:?}"
    );
    let capture = Capture::read(&fs::read(artifact).expect("completed native artifact"));
    assert_eq!(capture.counters[3..], [0, 0]);
    assert_eq!(&capture.owner(1)[..source.len()], source);
    capture
}

pub(super) fn growth_events_before_starts(source: &[u8]) -> u64 {
    let model = successor_parser(source, &module_caps());
    assert_eq!(model.status, 0);
    let used = [
        source.len(),
        model.names.len() * 8,
        model.tokens.len() * 24,
        model.nodes.len() * 16,
        model.parameters.len() * 8,
        model.counts.2 * 80,
        model.counts.3 * 12,
        model.counts.0 * 8,
        model.counts.1 * 20,
        model.origins.len() * 20,
    ];
    used.into_iter()
        .map(|bytes| {
            if bytes == 0 {
                return 0;
            }
            let mut capacity = 8;
            let mut events = 1;
            while capacity < bytes {
                capacity *= 2;
                events += 1;
            }
            events
        })
        .sum()
}

pub(super) fn growth_events(bytes: usize) -> u64 {
    if bytes == 0 {
        return 0;
    }
    let mut capacity = 8;
    let mut events = 1;
    while capacity < bytes {
        capacity *= 2;
        events += 1;
    }
    events
}

#[test]
fn brace_history_and_end_map_first_allocations_and_growth_fail_closed() {
    // End records start at node1; their first growth is node3. For the
    // parameter witness the first closing occurrence is Block8, after seven
    // end words; history's sole opening uses one eight-byte record. Match13
    // sees two openings (function and Match), so its second history record is
    // the first history growth. All event thresholds are derived beforehand.
    let parameter = successor_parser(PARAMETERS, &module_caps());
    let matched = successor_parser(ARBITRARY_MATCH, &module_caps());
    let parameter_start_events =
        growth_events_before_starts(PARAMETERS) + growth_events(parameter.nodes.len() * 4);
    let match_start_events =
        growth_events_before_starts(ARBITRARY_MATCH) + growth_events(matched.nodes.len() * 4);
    let cases = [
        (PARAMETERS, 6, 1, parameter_start_events, true, 8, 0),
        (PARAMETERS, 6, 3, parameter_start_events + 1, false, 9, 9),
        (
            PARAMETERS,
            5,
            8,
            parameter_start_events + growth_events(7 * 4),
            true,
            9,
            0,
        ),
        (
            ARBITRARY_MATCH,
            5,
            13,
            match_start_events + growth_events(12 * 4) + 1,
            false,
            11,
            11,
        ),
    ];
    for (source, code, node, events, initial, owners, failed_owner) in cases {
        let model = successor_parser(source, &module_caps());
        let meaning = unauthenticated_stop(26, node, -1, 0, 0, code);
        let expected = oracle::refused_expectation_vector(source, &model, &meaning);
        for optimization in ["-O0", "-O2"] {
            let capture = run_fault_vector(
                source,
                &expected,
                optimization,
                0,
                "",
                Some((events, initial)),
            );
            assert_eq!(capture.owners.len(), owners);
            assert_eq!(
                capture
                    .events
                    .iter()
                    .filter(|event| !event.succeeded)
                    .count(),
                1
            );
            let failure = capture
                .events
                .iter()
                .find(|event| !event.succeeded)
                .unwrap();
            assert_eq!(
                (failure.kind, failure.owner),
                (if initial { 1 } else { 2 }, failed_owner)
            );
            let (nodes, origins) = if source == PARAMETERS { (6, 5) } else { (6, 5) };
            assert_eq!(capture.records::<4>(nodes), model.nodes);
            assert_eq!(capture.records::<5>(origins), model.origins);
            if code == 6 && node == 3 {
                assert_eq!(capture.records::<1>(9), vec![[0], [0]]);
            }
            if code == 5 && !initial {
                let opening = model
                    .tokens
                    .iter()
                    .position(|token| token[0] == 12)
                    .unwrap() as i32;
                assert_eq!(
                    capture.records::<2>(11),
                    vec![[0, opening]],
                    "failed growth retains history"
                );
            }
        }
    }
}

#[test]
fn brace_mapping_rejects_missing_extra_and_unmatched_events() {
    let source = &b"fn f()->int{let x:int=1;return 2;}"[..];
    let model = successor_parser(source, &module_caps());
    assert_eq!(model.nodes.len(), 10);
    let equals = model
        .tokens
        .iter()
        .position(|token| token[0] == 25)
        .unwrap();
    let cases = [
        (equals, 13, 9, true),
        (model.tokens.len() - 2, 0, 7, true),
        (2, 13, 7, false),
        (equals, 12, 7, true),
    ];
    for (token, kind, rejecting, history_allocated) in cases {
        let origin = model.origins[rejecting];
        let meaning =
            unauthenticated_stop(27, rejecting as i32 + 1, origin[1], origin[2], origin[3], 1);
        let expected = oracle::refused_expectation_vector(source, &model, &meaning);
        let mutation = store_mutation(2, token * 24, kind);
        for optimization in ["-O0", "-O2"] {
            let capture = run_fault_vector(source, &expected, optimization, 7, &mutation, None);
            let cleanup = if history_allocated {
                &[8, 9, 7, 4, 6, 5, 2, 3, 1][..]
            } else {
                &[8, 7, 4, 6, 5, 2, 3, 1][..]
            };
            capture.validate_trace_at(cleanup, 91);
        }
    }
}

/// Closed form for a resource-boundary witness, not a recursive Rust-parser
/// replay. n conditions, inner literal/Return/Cell/Block, n While records with
/// n-1 enclosing Cell/Blocks, then the seven top-body/item records:4*n+9.
pub(super) fn deeply_nested_while_model(depth: usize) -> (Vec<u8>, oracle::Ingestion) {
    let mut source = b"fn f()->int{".to_vec();
    for _ in 0..depth {
        source.extend_from_slice(b"while 1{");
    }
    source.extend_from_slice(b"return 1;");
    source.extend(std::iter::repeat_n(b'}', depth));
    source.extend_from_slice(b"return 1;}");
    let mut data = module_ingest(&source);
    data.signature_grammar = true;
    let append =
        |data: &mut oracle::Ingestion, kind, payload, left, right, offset: usize, token_kind| {
            data.nodes.push([kind, payload, left, right]);
            let id = data.nodes.len() as i32;
            data.origins
                .push([id, offset as i32, 1, offset as i32 + 1, token_kind]);
            id
        };
    for level in 0..depth {
        append(&mut data, 1, 1, 0, 0, 18 + 8 * level, 2);
    }
    let inner = 12 + 8 * depth;
    let literal = append(&mut data, 1, 1, 0, 0, inner + 7, 2);
    let returned = append(&mut data, 18, 0, literal, 0, inner, 6);
    let cell = append(&mut data, 29, 0, 0, returned, inner, 6);
    let mut body = append(&mut data, 30, 0, cell, 0, 19 + 8 * (depth - 1), 12);
    let mut outer = 0;
    for level in (0..depth).rev() {
        outer = append(&mut data, 33, 0, level as i32 + 1, body, 12 + 8 * level, 9);
        if level > 0 {
            let cell = append(&mut data, 29, 0, 0, outer, 12 + 8 * level, 9);
            body = append(&mut data, 30, 0, cell, 0, 19 + 8 * (level - 1), 12);
        }
    }
    let first = append(&mut data, 29, 0, 0, outer, 12, 9);
    let final_return = 21 + 9 * depth;
    let literal = append(&mut data, 1, 1, 0, 0, final_return + 7, 2);
    let returned = append(&mut data, 18, 0, literal, 0, final_return, 6);
    let cell = append(&mut data, 29, 0, first, returned, final_return, 6);
    let block = append(&mut data, 30, 0, cell, 0, 11, 12);
    let descriptor = append(&mut data, 38, 1, 0, block, 0, 3);
    data.root = append(&mut data, 19, 1, descriptor, 0, 0, 3);
    data.counts = (depth + 2, 0, depth, 0);
    (source, data)
}

#[test]
fn maximum_formula_nested_while_authenticates_without_a_lower_depth_cutoff() {
    const DEPTH: usize = 16381;
    let (source, model) = deeply_nested_while_model(DEPTH);
    assert_eq!(
        (source.len(), model.tokens.len(), model.nodes.len()),
        (147460, 65539, 65533)
    );
    assert_eq!(
        model.tokens.iter().filter(|token| token[0] == 12).count(),
        DEPTH + 1
    );
    validate_shapes(&model.nodes, model.names.len());
    let first = DEPTH + 2; // first new kind is the inner Return's Cell29
    assert_eq!(model.nodes[first][0], 29);
    let origin = model.origins[first];
    let mut meaning =
        unauthenticated_stop(28, first as i32 + 1, origin[1], origin[2], origin[3], 29);
    meaning.symbols = 1;
    meaning.symbol_words = vec![1, 1, model.root, 1];
    let predecessor = oracle::refused_expectation_vector(&source, &model, &meaning);
    assert_eq!(
        predecessor[11], 28,
        "historical representation refusal is retained"
    );
    let successor = scoped_semantics::scalar_meaning(&model);
    assert_eq!(
        (
            successor.status,
            successor.code,
            successor.node,
            successor.facts
        ),
        (31, 3, DEPTH as i32 + 5, DEPTH as i32 + 4)
    );
    let expected = oracle::refused_expectation_vector(&source, &model, &successor);
    for optimization in ["-O0", "-O2"] {
        assert_eq!(run_cached_vector(&source, &expected, optimization), 91);
    }
}

fn store_mutation(owner: u32, byte_offset: usize, word: i32) -> String {
    let bytes = word.to_le_bytes();
    format!(
        "if ({owner} > ast_owner_count || !ast_owners[{owner}-1].live || ast_owners[{owner}-1].size < {}) ast_observer_error=1; else {{ unsigned char *p=(unsigned char *)ast_owners[{owner}-1].pointer; p[{byte_offset}]={}; p[{}]={}; p[{}]={}; p[{}]={}; }}",
        byte_offset + 4,
        bytes[0],
        byte_offset + 1,
        bytes[1],
        byte_offset + 2,
        bytes[2],
        byte_offset + 3,
        bytes[3]
    )
}

fn unauthenticated_stop(
    status: i32,
    node: i32,
    offset: i32,
    line: i32,
    column: i32,
    code: i32,
) -> oracle::SemanticStop {
    oracle::SemanticStop {
        status,
        node,
        offset,
        line,
        column,
        code,
        expected: 0,
        actual: 0,
        symbols: 0,
        facts: 0,
        root_type: 0,
        symbol_words: vec![],
        fact_words: vec![],
    }
}

#[test]
fn start_sidecar_first_allocation_and_first_growth_fail_with_unlocated_code4() {
    let model = successor_parser(PARAMETERS, &module_caps());
    let before = growth_events_before_starts(PARAMETERS);
    // Eight initial bytes hold nodes1/2; node3 requests the first growth.
    for (node, events) in [(1, before), (3, before + 1)] {
        let meaning = unauthenticated_stop(26, node, -1, 0, 0, 4);
        let expected = oracle::refused_expectation_vector(PARAMETERS, &model, &meaning);
        for optimization in ["-O0", "-O2"] {
            let capture = run_fault_vector(
                PARAMETERS,
                &expected,
                optimization,
                8,
                "",
                Some((events, node == 1)),
            );
            assert_eq!(capture.result, 91);
            assert_eq!(capture.owners.len(), if node == 1 { 7 } else { 8 });
            assert_eq!(
                capture
                    .events
                    .iter()
                    .filter(|event| !event.succeeded)
                    .count(),
                1
            );
            let failure = capture
                .events
                .iter()
                .find(|event| !event.succeeded)
                .unwrap();
            assert_eq!(failure.kind, if node == 1 { 1 } else { 2 });
            assert_eq!(failure.owner, if node == 1 { 0 } else { 8 });
            assert_eq!(capture.records::<4>(6), model.nodes);
            assert_eq!(capture.records::<5>(5), model.origins);
            if node == 3 {
                assert_eq!(capture.records::<1>(8), vec![[1], [1]]);
            }
        }
    }
}

#[test]
fn native_origin_lookup_rejects_wrong_coordinates_non_token_and_eof_anchors() {
    let model = successor_parser(PARAMETERS, &module_caps());
    for (index, field, value) in [
        (0, 3, model.origins[0][3] + 1),
        (0, 1, model.origins[0][1] + 1),
        (9, 3, 2),
        (9, 1, PARAMETERS.len() as i32),
    ] {
        let mut corrupt = model.clone();
        corrupt.origins[index][field] = value;
        let origin = corrupt.origins[index];
        let meaning =
            unauthenticated_stop(27, index as i32 + 1, origin[1], origin[2], origin[3], 1);
        let mut expected = oracle::refused_expectation_vector(PARAMETERS, &corrupt, &meaning);
        expected[10] = oracle::parse_checksum(PARAMETERS, &model); // fault after parse grading
        let mutation = store_mutation(5, index * 20 + field * 4, value);
        for optimization in ["-O0", "-O2"] {
            let capture = run_fault_vector(PARAMETERS, &expected, optimization, 8, &mutation, None);
            capture.validate_trace_at(&[8, 5, 7, 4, 6, 2, 3, 1], 91);
            assert_eq!(capture.records::<5>(5), corrupt.origins);
        }
    }
}

#[test]
fn native_role_authentication_rejects_same_kind_and_same_spelling_wrong_owners() {
    let same_headers = &b"fn f(a:int)->int{return 1;} fn g(a:int)->int{return 2;}"[..];
    let same_binders =
        &b"fn f(r:Result<int,int>)->int{return match r{Alpha(x)=>x,Beta(x)=>0-x,};}"[..];
    // (source, origin-owner, starts-owner, cleanup, mutations, rejecting node)
    let cases: Vec<(&[u8], u32, u32, &[u32], Vec<(usize, usize)>, usize)> = vec![
        (
            NESTED_IF,
            4,
            8,
            &[9, 10, 8, 4, 6, 7, 5, 2, 3, 1],
            vec![(4, 7)],
            4,
        ),
        (
            NESTED_IF,
            4,
            8,
            &[9, 10, 8, 4, 6, 7, 5, 2, 3, 1],
            vec![(10, 19)],
            11,
        ),
        (
            NESTED_IF,
            4,
            8,
            &[9, 10, 8, 4, 6, 7, 5, 2, 3, 1],
            vec![(5, 9)],
            5,
        ),
        (
            same_headers,
            5,
            8,
            &[9, 10, 8, 5, 7, 4, 6, 2, 3, 1],
            vec![(8, 0), (9, 1)],
            14,
        ),
        (
            same_binders,
            5,
            9,
            &[10, 11, 9, 5, 8, 7, 4, 6, 2, 3, 1],
            vec![(3, 6)],
            5,
        ),
    ];
    for (source, owner, starts, cleanup, swaps, rejecting) in cases {
        let model = successor_parser(source, &module_caps());
        assert_eq!(model.status, 0);
        let mut corrupt = model.clone();
        let mut mutation = String::new();
        for (target, replacement) in swaps {
            corrupt.origins[target][1..].copy_from_slice(&model.origins[replacement][1..]);
            for field in 1..5 {
                mutation.push_str(&store_mutation(
                    owner,
                    target * 20 + field * 4,
                    corrupt.origins[target][field],
                ));
            }
        }
        assert!(
            std::panic::catch_unwind(|| validate_origins(source, &corrupt.nodes, &corrupt.origins))
                .is_err()
        );
        let origin = corrupt.origins[rejecting];
        let meaning =
            unauthenticated_stop(27, rejecting as i32 + 1, origin[1], origin[2], origin[3], 1);
        let mut expected = oracle::refused_expectation_vector(source, &corrupt, &meaning);
        expected[10] = oracle::parse_checksum(source, &model);
        for optimization in ["-O0", "-O2"] {
            let capture =
                run_fault_vector(source, &expected, optimization, starts, &mutation, None);
            capture.validate_trace_at(cleanup, 91);
            assert_eq!(capture.records::<5>(owner), corrupt.origins);
        }
    }
}

#[test]
fn native_connectivity_rejects_a_backward_but_gapped_module_before_origins() {
    let model = successor_parser(TWO_FUNCTIONS, &module_caps());
    assert_eq!(model.nodes.len(), 18);
    let mut corrupt = model.clone();
    corrupt.nodes[17][2] = 8; // a legal Descriptor kind but the other subtree
    let meaning = unauthenticated_stop(27, 18, -1, 0, 0, 1);
    let mut expected = oracle::refused_expectation_vector(TWO_FUNCTIONS, &corrupt, &meaning);
    expected[10] = oracle::parse_checksum(TWO_FUNCTIONS, &model);
    let mutation = store_mutation(6, 17 * 16 + 8, 8);
    for optimization in ["-O0", "-O2"] {
        let capture = run_fault_vector(TWO_FUNCTIONS, &expected, optimization, 8, &mutation, None);
        capture.validate_trace_at(&[8, 5, 7, 4, 6, 2, 3, 1], 91);
        assert_eq!(capture.records::<4>(6), corrupt.nodes);
        assert_eq!(capture.records::<5>(5), model.origins);
    }
}

#[test]
fn native_annotation_and_let_roles_require_identifier_arguments_and_assignment_token() {
    let binding = &b"fn f()->int{let x:Result<int,int> = 1;return 2;}"[..];
    let mut outcomes = Vec::new();
    for (source, starts, rejecting) in [(PARAMETERS, 8, 2), (binding, 7, 2)] {
        let model = successor_parser(source, &module_caps());
        assert_eq!(model.status, 0);
        let result_token = model
            .tokens
            .iter()
            .position(|token| {
                &source[token[1] as usize..(token[1] + token[2]) as usize] == b"Result"
            })
            .unwrap();
        let token = if source == PARAMETERS {
            result_token + 2
        } else {
            result_token + 6
        };
        assert_eq!(
            model.tokens[token][0],
            if source == PARAMETERS { 1 } else { 25 }
        );
        let mutation = store_mutation(2, token * 24, if source == PARAMETERS { 2 } else { 26 });
        let origin = model.origins[rejecting];
        let meaning =
            unauthenticated_stop(27, rejecting as i32 + 1, origin[1], origin[2], origin[3], 1);
        let expected = oracle::refused_expectation_vector(source, &model, &meaning);
        for optimization in ["-O0", "-O2"] {
            outcomes.push(
                std::panic::catch_unwind(|| {
                    run_fault_vector(source, &expected, optimization, starts, &mutation, None)
                })
                .is_ok(),
            );
        }
    }
    assert!(
        outcomes.iter().all(|passed| *passed),
        "every completed O0/O2 exact role-authentication vector must match: {outcomes:?}"
    );
}

#[test]
fn native_expression_occurrences_reject_same_value_moves_and_swaps() {
    let bindings = &b"fn f()->int{let x:int=1;let y:int=1;return 1;}"[..];
    let arithmetic = &b"fn f()->int{return 1+1;}"[..];
    let mut outcomes = Vec::new();
    for (source, swaps, rejecting) in [
        (bindings, vec![(1, 5)], 2),
        (bindings, vec![(1, 5), (5, 1)], 2),
        (arithmetic, vec![(0, 1), (1, 0)], 1),
    ] {
        let model = successor_parser(source, &module_caps());
        assert_eq!(model.status, 0);
        let mut corrupt = model.clone();
        let mut mutation = String::new();
        for (target, replacement) in swaps {
            assert_eq!(
                model.nodes[target], model.nodes[replacement],
                "same-valued atom records"
            );
            corrupt.origins[target][1..].copy_from_slice(&model.origins[replacement][1..]);
            for field in 1..5 {
                mutation.push_str(&store_mutation(
                    4,
                    target * 20 + field * 4,
                    corrupt.origins[target][field],
                ));
            }
        }
        let origin = corrupt.origins[rejecting];
        let meaning =
            unauthenticated_stop(27, rejecting as i32 + 1, origin[1], origin[2], origin[3], 1);
        let mut expected = oracle::refused_expectation_vector(source, &corrupt, &meaning);
        expected[10] = oracle::parse_checksum(source, &model);
        // Bindings allocate no operator records: starts7. Arithmetic adds
        // operators7, so starts8. The fault occurs after parse grading.
        let starts = if source == bindings { 7 } else { 8 };
        for optimization in ["-O0", "-O2"] {
            outcomes.push(
                std::panic::catch_unwind(|| {
                    run_fault_vector(source, &expected, optimization, starts, &mutation, None)
                })
                .is_ok(),
            );
        }
    }
    assert!(
        outcomes.iter().all(|passed| *passed),
        "every completed same-value native occurrence vector must match: {outcomes:?}"
    );
}

#[test]
fn native_literal_payload_must_encode_its_actual_source_digits() {
    let source = &b"fn f()->int{return 1+1;}"[..];
    let model = successor_parser(source, &module_caps());
    let mut corrupt = model.clone();
    corrupt.nodes[0][1] = 2;
    let origin = model.origins[0];
    let meaning = unauthenticated_stop(27, 1, origin[1], origin[2], origin[3], 1);
    let mut expected = oracle::refused_expectation_vector(source, &corrupt, &meaning);
    expected[10] = oracle::parse_checksum(source, &model);
    let mutation = store_mutation(5, 4, 2);
    for optimization in ["-O0", "-O2"] {
        let capture = run_fault_vector(source, &expected, optimization, 8, &mutation, None);
        capture.validate_trace_at(&[9, 10, 8, 4, 7, 6, 5, 2, 3, 1], 91);
        assert_eq!(capture.records::<4>(5), corrupt.nodes);
    }
}

#[test]
fn range_table_first_allocation_and_first_growth_are_unlocated_code7() {
    let model = successor_parser(PARAMETERS, &module_caps());
    let before = growth_events_before_starts(PARAMETERS)
        + growth_events(model.nodes.len() * 4) * 2
        + growth_events(8);
    for (node, events, initial) in [(1, before, true), (2, before + 1, false)] {
        let meaning = unauthenticated_stop(26, node, -1, 0, 0, 7);
        let expected = oracle::refused_expectation_vector(PARAMETERS, &model, &meaning);
        for optimization in ["-O0", "-O2"] {
            let capture = run_fault_vector(
                PARAMETERS,
                &expected,
                optimization,
                0,
                "",
                Some((events, initial)),
            );
            assert_eq!(capture.owners.len(), if initial { 10 } else { 11 });
            let failed: Vec<_> = capture
                .events
                .iter()
                .filter(|event| !event.succeeded)
                .collect();
            assert_eq!(failed.len(), 1);
            assert_eq!(
                (failed[0].kind, failed[0].owner),
                if initial { (1, 0) } else { (2, 11) }
            );
            assert_eq!(capture.records::<4>(6), model.nodes);
            assert_eq!(capture.records::<5>(5), model.origins);
            if !initial {
                assert_eq!(capture.records::<2>(11), vec![[6, 6]]);
            }
        }
    }
}

#[test]
fn grouped_expressions_and_calls_keep_their_exact_owned_slots() {
    for source in [
        &b"fn f(a:int)->int{return (1);}"[..],
        &b"fn f(a:int)->int{return (-1);}"[..],
        &b"fn f(a:int)->int{return (g(1));}"[..],
        &b"fn f(a:int)->int{return g((1),h(),(2+3));}"[..],
        &b"fn f(a:int)->int{return g(&x,&mut y,((1)));}"[..],
        &b"fn f(a:int)->int{let x:int=(1);x=(-1);if (1){x=g((1));}return (2);}"[..],
    ] {
        let model = successor_parser(source, &module_caps());
        assert_eq!(model.status, 0);
        let historical = successor_refusal(&model);
        assert_eq!(historical.facts, 0, "preserved representation-only model");
        let expected = oracle::refused_expectation_vector(
            source,
            &model,
            &scoped_semantics::scalar_meaning(&model),
        );
        for optimization in ["-O0", "-O2"] {
            assert_eq!(run_cached_vector(source, &expected, optimization), 91);
        }
    }
}

#[test]
fn grouping_witnesses_belong_to_the_unchanged_admitted_grammar() {
    for source in [
        &b"fn f(a:int)->int{return (1);}"[..],
        &b"fn f(a:int)->int{return (-1);}"[..],
        &b"fn f(a:int)->int{return (g(1));}"[..],
        &b"fn f(a:int)->int{return g((1),h(),(2+3));}"[..],
        &b"fn f(a:int)->int{return g(&x,&mut y,((1)));}"[..],
        &b"fn f(a:int)->int{let x:int=(1);x=(-1);if (1){x=g((1));}return (2);}"[..],
    ] {
        let historical =
            oracle::binding_parser_stop(&module_ingest(source), source, &module_caps());
        let current = successor_parser(source, &module_caps());
        assert_eq!(historical.status, 0, "{}", String::from_utf8_lossy(source));
        assert_eq!(current.status, 0);
    }
}

#[test]
fn balanced_grouping_at_the_existing_operator_bound_has_no_new_cutoff() {
    let mut source = b"fn f(a:int)->int{return ".to_vec();
    source.extend(std::iter::repeat_n(b'(', 65535));
    source.push(b'1');
    source.extend(std::iter::repeat_n(b')', 65535));
    source.extend_from_slice(b";}");
    let model = successor_parser(&source, &module_caps());
    assert_eq!(model.status, 0);
    assert_eq!(model.counts.1, 65535);
    assert_eq!(model.nodes.len(), 8);
    let historical = successor_refusal(&model);
    assert_eq!(historical.facts, 0, "preserved representation-only model");
    let expected = oracle::refused_expectation_vector(
        &source,
        &model,
        &scoped_semantics::scalar_meaning(&model),
    );
    for optimization in ["-O0", "-O2"] {
        assert_eq!(run_cached_vector(&source, &expected, optimization), 91);
    }
}

#[test]
fn grouping_does_not_lift_the_existing_borrow_argument_boundary() {
    let source = &b"fn f(a:int)->int{return g((&x),(&mut y),((1)));}"[..];
    let historical = oracle::binding_parser_stop(&module_ingest(source), source, &module_caps());
    assert_eq!(
        historical.status, 11,
        "grouping clears the preserved arg-leading flag"
    );
    let model = successor_parser(source, &module_caps());
    assert_eq!(
        (
            model.status,
            model.error_offset,
            model.error_line,
            model.error_column,
            model.diagnostic_code,
            model.diagnostic_actual
        ),
        (
            historical.status,
            historical.error_offset,
            historical.error_line,
            historical.error_column,
            historical.diagnostic_code,
            historical.diagnostic_actual
        )
    );
    let expected = oracle::expectation_vector(source, &model);
    for optimization in ["-O0", "-O2"] {
        assert_eq!(run_cached_vector(source, &expected, optimization), 91);
    }
}

#[test]
fn rich_hand_graphs_authenticate_and_refuse_before_meaning_at_o0_o2() {
    for source in [
        PARAMETERS,
        MUTATION,
        NESTED_IF,
        ELSE_IF,
        NESTED_WHILE,
        ARBITRARY_MATCH,
        RESULT_MATCH,
        TWO_FUNCTIONS,
        BORROWED_CALL,
    ] {
        let stopped = successor_parser(source, &module_caps());
        assert_eq!(stopped.status, 0);
        validate_origins(source, &stopped.nodes, &stopped.origins);
        let meaning = successor_refusal(&stopped);
        assert_eq!((meaning.facts, meaning.root_type), (0, 0));
        let current = scoped_semantics::scalar_meaning(&stopped);
        let expected = oracle::refused_expectation_vector(source, &stopped, &current);
        for optimization in ["-O0", "-O2"] {
            assert_eq!(
                run_cached_vector(source, &expected, optimization),
                91,
                "complete authentication/refusal {} at {optimization}",
                String::from_utf8_lossy(source)
            );
        }
    }
}

#[test]
fn canonical_complete_connected_source_authenticates_to_the_frozen_identifier_refusal() {
    let source = h1a_source();
    let stopped = successor_parser(source.as_bytes(), &module_caps());
    assert_eq!(stopped.status, 0);
    assert_eq!(
        stopped.nodes.len(),
        occurrence_count(&rust_projection(source.as_bytes()))
    );
    validate_origins(source.as_bytes(), &stopped.nodes, &stopped.origins);
    let meaning = successor_refusal(&stopped);
    assert_eq!(
        (
            meaning.status,
            meaning.node,
            meaning.offset,
            meaning.line,
            meaning.column,
            meaning.code
        ),
        (17, 3, 68, 2, 18, 2)
    );
    assert_eq!((meaning.symbols, meaning.facts), (23, 0));
    let current = scoped_semantics::scalar_meaning(&stopped);
    assert_eq!(
        (
            current.status,
            current.code,
            current.node,
            current.symbols,
            current.facts
        ),
        (28, 24, 1, 23, 0),
        "new rich subset refusal is explicit, not the old identifier model"
    );
    let expected = oracle::refused_expectation_vector(source.as_bytes(), &stopped, &current);
    for optimization in ["-O0", "-O2"] {
        assert_eq!(
            run_cached_vector(source.as_bytes(), &expected, optimization),
            91
        );
    }
}
const PARAMETERS: &[u8] = b"fn f(a:int,b:Result<int,int>)->int{return 1;}";
const MUTATION: &[u8] = b"fn f()->int{let mut x:int=1;x=2;return 3;}";
const NESTED_IF: &[u8] =
    b"fn f()->int{if 1{if 2{return 3;}else{return 4;}}else{return 5;}return 6;}";
const ELSE_IF: &[u8] =
    b"fn f()->int{if 1{return 2;}else if 3{let x:int=4;return 5;}else{return 6;}return 7;}";
const NESTED_WHILE: &[u8] = b"fn f()->int{while 1{while 2{return 3;}return 4;}return 5;}";
const WHILE_NO_ELSE: &[u8] = b"fn f()->int{while 1{if 2{x=3;}x=4;}return 5;}";
const WHILE_NO_ELSE_NODES: &[[i32; 4]] = &[
    [1, 1, 0, 0],
    [1, 2, 0, 0],
    [1, 3, 0, 0],
    [28, 3, 3, 0],
    [29, 0, 0, 4],
    [30, 0, 5, 0],
    [31, 0, 6, 0],
    [32, 0, 2, 7],
    [29, 0, 0, 8],
    [1, 4, 0, 0],
    [28, 3, 10, 0],
    [29, 0, 9, 11],
    [30, 0, 12, 0],
    [33, 0, 1, 13],
    [29, 0, 0, 14],
    [1, 5, 0, 0],
    [18, 0, 16, 0],
    [29, 0, 15, 17],
    [30, 0, 18, 0],
    [38, 1, 0, 19],
    [19, 1, 20, 0],
];
const WHILE_FINAL_NO_ELSE: &[u8] = b"fn f()->int{while 1{x=4;if 2{return 3;}}return 5;}";
const WHILE_FINAL_NO_ELSE_NODES: &[[i32; 4]] = &[
    [1, 1, 0, 0],
    [1, 4, 0, 0],
    [28, 3, 2, 0],
    [29, 0, 0, 3],
    [1, 2, 0, 0],
    [1, 3, 0, 0],
    [18, 0, 6, 0],
    [29, 0, 0, 7],
    [30, 0, 8, 0],
    [31, 0, 9, 0],
    [32, 0, 5, 10],
    [29, 0, 4, 11],
    [30, 0, 12, 0],
    [33, 0, 1, 13],
    [29, 0, 0, 14],
    [1, 5, 0, 0],
    [18, 0, 16, 0],
    [29, 0, 15, 17],
    [30, 0, 18, 0],
    [38, 1, 0, 19],
    [19, 1, 20, 0],
];
const ARBITRARY_MATCH: &[u8] =
    b"fn f(r:Result<int,int>)->int{return match r{Alpha(x)=>x,Beta(y)=>0-y,};}";
const RESULT_MATCH: &[u8] = b"fn f(r:Result<int,int>)->int{return match r{Ok(x)=>x,Err(y)=>0-y,};}";
const TWO_FUNCTIONS: &[u8] =
    b"fn f(a:int)->int{return 1;} fn g(b:Result<int,int>,c:int)->int{return 2;}";
const BORROWED_CALL: &[u8] = b"fn f()->int{return g(&a,&mut b,3);}";
const COMPACT_NODES: &[[i32; 4]] = &[[1, 1, 0, 0], [18, 0, 1, 0], [19, 1, 2, 0]];
const PARAMETER_NODES: &[[i32; 4]] = &[
    [24, 1, 0, 0],
    [25, 2, 1, 0],
    [24, 2, 0, 0],
    [25, 4, 3, 2],
    [1, 1, 0, 0],
    [18, 0, 5, 0],
    [29, 0, 0, 6],
    [30, 0, 7, 0],
    [38, 1, 4, 8],
    [19, 1, 9, 0],
];
const MUTATION_NODES: &[[i32; 4]] = &[
    [24, 1, 0, 0],
    [1, 1, 0, 0],
    [27, 3, 1, 2],
    [29, 0, 0, 3],
    [1, 2, 0, 0],
    [28, 3, 5, 0],
    [29, 0, 4, 6],
    [1, 3, 0, 0],
    [18, 0, 8, 0],
    [29, 0, 7, 9],
    [30, 0, 10, 0],
    [38, 1, 0, 11],
    [19, 1, 12, 0],
];
// Independently derived and reviewed before native replay, not sampled output.
const NESTED_IF_NODES: &[[i32; 4]] = &[
    [1, 1, 0, 0],
    [1, 2, 0, 0],
    [1, 3, 0, 0],
    [18, 0, 3, 0],
    [29, 0, 0, 4],
    [30, 0, 5, 0],
    [1, 4, 0, 0],
    [18, 0, 7, 0],
    [29, 0, 0, 8],
    [30, 0, 9, 0],
    [31, 0, 6, 10],
    [32, 0, 2, 11],
    [29, 0, 0, 12],
    [30, 0, 13, 0],
    [1, 5, 0, 0],
    [18, 0, 15, 0],
    [29, 0, 0, 16],
    [30, 0, 17, 0],
    [31, 0, 14, 18],
    [32, 0, 1, 19],
    [29, 0, 0, 20],
    [1, 6, 0, 0],
    [18, 0, 22, 0],
    [29, 0, 21, 23],
    [30, 0, 24, 0],
    [38, 1, 0, 25],
    [19, 1, 26, 0],
];
const ELSE_IF_NODES: &[[i32; 4]] = &[
    [1, 1, 0, 0],
    [1, 2, 0, 0],
    [18, 0, 2, 0],
    [29, 0, 0, 3],
    [30, 0, 4, 0],
    [1, 3, 0, 0],
    [24, 1, 0, 0],
    [1, 4, 0, 0],
    [26, 3, 7, 8],
    [29, 0, 0, 9],
    [1, 5, 0, 0],
    [18, 0, 11, 0],
    [29, 0, 10, 12],
    [30, 0, 13, 0],
    [1, 6, 0, 0],
    [18, 0, 15, 0],
    [29, 0, 0, 16],
    [30, 0, 17, 0],
    [31, 0, 14, 18],
    [32, 0, 6, 19],
    [31, 0, 5, 20],
    [32, 0, 1, 21],
    [29, 0, 0, 22],
    [1, 7, 0, 0],
    [18, 0, 24, 0],
    [29, 0, 23, 25],
    [30, 0, 26, 0],
    [38, 1, 0, 27],
    [19, 1, 28, 0],
];
const NESTED_WHILE_NODES: &[[i32; 4]] = &[
    [1, 1, 0, 0],
    [1, 2, 0, 0],
    [1, 3, 0, 0],
    [18, 0, 3, 0],
    [29, 0, 0, 4],
    [30, 0, 5, 0],
    [33, 0, 2, 6],
    [29, 0, 0, 7],
    [1, 4, 0, 0],
    [18, 0, 9, 0],
    [29, 0, 8, 10],
    [30, 0, 11, 0],
    [33, 0, 1, 12],
    [29, 0, 0, 13],
    [1, 5, 0, 0],
    [18, 0, 15, 0],
    [29, 0, 14, 16],
    [30, 0, 17, 0],
    [38, 1, 0, 18],
    [19, 1, 19, 0],
];
const MATCH_NODES: &[[i32; 4]] = &[
    [24, 2, 0, 0],
    [25, 2, 1, 0],
    [2, 2, 0, 0],
    [34, 7, 0, 0],
    [2, 7, 0, 0],
    [35, 6, 4, 5],
    [34, 9, 0, 0],
    [1, 0, 0, 0],
    [2, 9, 0, 0],
    [9, 0, 8, 9],
    [35, 8, 7, 10],
    [36, 0, 6, 11],
    [37, 0, 3, 12],
    [18, 0, 13, 0],
    [29, 0, 0, 14],
    [30, 0, 15, 0],
    [38, 1, 2, 16],
    [19, 1, 17, 0],
];
const TWO_FUNCTION_NODES: &[[i32; 4]] = &[
    [24, 1, 0, 0],
    [25, 2, 1, 0],
    [1, 1, 0, 0],
    [18, 0, 3, 0],
    [29, 0, 0, 4],
    [30, 0, 5, 0],
    [38, 1, 2, 6],
    [19, 1, 7, 0],
    [24, 2, 0, 0],
    [25, 5, 9, 0],
    [24, 1, 0, 0],
    [25, 7, 11, 10],
    [1, 2, 0, 0],
    [18, 0, 13, 0],
    [29, 0, 0, 14],
    [30, 0, 15, 0],
    [38, 1, 12, 16],
    [19, 4, 17, 8],
];
const BORROWED_CALL_NODES: &[[i32; 4]] = &[
    [2, 4, 0, 0],
    [22, 0, 1, 0],
    [2, 5, 0, 0],
    [23, 0, 3, 0],
    [1, 3, 0, 0],
    [21, 0, 5, 0],
    [21, 0, 4, 6],
    [21, 0, 2, 7],
    [20, 3, 8, 0],
    [18, 0, 9, 0],
    [19, 1, 10, 0],
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Int(i64),
    Name(String),
    Unary(&'static str, Box<Expr>),
    Binary(&'static str, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
    Match(String, Vec<(String, String, Expr)>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Stmt {
    Let(String, bool, String, Expr),
    Assign(String, Expr),
    Return(Expr),
    If(Expr, Vec<Stmt>, Option<Else>),
    While(Expr, Vec<Stmt>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Else {
    Block(Vec<Stmt>),
    If(Box<Stmt>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionShape {
    name: String,
    parameters: Vec<(String, String)>,
    body: Vec<Stmt>,
}

fn ast_annotation(ty: &compiler::ast::Type) -> String {
    use compiler::ast::Type;
    match ty {
        Type::Named(name) if name == "int" || name == "ByteBuffer" => name.clone(),
        Type::Generic(name, args)
            if name == "Result"
                && args.len() == 2
                && args
                    .iter()
                    .all(|arg| matches!(arg, Type::Named(name) if name == "int")) =>
        {
            "Result<int,int>".to_string()
        }
        other => panic!("type outside the frozen grammar: {other:?}"),
    }
}

fn rust_expr(expression: &compiler::ast::Expression) -> Expr {
    use compiler::ast::{ComparisonOp, Expression, LogicalOp, Pattern, UnaryOp};
    match expression {
        Expression::IntegerLiteral(value) => Expr::Int(*value),
        Expression::Identifier(name) => Expr::Name(name.clone()),
        Expression::Unary { op, operand } => Expr::Unary(
            match op {
                UnaryOp::Not => "!",
                UnaryOp::Negate => "-",
            },
            Box::new(rust_expr(operand)),
        ),
        Expression::Borrow { expr, mutable } => Expr::Unary(
            if *mutable { "&mut" } else { "&" },
            Box::new(rust_expr(expr)),
        ),
        Expression::Binary {
            op, left, right, ..
        } => Expr::Binary(
            op.as_str(),
            Box::new(rust_expr(left)),
            Box::new(rust_expr(right)),
        ),
        Expression::Comparison { op, left, right } => Expr::Binary(
            match op {
                ComparisonOp::Equal => "==",
                ComparisonOp::NotEqual => "!=",
                ComparisonOp::LessThan => "<",
                ComparisonOp::GreaterThan => ">",
                ComparisonOp::LessEqual => "<=",
                ComparisonOp::GreaterEqual => ">=",
            },
            Box::new(rust_expr(left)),
            Box::new(rust_expr(right)),
        ),
        Expression::Logical { op, left, right } => Expr::Binary(
            match op {
                LogicalOp::And => "&&",
                LogicalOp::Or => "||",
            },
            Box::new(rust_expr(left)),
            Box::new(rust_expr(right)),
        ),
        Expression::FunctionCall { name, arguments } => {
            Expr::Call(name.clone(), arguments.iter().map(rust_expr).collect())
        }
        Expression::Match { expr, arms } => {
            let Expression::Identifier(subject) = expr.as_ref() else {
                panic!("frozen Match requires identifier subject")
            };
            Expr::Match(
                subject.clone(),
                arms.iter()
                    .map(|arm| {
                        let Pattern::Enum {
                            variant,
                            data: Some(data),
                            ..
                        } = &arm.pattern
                        else {
                            panic!("frozen Match requires unary constructor binder")
                        };
                        assert_eq!(data.len(), 1);
                        let Pattern::Identifier(binder) = &data[0] else {
                            panic!("frozen Match requires named binder")
                        };
                        (variant.clone(), binder.clone(), rust_expr(&arm.body))
                    })
                    .collect(),
            )
        }
        other => panic!("expression outside the frozen grammar: {other:?}"),
    }
}

fn rust_block(block: &compiler::ast::Block) -> Vec<Stmt> {
    assert!(
        block.expression.is_none(),
        "frozen blocks use explicit statements"
    );
    block.statements.iter().map(rust_stmt).collect()
}

fn rust_stmt(statement: &compiler::ast::Statement) -> Stmt {
    use compiler::ast::{Expression, Statement};
    match statement {
        Statement::Let {
            name,
            mutable,
            type_annotation: Some(annotation),
            value: Some(value),
        } => Stmt::Let(
            name.clone(),
            *mutable,
            ast_annotation(annotation),
            rust_expr(value),
        ),
        Statement::Assignment {
            target: Expression::Identifier(name),
            value,
        } => Stmt::Assign(name.clone(), rust_expr(value)),
        Statement::Return(Some(value)) => Stmt::Return(rust_expr(value)),
        Statement::If {
            condition,
            then_block,
            else_block,
        } => Stmt::If(
            rust_expr(condition),
            rust_block(then_block),
            else_block
                .as_ref()
                .map(|otherwise| match otherwise.as_ref() {
                    Statement::Block(block) => Else::Block(rust_block(block)),
                    statement @ Statement::If { .. } => Else::If(Box::new(rust_stmt(statement))),
                    other => panic!("else outside frozen grammar: {other:?}"),
                }),
        ),
        Statement::While { condition, body } => Stmt::While(rust_expr(condition), rust_block(body)),
        other => panic!("statement outside the frozen grammar: {other:?}"),
    }
}

fn rust_projection(source: &[u8]) -> Vec<FunctionShape> {
    use compiler::ast::{AstNode, Statement};
    let source = std::str::from_utf8(source).unwrap();
    let tokens =
        compiler::try_tokenize_with_locations(source, None).expect("projection source lexes");
    let ast = compiler::parse_with_locations(tokens).expect("Rust AST parses the common grammar");
    ast.iter()
        .map(|node| {
            let AstNode::Statement(Statement::Function {
                name,
                parameters,
                return_type: Some(return_type),
                body,
                type_params,
                trait_bounds,
            }) = node
            else {
                panic!("common grammar contains only nongeneric functions")
            };
            assert!(type_params.is_empty() && trait_bounds.is_empty());
            assert_eq!(ast_annotation(return_type), "int");
            FunctionShape {
                name: name.clone(),
                parameters: parameters
                    .iter()
                    .map(|parameter| {
                        (
                            parameter.name.clone(),
                            ast_annotation(&parameter.param_type),
                        )
                    })
                    .collect(),
                body: rust_block(body),
            }
        })
        .collect()
}

fn expr_count(expression: &Expr) -> usize {
    match expression {
        Expr::Int(_) | Expr::Name(_) => 1,
        Expr::Unary(_, operand) => 1 + expr_count(operand),
        Expr::Binary(_, left, right) => 1 + expr_count(left) + expr_count(right),
        Expr::Call(_, arguments) => {
            1 + arguments.len() + arguments.iter().map(expr_count).sum::<usize>()
        }
        Expr::Match(_, arms) => {
            3 + arms
                .iter()
                .map(|(_, _, body)| 2 + expr_count(body))
                .sum::<usize>()
        }
    }
}
fn block_count(statements: &[Stmt]) -> usize {
    1 + statements.len() + statements.iter().map(stmt_count).sum::<usize>()
}
fn stmt_count(statement: &Stmt) -> usize {
    match statement {
        Stmt::Let(_, _, _, value) => 2 + expr_count(value),
        Stmt::Assign(_, value) | Stmt::Return(value) => 1 + expr_count(value),
        Stmt::If(condition, then_body, otherwise) => {
            expr_count(condition)
                + block_count(then_body)
                + 2
                + otherwise.as_ref().map_or(0, |otherwise| match otherwise {
                    Else::Block(body) => block_count(body),
                    Else::If(statement) => stmt_count(statement),
                })
        }
        Stmt::While(condition, body) => 1 + expr_count(condition) + block_count(body),
    }
}
fn occurrence_count(functions: &[FunctionShape]) -> usize {
    functions
        .iter()
        .map(|function| {
            if function.parameters.is_empty()
                && matches!(function.body.as_slice(), [Stmt::Return(_)])
            {
                1 + stmt_count(&function.body[0])
            } else {
                function.parameters.len() * 2 + block_count(&function.body) + 2
            }
        })
        .sum()
}

pub(super) fn independent_occurrence_count(source: &[u8]) -> usize {
    occurrence_count(&rust_projection(source))
}

fn control_budget(statements: &[Stmt], depth: usize) -> (usize, usize) {
    let mut records = 0;
    let mut peak = depth;
    for statement in statements {
        match statement {
            Stmt::If(_, then_body, otherwise) => {
                records += 2 + usize::from(otherwise.is_some());
                peak = peak.max(depth + 1);
                let (nested, nested_peak) = control_budget(then_body, depth + 1);
                records += nested;
                peak = peak.max(nested_peak);
                if let Some(otherwise) = otherwise {
                    let body = match otherwise {
                        Else::Block(body) => body.as_slice(),
                        Else::If(statement) => std::slice::from_ref(statement.as_ref()),
                    };
                    let (nested, nested_peak) = control_budget(body, depth + 1);
                    records += nested;
                    peak = peak.max(nested_peak);
                }
            }
            Stmt::While(_, body) => {
                records += 1;
                peak = peak.max(depth + 1);
                let (nested, nested_peak) = control_budget(body, depth + 1);
                records += nested;
                peak = peak.max(nested_peak);
            }
            _ => {}
        }
    }
    (records, peak)
}

fn interval_starts(nodes: &[[i32; 4]]) -> Vec<usize> {
    assert!(!nodes.is_empty());
    let mut starts = vec![0];
    for (index, &[kind, payload, left, right]) in nodes.iter().enumerate() {
        let id = index + 1;
        assert!(kind > 0 && payload >= 0 && left >= 0 && right >= 0);
        assert!(
            left < id as i32 && right < id as i32,
            "strictly backward occurrence edges at {id}"
        );
        let mut children: Vec<_> = [left, right]
            .into_iter()
            .filter(|child| *child != 0)
            .map(|child| child as usize)
            .collect();
        children.sort_unstable();
        let start = match children.as_slice() {
            [] => id,
            [child] => {
                assert_eq!(*child, id - 1, "one-child interval at {id}");
                starts[*child]
            }
            [early, late] => {
                assert_ne!(early, late);
                assert_eq!(*late, id - 1);
                assert_eq!(
                    *early,
                    starts[*late] - 1,
                    "adjacent disjoint intervals at {id}"
                );
                starts[*early]
            }
            _ => unreachable!(),
        };
        starts.push(start);
    }
    assert_eq!(nodes.last().unwrap()[0], 19, "complete module root");
    assert_eq!(
        *starts.last().unwrap(),
        1,
        "no gaps, shared occurrences, or orphans"
    );
    starts
}

struct Graph<'a> {
    nodes: &'a [[i32; 4]],
    names: Vec<String>,
}
impl Graph<'_> {
    fn node(&self, id: i32) -> [i32; 4] {
        assert!(id > 0);
        self.nodes[usize::try_from(id - 1).unwrap()]
    }
    fn name(&self, id: i32) -> String {
        assert!(id > 0);
        self.names[usize::try_from(id - 1).unwrap()].clone()
    }
    fn annotation(&self, id: i32) -> String {
        let [kind, code, left, right] = self.node(id);
        assert_eq!((kind, left, right), (24, 0, 0));
        match code {
            1 => "int",
            2 => "Result<int,int>",
            3 => "ByteBuffer",
            _ => panic!("unsupported annotation code"),
        }
        .to_string()
    }
    fn expression(&self, id: i32) -> Expr {
        let [kind, payload, left, right] = self.node(id);
        match kind {
            1 => Expr::Int(i64::from(payload)),
            2 => Expr::Name(self.name(payload)),
            3 | 4 | 22 | 23 => Expr::Unary(
                match kind {
                    3 => "-",
                    4 => "!",
                    22 => "&",
                    _ => "&mut",
                },
                Box::new(self.expression(left)),
            ),
            5..=17 => Expr::Binary(
                match kind {
                    5 => "*",
                    6 => "/",
                    7 => "%",
                    8 => "+",
                    9 => "-",
                    10 => "<",
                    11 => "<=",
                    12 => ">",
                    13 => ">=",
                    14 => "==",
                    15 => "!=",
                    16 => "&&",
                    _ => "||",
                },
                Box::new(self.expression(left)),
                Box::new(self.expression(right)),
            ),
            20 => {
                let mut arguments = vec![];
                let mut cell = left;
                while cell != 0 {
                    let [kind, payload, argument, next] = self.node(cell);
                    assert_eq!((kind, payload), (21, 0));
                    arguments.push(self.expression(argument));
                    cell = next;
                }
                Expr::Call(self.name(payload), arguments)
            }
            37 => {
                let [subject_kind, subject, subject_left, subject_right] = self.node(left);
                assert_eq!((subject_kind, subject_left, subject_right), (2, 0, 0));
                let [sequence_kind, sequence_payload, first, second] = self.node(right);
                assert_eq!((sequence_kind, sequence_payload), (36, 0));
                Expr::Match(
                    self.name(subject),
                    [first, second]
                        .into_iter()
                        .map(|arm| {
                            let [kind, constructor, binder, body] = self.node(arm);
                            assert_eq!(kind, 35);
                            let [kind, name, left, right] = self.node(binder);
                            assert_eq!((kind, left, right), (34, 0, 0));
                            (
                                self.name(constructor),
                                self.name(name),
                                self.expression(body),
                            )
                        })
                        .collect(),
                )
            }
            _ => panic!("unsupported expression node {kind}"),
        }
    }
    fn statement(&self, id: i32) -> Stmt {
        let [kind, payload, left, right] = self.node(id);
        match kind {
            18 => Stmt::Return(self.expression(left)),
            26 | 27 => Stmt::Let(
                self.name(payload),
                kind == 27,
                self.annotation(left),
                self.expression(right),
            ),
            28 => Stmt::Assign(self.name(payload), self.expression(left)),
            32 => {
                let [kind, payload, then_block, otherwise] = self.node(right);
                assert_eq!((kind, payload), (31, 0));
                let otherwise = if otherwise == 0 {
                    None
                } else {
                    Some(match self.node(otherwise)[0] {
                        30 => Else::Block(self.block(otherwise)),
                        32 => Else::If(Box::new(self.statement(otherwise))),
                        other => panic!("invalid else child {other}"),
                    })
                };
                Stmt::If(self.expression(left), self.block(then_block), otherwise)
            }
            33 => Stmt::While(self.expression(left), self.block(right)),
            _ => panic!("unsupported statement node {kind}"),
        }
    }
    fn block(&self, id: i32) -> Vec<Stmt> {
        let [kind, payload, mut cell, right] = self.node(id);
        assert_eq!((kind, payload, right), (30, 0, 0));
        let mut statements = vec![];
        while cell != 0 {
            let [kind, payload, previous, statement] = self.node(cell);
            assert_eq!((kind, payload), (29, 0));
            statements.push(self.statement(statement));
            cell = previous;
        }
        statements.reverse();
        statements
    }
    fn functions(&self) -> Vec<FunctionShape> {
        interval_starts(self.nodes);
        let mut item = self.nodes.len() as i32;
        let mut functions = vec![];
        while item != 0 {
            let [kind, name, body, previous] = self.node(item);
            assert_eq!(kind, 19);
            let (parameters, body) = match self.node(body)[0] {
                18 => (vec![], vec![self.statement(body)]),
                38 => {
                    let [kind, payload, mut parameter, block] = self.node(body);
                    assert_eq!((kind, payload), (38, 1));
                    let mut parameters = vec![];
                    while parameter != 0 {
                        let [kind, name, annotation, previous] = self.node(parameter);
                        assert_eq!(kind, 25);
                        parameters.push((self.name(name), self.annotation(annotation)));
                        parameter = previous;
                    }
                    parameters.reverse();
                    (parameters, self.block(block))
                }
                other => panic!("invalid function body {other}"),
            };
            functions.push(FunctionShape {
                name: self.name(name),
                parameters,
                body,
            });
            item = previous;
        }
        functions.reverse();
        functions
    }
}

fn graph_projection(source: &[u8], nodes: &[[i32; 4]]) -> Vec<FunctionShape> {
    let ingested = module_ingest(source);
    let names = ingested
        .names
        .iter()
        .map(|&(at, length)| {
            std::str::from_utf8(&source[at as usize..(at + length) as usize])
                .unwrap()
                .to_string()
        })
        .collect();
    Graph { nodes, names }.functions()
}

fn validate_shapes(nodes: &[[i32; 4]], name_count: usize) {
    interval_starts(nodes);
    let kind_of = |id: i32| nodes[usize::try_from(id - 1).unwrap()][0];
    let expression = |id: i32| id > 0 && matches!(kind_of(id), 1..=17 | 20 | 22 | 23 | 37);
    let named = |name: i32| name > 0 && (name as usize) <= name_count;
    for &[kind, payload, left, right] in nodes {
        match kind {
            1 => assert!(payload >= 0 && left == 0 && right == 0),
            2 | 34 => assert!(named(payload) && left == 0 && right == 0),
            3 | 4 | 18 | 22 | 23 => assert!(payload == 0 && expression(left) && right == 0),
            5..=17 => assert!(payload == 0 && expression(left) && expression(right)),
            19 => assert!(
                named(payload)
                    && left > 0
                    && matches!(kind_of(left), 18 | 38)
                    && (right == 0 || kind_of(right) == 19)
            ),
            20 => assert!(named(payload) && (left == 0 || kind_of(left) == 21) && right == 0),
            21 => assert!(payload == 0 && expression(left) && (right == 0 || kind_of(right) == 21)),
            24 => assert!((1..=3).contains(&payload) && left == 0 && right == 0),
            25 => assert!(
                named(payload)
                    && left > 0
                    && kind_of(left) == 24
                    && (1..=2).contains(&nodes[(left - 1) as usize][1])
                    && (right == 0 || kind_of(right) == 25)
            ),
            26 | 27 => {
                assert!(named(payload) && left > 0 && kind_of(left) == 24 && expression(right))
            }
            28 => assert!(named(payload) && expression(left) && right == 0),
            29 => assert!(
                payload == 0
                    && (left == 0 || kind_of(left) == 29)
                    && right > 0
                    && matches!(kind_of(right), 18 | 26 | 27 | 28 | 32 | 33)
            ),
            30 => assert!(payload == 0 && left > 0 && kind_of(left) == 29 && right == 0),
            31 => assert!(
                payload == 0
                    && left > 0
                    && kind_of(left) == 30
                    && (right == 0 || matches!(kind_of(right), 30 | 32))
            ),
            32 => assert!(payload == 0 && expression(left) && right > 0 && kind_of(right) == 31),
            33 => assert!(payload == 0 && expression(left) && right > 0 && kind_of(right) == 30),
            35 => assert!(named(payload) && left > 0 && kind_of(left) == 34 && expression(right)),
            36 => assert!(
                payload == 0
                    && left > 0
                    && kind_of(left) == 35
                    && right > 0
                    && kind_of(right) == 35
            ),
            37 => assert!(
                payload == 0 && left > 0 && kind_of(left) == 2 && right > 0 && kind_of(right) == 36
            ),
            38 => assert!(
                payload == 1
                    && (left == 0 || kind_of(left) == 25)
                    && right > 0
                    && kind_of(right) == 30
            ),
            other => panic!("unfrozen node kind {other}"),
        }
    }
}

fn validate_origins(source: &[u8], nodes: &[[i32; 4]], origins: &[[i32; 5]]) {
    let lexed = module_ingest(source);
    validate_shapes(nodes, lexed.names.len());
    assert_eq!(nodes.len(), origins.len());
    let token_word = |token: &[i32; 6]| &source[token[1] as usize..(token[1] + token[2]) as usize];
    for (index, (&[kind, payload, left, right], origin)) in nodes.iter().zip(origins).enumerate() {
        assert_eq!(origin[0], index as i32 + 1);
        let token_index = lexed
            .tokens
            .binary_search_by_key(&origin[1], |token| token[1])
            .expect("origin must identify an exact source token");
        let token = lexed.tokens[token_index];
        assert_eq!([token[1], token[3], token[4], token[0]], origin[1..]);
        let expected_kind = match kind {
            1 => 2,
            2 | 24 | 25 | 28 | 34 | 35 | 36 | 37 => 1,
            3 | 9 => 21,
            4 => 27,
            5 => 22,
            6 => 23,
            7 => 24,
            8 => 20,
            10 => 29,
            11 => 30,
            12 => 31,
            13 => 32,
            14 => 26,
            15 => 28,
            16 => 33,
            17 => 34,
            18 => 6,
            19 | 38 => 3,
            20 => 10,
            21 => 11,
            22 | 23 => 37,
            26 | 27 => 4,
            30 => 12,
            31 | 32 => 7,
            33 => 9,
            29 => origins[(right - 1) as usize][4],
            _ => panic!("unfrozen origin role"),
        };
        assert_eq!(token[0], expected_kind);
        if matches!(kind, 2 | 25 | 28 | 34 | 35) {
            assert_eq!(
                token[5], payload,
                "name payload must authenticate at its own declaration/use token"
            );
        }
        match kind {
            1 => assert_eq!(
                std::str::from_utf8(token_word(&token))
                    .unwrap()
                    .parse::<i32>()
                    .unwrap(),
                payload
            ),
            24 => {
                let spelling = token_word(&token);
                match payload {
                    1 => assert_eq!(spelling, b"int"),
                    3 => assert_eq!(spelling, b"ByteBuffer"),
                    2 => {
                        assert_eq!(spelling, b"Result");
                        let following = &lexed.tokens[token_index + 1..token_index + 6];
                        assert_eq!(
                            following.iter().map(|token| token[0]).collect::<Vec<_>>(),
                            vec![29, 1, 16, 1, 31]
                        );
                        assert_eq!(token_word(&following[1]), b"int");
                        assert_eq!(token_word(&following[3]), b"int");
                    }
                    _ => panic!("unknown type spelling"),
                }
            }
            19 => assert_eq!(
                lexed.tokens[token_index + 1][5],
                payload,
                "function payload at its own fn declaration"
            ),
            20 => assert_eq!(
                lexed.tokens[token_index - 1][5],
                payload,
                "call payload at its actual callee"
            ),
            25 => {
                assert_eq!(lexed.tokens[token_index + 1][0], 17);
                assert_eq!(
                    origins[(left - 1) as usize][1],
                    lexed.tokens[token_index + 2][1]
                );
            }
            28 => assert_eq!(lexed.tokens[token_index + 1][0], 25),
            26 | 27 => {
                let name_index = token_index
                    + if kind == 27 {
                        assert_eq!(lexed.tokens[token_index + 1][0], 5);
                        2
                    } else {
                        1
                    };
                assert_eq!(lexed.tokens[name_index][5], payload);
                assert_eq!(lexed.tokens[name_index + 1][0], 17);
                assert_eq!(
                    origins[(left - 1) as usize][1],
                    lexed.tokens[name_index + 2][1]
                );
            }
            34 => {
                assert_eq!(lexed.tokens[token_index - 1][0], 10);
                assert_eq!(lexed.tokens[token_index + 1][0], 11);
            }
            35 => {
                assert_eq!(lexed.tokens[token_index + 1][0], 10);
                assert_eq!(
                    origins[(left - 1) as usize][1],
                    lexed.tokens[token_index + 2][1]
                );
            }
            36 | 37 => assert_eq!(token_word(&token), b"match"),
            22 => assert_ne!(lexed.tokens[token_index + 1][0], 5),
            23 => assert_eq!(lexed.tokens[token_index + 1][0], 5),
            _ => {}
        }
        match kind {
            29 => assert_eq!(
                &origin[1..],
                &origins[(right - 1) as usize][1..],
                "statement cell copies the owned statement origin"
            ),
            32 => assert_eq!(
                &origin[1..],
                &origins[(right - 1) as usize][1..],
                "branch pair copies its owning If origin"
            ),
            19 if nodes[(left - 1) as usize][0] == 38 => assert_eq!(
                &origin[1..],
                &origins[(left - 1) as usize][1..],
                "descriptor copies its owning function origin"
            ),
            37 => assert_eq!(
                &origin[1..],
                &origins[(right - 1) as usize][1..],
                "arm sequence copies its owning Match origin"
            ),
            _ => {}
        }
    }
    let brace_close = |open_offset: i32| {
        let open = lexed
            .tokens
            .binary_search_by_key(&open_offset, |token| token[1])
            .unwrap();
        assert_eq!(lexed.tokens[open][0], 12);
        let mut depth = 0;
        for token in &lexed.tokens[open..] {
            if token[0] == 12 {
                depth += 1;
            }
            if token[0] == 13 {
                depth -= 1;
                if depth == 0 {
                    return token[1];
                }
            }
        }
        panic!("unterminated authenticated block");
    };
    for (index, &[kind, _, left, right]) in nodes.iter().enumerate() {
        let at = origins[index][1];
        match kind {
            19 if right != 0 => assert!(
                origins[(right - 1) as usize][1] < at,
                "module predecessors must preserve source order"
            ),
            25 if right != 0 => assert!(
                origins[(right - 1) as usize][1] < at,
                "parameter predecessors must preserve source order"
            ),
            29 if left != 0 => assert!(
                origins[(left - 1) as usize][1] < at,
                "statement predecessors must preserve source order"
            ),
            30 => {
                let close = brace_close(at);
                let mut cell = left;
                while cell != 0 {
                    let cell_index = (cell - 1) as usize;
                    assert!(
                        at < origins[cell_index][1] && origins[cell_index][1] < close,
                        "statement occurrence belongs to its actual block"
                    );
                    cell = nodes[cell_index][2];
                }
            }
            35 => assert!(
                at < origins[(left - 1) as usize][1]
                    && origins[(left - 1) as usize][1] < origins[(right - 1) as usize][1],
                "binder occurrence precedes its owned arm body"
            ),
            36 => assert!(
                origins[(left - 1) as usize][1] < origins[(right - 1) as usize][1],
                "Match arms preserve source order even with equal names"
            ),
            38 => {
                let header = lexed
                    .tokens
                    .binary_search_by_key(&at, |token| token[1])
                    .unwrap();
                let own_open = lexed.tokens[header + 1..]
                    .iter()
                    .find(|token| token[0] == 12)
                    .unwrap()[1];
                let body_open = origins[(right - 1) as usize][1];
                assert_eq!(
                    body_open, own_open,
                    "descriptor owns its own function body brace"
                );
                let mut parameter = left;
                while parameter != 0 {
                    let parameter_index = (parameter - 1) as usize;
                    let declaration = origins[parameter_index][1];
                    assert!(
                        at < declaration && declaration < body_open,
                        "parameter occurrence belongs to its own header"
                    );
                    parameter = nodes[parameter_index][3];
                }
            }
            _ => {}
        }
    }
}

fn origins(offsets: &[i32], kinds: &[i32]) -> Vec<[i32; 5]> {
    assert_eq!(offsets.len(), kinds.len());
    offsets
        .iter()
        .zip(kinds)
        .enumerate()
        .map(|(index, (&at, &kind))| [i32::try_from(index + 1).unwrap(), at, 1, at + 1, kind])
        .collect()
}

#[derive(Debug)]
struct Event {
    kind: u32,
    owner: u32,
    old_size: u64,
    new_size: u64,
    succeeded: bool,
}

#[derive(Debug)]
struct Owner {
    id: u32,
    bytes: Vec<u8>,
}

#[derive(Debug)]
pub(super) struct Capture {
    result: i32,
    pub(super) counters: [u64; 5],
    events: Vec<Event>,
    owners: Vec<Owner>,
}

fn u32_word(input: &mut impl Read) -> u32 {
    let mut word = [0; 4];
    input.read_exact(&mut word).expect("complete observer u32");
    u32::from_le_bytes(word)
}
fn u64_word(input: &mut impl Read) -> u64 {
    let mut word = [0; 8];
    input.read_exact(&mut word).expect("complete observer u64");
    u64::from_le_bytes(word)
}

impl Capture {
    fn read(bytes: &[u8]) -> Self {
        let mut input = std::io::Cursor::new(bytes);
        let mut magic = [0; 8];
        input.read_exact(&mut magic).unwrap();
        assert_eq!(&magic, b"ASTOBS01");
        let result = u32_word(&mut input) as i32;
        let owners = u32_word(&mut input);
        let events = u32_word(&mut input);
        assert!(owners <= 128 && events <= 4096, "bounded observer artifact");
        let counters = std::array::from_fn(|_| u64_word(&mut input));
        let events = (0..events)
            .map(|_| Event {
                kind: u32_word(&mut input),
                owner: u32_word(&mut input),
                old_size: u64_word(&mut input),
                new_size: u64_word(&mut input),
                succeeded: u32_word(&mut input) == 1,
            })
            .collect();
        let owners = (0..owners)
            .map(|_| {
                let id = u32_word(&mut input);
                let length = u64_word(&mut input);
                assert!(length <= 8 * 1024 * 1024, "bounded captured capacity");
                let mut bytes = vec![0; usize::try_from(length).unwrap()];
                input.read_exact(&mut bytes).unwrap();
                Owner { id, bytes }
            })
            .collect();
        assert_eq!(
            input.position(),
            bytes.len() as u64,
            "no trailing observer bytes"
        );
        Self {
            result,
            counters,
            events,
            owners,
        }
    }

    fn owner(&self, id: u32) -> &[u8] {
        &self
            .owners
            .iter()
            .find(|owner| owner.id == id)
            .expect("predicted owner was captured")
            .bytes
    }

    fn validate_trace(&self, cleanup: &[u32]) {
        self.validate_trace_at(cleanup, 80);
    }

    pub(super) fn validate_trace_at(&self, cleanup: &[u32], result: i32) {
        assert_eq!(
            self.result, result,
            "exact full-vector or parser-only capture exit"
        );
        assert_eq!(
            self.counters[3..],
            [0, 0],
            "no live owners or size mismatches"
        );
        let allocations: Vec<_> = self.events.iter().filter(|event| event.kind == 1).collect();
        assert_eq!(
            allocations.len(),
            cleanup.len(),
            "pre-derived first-use owner count"
        );
        for (index, event) in allocations.iter().enumerate() {
            assert_eq!(event.owner, u32::try_from(index + 1).unwrap());
            assert!(event.succeeded);
            assert_eq!(
                (event.old_size, event.new_size),
                (0, 8),
                "first byte push allocates eight"
            );
        }
        let releases: Vec<_> = self
            .events
            .iter()
            .filter(|event| event.kind == 3)
            .map(|event| event.owner)
            .collect();
        assert_eq!(
            releases, cleanup,
            "independently derived cleanup identity order"
        );
        assert_eq!(self.owners.len(), cleanup.len());
        assert_eq!(self.counters[0], allocations.len() as u64);
        assert_eq!(
            self.counters[1],
            self.events.iter().filter(|event| event.kind == 2).count() as u64
        );
        assert_eq!(self.counters[2], releases.len() as u64);
        let mut capacities = vec![0_u64; cleanup.len() + 1];
        for event in &self.events {
            let id = usize::try_from(event.owner).unwrap();
            assert!(id > 0 && id < capacities.len());
            match event.kind {
                1 => {
                    assert_eq!(capacities[id], 0);
                    capacities[id] = event.new_size;
                }
                2 => {
                    assert_eq!(capacities[id], event.old_size);
                    assert_eq!(
                        event.new_size,
                        event.old_size * 2,
                        "growth doubles existing owner"
                    );
                    assert!(event.succeeded);
                    capacities[id] = event.new_size;
                }
                3 => {
                    assert_eq!(capacities[id], event.old_size);
                    assert_eq!(event.new_size, event.old_size);
                    assert!(event.succeeded);
                    assert_eq!(self.owner(event.owner).len() as u64, event.old_size);
                    capacities[id] = 0;
                }
                other => panic!("unknown observer event {other}"),
            }
        }
        assert!(capacities.iter().all(|capacity| *capacity == 0));
    }

    pub(super) fn records<const N: usize>(&self, id: u32) -> Vec<[i32; N]> {
        let bytes = self.owner(id);
        assert_eq!(bytes.len() % 4, 0, "word-aligned physical capacity");
        let words = bytes
            .chunks_exact(4)
            .take_while(|word| word[3] < 128)
            .count();
        let used = words * 4;
        assert!(
            bytes[used..].iter().all(|byte| *byte == 0xa5),
            "partial write or non-poison tail on owner {id}"
        );
        assert_eq!(
            words % N,
            0,
            "complete logical {N}-word records on owner {id}"
        );
        bytes[..used]
            .chunks_exact(N * 4)
            .map(|record| {
                std::array::from_fn(|field| {
                    i32::from_le_bytes(record[field * 4..field * 4 + 4].try_into().unwrap())
                })
            })
            .collect()
    }

    pub(super) fn failure_descriptor(&self) -> [u64; 4] {
        let failed: Vec<_> = self
            .events
            .iter()
            .filter(|event| !event.succeeded)
            .collect();
        assert_eq!(failed.len(), 1, "exactly one injected allocation failure");
        let event = failed[0];
        [
            u64::from(event.kind),
            u64::from(event.owner),
            event.old_size,
            event.new_size,
        ]
    }
}

const OBSERVER: &str = r#"
#define _CRT_SECURE_NO_WARNINGS
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define aero_alloc ast_runtime_alloc
#define aero_realloc ast_runtime_realloc
#define aero_dealloc ast_runtime_dealloc
#include "RUNTIME_PATH"
#undef aero_alloc
#undef aero_realloc
#undef aero_dealloc

typedef struct { void *pointer; uint64_t size; unsigned char *snapshot; int live; } AstOwner;
typedef struct { uint32_t kind, id; uint64_t old_size, new_size; uint32_t succeeded; } AstEvent;
static AstOwner ast_owners[128];
static AstEvent ast_events[4096];
static uint32_t ast_owner_count, ast_event_count;
static int ast_observer_error;

static uint32_t ast_owner_id(void *pointer) {
    for (uint32_t i = 0; i < ast_owner_count; i++)
        if (ast_owners[i].live && ast_owners[i].pointer == pointer) return i + 1;
    ast_observer_error = 1;
    return 0;
}
static void ast_event(uint32_t kind, uint32_t id, uint64_t old_size, uint64_t new_size, uint32_t succeeded) {
    if (ast_event_count == 4096) { ast_observer_error = 1; return; }
    ast_events[ast_event_count++] = (AstEvent){kind, id, old_size, new_size, succeeded};
}
void *aero_alloc(uint64_t size) {
    void *pointer = ast_runtime_alloc(size);
    if (pointer == NULL) { ast_event(1, 0, 0, size, 0); return NULL; }
    if (ast_owner_count == 128) { ast_observer_error = 1; return pointer; }
    uint32_t id = ++ast_owner_count;
    ast_owners[id - 1] = (AstOwner){pointer, size, NULL, 1};
    memset(pointer, 0xa5, (size_t)size);
    ast_event(1, id, 0, size, 1);
    return pointer;
}
void *aero_realloc(void *old, uint64_t old_size, uint64_t new_size) {
    uint32_t id = ast_owner_id(old);
    unsigned char *before = NULL;
    if (id != 0 && ast_owners[id - 1].size == old_size) {
        before = (unsigned char *)malloc((size_t)old_size);
        if (before != NULL) memcpy(before, old, (size_t)old_size);
        else ast_observer_error = 1;
    } else ast_observer_error = 1;
    void *pointer = ast_runtime_realloc(old, old_size, new_size);
    ast_event(2, id, old_size, new_size, pointer != NULL);
    if (id != 0) {
        if (pointer != NULL) {
            ast_owners[id - 1].pointer = pointer;
            ast_owners[id - 1].size = new_size;
            if (new_size > old_size) memset((unsigned char *)pointer + old_size, 0xa5, (size_t)(new_size - old_size));
        } else if (before != NULL && memcmp(before, old, (size_t)old_size) != 0) ast_observer_error = 1;
    }
    free(before);
    return pointer;
}
void aero_dealloc(void *pointer, uint64_t size) {
    uint32_t id = ast_owner_id(pointer);
    if (id != 0 && ast_owners[id - 1].size == size) {
        AstOwner *owner = &ast_owners[id - 1];
        owner->snapshot = (unsigned char *)malloc((size_t)size);
        if (owner->snapshot != NULL) memcpy(owner->snapshot, pointer, (size_t)size);
        else ast_observer_error = 1;
        owner->live = 0;
    } else ast_observer_error = 1;
    ast_event(3, id, size, size, 1);
    ast_runtime_dealloc(pointer, size);
}
static int ast_u32(FILE *file, uint32_t value) {
    unsigned char bytes[4];
    for (int i = 0; i < 4; i++) bytes[i] = (unsigned char)(value >> (i * 8));
    return fwrite(bytes, 1, 4, file) == 4;
}
static int ast_u64(FILE *file, uint64_t value) {
    unsigned char bytes[8];
    for (int i = 0; i < 8; i++) bytes[i] = (unsigned char)(value >> (i * 8));
    return fwrite(bytes, 1, 8, file) == 8;
}
static int ast_write_capture(const char *path, int32_t result) {
    FILE *file = fopen(path, "wb");
    if (file == NULL) return 0;
    int ok = fwrite("ASTOBS01", 1, 8, file) == 8;
    ok = ast_u32(file, (uint32_t)result) && ok;
    ok = ast_u32(file, ast_owner_count) && ok;
    ok = ast_u32(file, ast_event_count) && ok;
    ok = ast_u64(file, aero_test_alloc_calls()) && ok;
    ok = ast_u64(file, aero_test_realloc_calls()) && ok;
    ok = ast_u64(file, aero_test_dealloc_calls()) && ok;
    ok = ast_u64(file, aero_test_live_allocations()) && ok;
    ok = ast_u64(file, aero_test_size_mismatch_calls()) && ok;
    for (uint32_t i = 0; i < ast_event_count; i++) {
        AstEvent event = ast_events[i];
        ok = ast_u32(file, event.kind) && ok; ok = ast_u32(file, event.id) && ok;
        ok = ast_u64(file, event.old_size) && ok; ok = ast_u64(file, event.new_size) && ok;
        ok = ast_u32(file, event.succeeded) && ok;
    }
    for (uint32_t i = 0; i < ast_owner_count; i++) {
        AstOwner owner = ast_owners[i];
        ok = ast_u32(file, i + 1) && ok; ok = ast_u64(file, owner.size) && ok;
        if (owner.snapshot == NULL) ok = 0;
        else ok = fwrite(owner.snapshot, 1, (size_t)owner.size, file) == owner.size && ok;
    }
    if (fclose(file) != 0) ok = 0;
    for (uint32_t i = 0; i < ast_owner_count; i++) free(ast_owners[i].snapshot);
    return ok && !ast_observer_error;
}
"#;

struct NativeProduct {
    object: Vec<u8>,
    exact_llvm: &'static str,
    optimization: String,
    llvm_path: PathBuf,
    llvm_version: Vec<u8>,
}

fn native_product(optimization: &str) -> &'static NativeProduct {
    static O0: OnceLock<NativeProduct> = OnceLock::new();
    static O2: OnceLock<NativeProduct> = OnceLock::new();
    let slot = match optimization {
        "-O0" => &O0,
        "-O2" => &O2,
        _ => panic!("uncontracted optimization"),
    };
    let llvm_path =
        fs::canonicalize(llvm_bin().join(if cfg!(windows) { "clang.exe" } else { "clang" }))
            .expect("canonical native compiler path");
    let version = Command::new(&llvm_path)
        .arg("--version")
        .output()
        .expect("read native compiler identity");
    assert!(version.status.success());
    let product = slot.get_or_init(|| {
        let workspace = TestWorkspace::new("connected-ast-native-object");
        let exact_llvm = compiled_h1a();
        let llvm = workspace.write("product.ll", renamed_product(exact_llvm));
        let object = workspace.root.join("product.o");
        let output = Command::new(&llvm_path)
            .args([optimization, "-Wno-error=override-module", "-c"])
            .arg(&llvm)
            .arg("-o")
            .arg(&object)
            .output()
            .expect("compile actual native product object");
        assert!(
            output.status.success(),
            "native product {optimization}: {output:?}"
        );
        NativeProduct {
            object: fs::read(object).expect("read immutable native object bytes"),
            exact_llvm,
            optimization: optimization.to_string(),
            llvm_path: llvm_path.clone(),
            llvm_version: version.stdout.clone(),
        }
    });
    assert_eq!(
        product.exact_llvm.as_bytes(),
        compiled_h1a().as_bytes(),
        "cached object requires exact product bytes"
    );
    assert_eq!(product.optimization, optimization);
    assert_eq!(product.llvm_path, llvm_path);
    assert_eq!(product.llvm_version, version.stdout);
    product
}

fn capture(source: &[u8], optimization: &str) -> Capture {
    let ingested = module_ingest(source);
    let stopped = oracle::binding_parser_stop(&ingested, source, &module_caps());
    assert_eq!(
        stopped.status, 0,
        "observer input must parse in the predecessor"
    );
    let mut expected = oracle::expectation_vector(source, &stopped);
    expected[8] = 0; // Every valid module has a node: force parser-only return 80.
    let workspace = TestWorkspace::new("connected-ast-observer");
    let product = native_product(optimization);
    let object = workspace.write("product.o", &product.object);
    let runtime = repository_path("../../src/compiler/runtime/aero_test_runtime.c");
    let include = runtime.to_string_lossy().replace('\\', "/");
    let observer = OBSERVER.replace("RUNTIME_PATH", &include);
    let harness = expectation_harness(&expected, source, stopped.consumed);
    assert_eq!(harness.matches("int main(void) {").count(), 1);
    assert_eq!(
        harness
            .matches("    if (wrote_output != 0) return 62;")
            .count(),
        1
    );
    let harness = harness.replace("int main(void) {", "int main(int argc, char **argv) {\n    if (argc != 2) return 58;")
        .replace("    if (wrote_output != 0) return 62;", "    if (!ast_write_capture(argv[1], result)) return 59;\n    if (wrote_output != 0) return 62;");
    let harness = workspace.write("observer.c", format!("{observer}\n{harness}"));
    let executable = clang_link(
        &workspace,
        "observer",
        optimization,
        &[object.as_path(), harness.as_path()],
    );
    let artifact = workspace.root.join("actual-arenas.bin");
    let output = Command::new(executable)
        .arg(&artifact)
        .output()
        .expect("run actual native arena observer");
    assert_eq!(
        output.status.code(),
        Some(80),
        "parser-only native capture at {optimization}: {output:?}"
    );
    assert!(
        output.stdout.is_empty() && output.stderr.is_empty(),
        "observer or product emitted forbidden channel bytes: {output:?}"
    );
    let capture =
        Capture::read(&fs::read(artifact).expect("read actual native allocation artifact"));
    let source_owner = capture.owner(1);
    assert_eq!(&source_owner[..source.len()], source);
    assert!(
        source_owner[source.len()..]
            .iter()
            .all(|byte| *byte == 0xa5)
    );
    let expected_tokens: Vec<u8> = ingested
        .tokens
        .iter()
        .flatten()
        .flat_map(|word| word.to_le_bytes())
        .collect();
    let tokens = capture.owner(2);
    assert_eq!(&tokens[..expected_tokens.len()], &expected_tokens);
    assert!(
        tokens[expected_tokens.len()..]
            .iter()
            .all(|byte| *byte == 0xa5)
    );
    let expected_names: Vec<u8> = ingested
        .names
        .iter()
        .flat_map(|&(at, length)| [at, length])
        .flat_map(|word| word.to_le_bytes())
        .collect();
    let names = capture.owner(3);
    assert_eq!(&names[..expected_names.len()], &expected_names);
    assert!(
        names[expected_names.len()..]
            .iter()
            .all(|byte| *byte == 0xa5)
    );
    capture
}

#[test]
fn compact_native_arena_observer_matches_frozen_graph_and_owner_trace() {
    let expected_origins = origins(&[19, 12, 0], &[2, 6, 3]);
    for optimization in ["-O0", "-O2"] {
        let capture = capture(COMPACT, optimization);
        capture.validate_trace(&[4, 6, 5, 2, 3, 1]);
        assert_eq!(capture.records::<4>(5), COMPACT_NODES);
        assert_eq!(capture.records::<5>(4), expected_origins);
        validate_origins(COMPACT, &capture.records::<4>(5), &capture.records::<5>(4));
        assert_eq!(
            graph_projection(COMPACT, &capture.records::<4>(5)),
            rust_projection(COMPACT)
        );
    }
}

fn assert_future_graph(
    source: &[u8],
    nodes: &[[i32; 4]],
    expected_origins: Vec<[i32; 5]>,
    trace: u32,
) {
    // Finish both native optimizations before reporting the intentional graph red.
    let observations: Vec<_> = ["-O0", "-O2"]
        .into_iter()
        .map(|optimization| {
            let capture = capture(source, optimization);
            let (cleanup, node_owner, origin_owner) = match trace {
                1 => (&[5, 7, 4, 6, 2, 3, 1][..], 6, 5),
                2 => (&[4, 6, 7, 5, 2, 3, 1][..], 5, 4),
                3 => (&[5, 8, 7, 4, 6, 2, 3, 1][..], 6, 5),
                4 => (&[6, 5, 8, 4, 7, 2, 3, 1][..], 7, 6),
                0 => (&[4, 6, 5, 2, 3, 1][..], 5, 4),
                _ => panic!("unfrozen owner trace"),
            };
            capture.validate_trace(cleanup);
            (
                optimization,
                capture.records::<4>(node_owner),
                capture.records::<5>(origin_owner),
            )
        })
        .collect();
    let differences: Vec<_> = observations
        .iter()
        .filter(|(_, actual_nodes, actual_origins)| {
            actual_nodes.as_slice() != nodes || *actual_origins != expected_origins
        })
        .collect();
    assert!(
        differences.is_empty(),
        "connected AST hand-graph mismatch at completed native O0/O2: expected nodes={nodes:?}, origins={expected_origins:?}; actual={differences:?}"
    );
    for (_, actual, actual_origins) in &observations {
        validate_origins(source, actual, actual_origins);
    }
    if source == ARBITRARY_MATCH {
        // The predecessor admits arbitrary bare constructors; Rust does not.
        // Do not rewrite Alpha/Beta or claim a source-identical Rust comparison.
        let hand = vec![FunctionShape {
            name: "f".into(),
            parameters: vec![("r".into(), "Result<int,int>".into())],
            body: vec![Stmt::Return(Expr::Match(
                "r".into(),
                vec![
                    ("Alpha".into(), "x".into(), Expr::Name("x".into())),
                    (
                        "Beta".into(),
                        "y".into(),
                        Expr::Binary(
                            "-",
                            Box::new(Expr::Int(0)),
                            Box::new(Expr::Name("y".into())),
                        ),
                    ),
                ],
            ))],
        }];
        for (_, actual, _) in &observations {
            assert_eq!(graph_projection(source, actual), hand);
        }
    } else {
        let projected = rust_projection(source);
        assert_eq!(nodes.len(), occurrence_count(&projected));
        for (_, actual, _) in &observations {
            assert_eq!(
                graph_projection(source, actual),
                projected,
                "independent normalized source-identical Rust AST equality"
            );
        }
    }
}

#[test]
fn typed_parameters_native_arena_matches_frozen_connected_graph() {
    assert_future_graph(
        PARAMETERS,
        PARAMETER_NODES,
        origins(
            &[7, 5, 13, 11, 42, 35, 35, 34, 0, 0],
            &[1, 1, 1, 1, 2, 6, 6, 12, 3, 3],
        ),
        1,
    );
}

#[test]
fn mutable_statements_native_arena_matches_frozen_connected_graph() {
    assert_future_graph(
        MUTATION,
        MUTATION_NODES,
        origins(
            &[22, 26, 12, 12, 30, 28, 28, 39, 32, 32, 11, 0, 0],
            &[1, 2, 4, 4, 2, 1, 1, 2, 6, 6, 12, 3, 3],
        ),
        0,
    );
}

fn nested_if_origins() -> Vec<[i32; 5]> {
    origins(
        &[
            15, 20, 29, 22, 22, 21, 44, 37, 37, 36, 17, 17, 17, 16, 60, 53, 53, 52, 12, 12, 12, 70,
            63, 63, 11, 0, 0,
        ],
        &[
            2, 2, 2, 6, 6, 12, 2, 6, 6, 12, 7, 7, 7, 12, 2, 6, 6, 12, 7, 7, 7, 2, 6, 6, 12, 3, 3,
        ],
    )
}

#[test]
fn nested_if_native_arena_matches_frozen_connected_graph() {
    assert_future_graph(NESTED_IF, NESTED_IF_NODES, nested_if_origins(), 2);
}

#[test]
fn else_if_native_arena_preserves_asymmetric_branch_ownership() {
    assert_future_graph(
        ELSE_IF,
        ELSE_IF_NODES,
        origins(
            &[
                15, 24, 17, 17, 16, 35, 43, 47, 37, 37, 56, 49, 49, 36, 71, 64, 64, 63, 32, 32, 12,
                12, 12, 81, 74, 74, 11, 0, 0,
            ],
            &[
                2, 2, 6, 6, 12, 2, 1, 2, 4, 4, 2, 6, 6, 12, 2, 6, 6, 12, 7, 7, 7, 7, 7, 2, 6, 6,
                12, 3, 3,
            ],
        ),
        2,
    );
}

#[test]
fn nested_while_native_arena_preserves_distinct_returns_and_parent_blocks() {
    assert_future_graph(
        NESTED_WHILE,
        NESTED_WHILE_NODES,
        origins(
            &[
                18, 26, 35, 28, 28, 27, 20, 20, 45, 38, 38, 19, 12, 12, 55, 48, 48, 11, 0, 0,
            ],
            &[
                2, 2, 2, 6, 6, 12, 9, 9, 2, 6, 6, 12, 9, 9, 2, 6, 6, 12, 3, 3,
            ],
        ),
        2,
    );
}

#[test]
fn no_else_if_lookahead_is_consumed_once_inside_while() {
    let expected = successor_parser(WHILE_NO_ELSE, &module_caps());
    assert_eq!(expected.status, 0);
    assert_eq!(expected.nodes, WHILE_NO_ELSE_NODES);
    assert_future_graph(WHILE_NO_ELSE, WHILE_NO_ELSE_NODES, expected.origins, 2);
    assert_future_graph(
        WHILE_FINAL_NO_ELSE,
        WHILE_FINAL_NO_ELSE_NODES,
        origins(
            &[
                18, 22, 20, 20, 27, 36, 29, 29, 28, 24, 24, 24, 19, 12, 12, 47, 40, 40, 11, 0, 0,
            ],
            &[
                2, 2, 1, 1, 2, 2, 6, 6, 12, 7, 7, 7, 12, 9, 9, 2, 6, 6, 12, 3, 3,
            ],
        ),
        2,
    );
}

#[test]
fn arbitrary_match_native_arena_keeps_actual_constructor_names_and_anchors() {
    assert_future_graph(
        ARBITRARY_MATCH,
        MATCH_NODES,
        origins(
            &[
                7, 5, 42, 50, 54, 44, 61, 65, 67, 66, 56, 36, 36, 29, 29, 28, 0, 0,
            ],
            &[1, 1, 1, 1, 1, 1, 1, 2, 1, 21, 1, 1, 1, 6, 6, 12, 3, 3],
        ),
        3,
    );
}

#[test]
fn result_match_native_arena_matches_source_identical_rust_patterns() {
    assert_future_graph(
        RESULT_MATCH,
        MATCH_NODES,
        origins(
            &[
                7, 5, 42, 47, 51, 44, 57, 61, 63, 62, 53, 36, 36, 29, 29, 28, 0, 0,
            ],
            &[1, 1, 1, 1, 1, 1, 1, 2, 1, 21, 1, 1, 1, 6, 6, 12, 3, 3],
        ),
        3,
    );
}

#[test]
fn two_function_native_arena_keeps_parameter_chains_owned_separately() {
    assert_future_graph(
        TWO_FUNCTIONS,
        TWO_FUNCTION_NODES,
        origins(
            &[
                7, 5, 24, 17, 17, 16, 0, 0, 35, 33, 53, 51, 70, 63, 63, 62, 28, 28,
            ],
            &[1, 1, 2, 6, 6, 12, 3, 3, 1, 1, 1, 1, 2, 6, 6, 12, 3, 3],
        ),
        1,
    );
}

#[test]
fn compact_borrowed_call_native_arena_preserves_argument_order() {
    assert_future_graph(
        BORROWED_CALL,
        BORROWED_CALL_NODES,
        origins(
            &[22, 21, 29, 24, 31, 32, 32, 32, 20, 12, 0],
            &[1, 37, 1, 37, 2, 11, 11, 11, 10, 6, 3],
        ),
        4,
    );
}

#[test]
fn canonical_native_arena_represents_every_function_without_orphans() {
    let source = h1a_source();
    let expected = rust_projection(source.as_bytes());
    let expected_count = occurrence_count(&expected);
    // Independently derived first-use: parameter store first in result_value,
    // origins/nodes/values at its first arm, operators at `0 - code`, controls
    // in is_identifier_start, and calls in is_identifier_continue. No owner is
    // chosen by declarations, capacity or a search for expected contents.
    let observations: Vec<_> = ["-O0", "-O2"]
        .into_iter()
        .map(|optimization| {
            let capture = capture(source.as_bytes(), optimization);
            capture.validate_trace(&[5, 8, 7, 10, 9, 4, 6, 2, 3, 1]);
            let actual_nodes = capture.records::<4>(6);
            let actual_origins = capture.records::<5>(5);
            if actual_nodes.len() != expected_count {
                for (node, origin) in actual_nodes.iter().zip(&actual_origins).rev().take(8) {
                    eprintln!("native canonical tail {optimization}: {node:?} {origin:?}");
                }
            }
            (optimization, actual_nodes, actual_origins)
        })
        .collect();
    let differences: Vec<_> = observations
        .iter()
        .filter(|(_, nodes, origins)| {
            nodes.len() != expected_count || origins.len() != expected_count
        })
        .map(|(optimization, nodes, origins)| (*optimization, nodes.len(), origins.len()))
        .collect();
    assert!(
        differences.is_empty(),
        "native canonical occurrence census differs from pre-derived Rust AST: expected {expected_count}, actual completed O0/O2 {differences:?}"
    );
    for (_, nodes, origins) in &observations {
        validate_origins(source.as_bytes(), nodes, origins);
        let actual = graph_projection(source.as_bytes(), nodes);
        assert_eq!(
            actual, expected,
            "every function and the complete module have source-identical normalized structure"
        );
    }
}

#[test]
fn hand_graphs_match_independent_rust_ast_and_occurrence_counts() {
    for (source, nodes) in [
        (COMPACT, COMPACT_NODES),
        (PARAMETERS, PARAMETER_NODES),
        (MUTATION, MUTATION_NODES),
        (NESTED_IF, NESTED_IF_NODES),
        (ELSE_IF, ELSE_IF_NODES),
        (NESTED_WHILE, NESTED_WHILE_NODES),
        (RESULT_MATCH, MATCH_NODES),
        (TWO_FUNCTIONS, TWO_FUNCTION_NODES),
        (BORROWED_CALL, BORROWED_CALL_NODES),
    ] {
        let projected = rust_projection(source);
        assert_eq!(graph_projection(source, nodes), projected);
        assert_eq!(occurrence_count(&projected), nodes.len());
    }
    let mut wrong_parameters = PARAMETER_NODES.to_vec();
    wrong_parameters[1][1] = 4;
    wrong_parameters[3][1] = 2;
    assert_ne!(
        graph_projection(PARAMETERS, &wrong_parameters),
        rust_projection(PARAMETERS),
        "same-size parameter ownership swap must be observable"
    );
    let mut wrong_target = MUTATION_NODES.to_vec();
    wrong_target[5][1] = 1;
    assert_ne!(
        graph_projection(MUTATION, &wrong_target),
        rust_projection(MUTATION),
        "same-size assignment target substitution must be observable"
    );
    let mut wrong_branches = NESTED_IF_NODES.to_vec();
    wrong_branches[10].swap(2, 3);
    assert_ne!(
        graph_projection(NESTED_IF, &wrong_branches),
        rust_projection(NESTED_IF),
        "same-size branch swap must be observable"
    );
    let mut wrong_binders = MATCH_NODES.to_vec();
    wrong_binders[3][1] = 9;
    wrong_binders[6][1] = 7;
    assert_ne!(
        graph_projection(RESULT_MATCH, &wrong_binders),
        rust_projection(RESULT_MATCH),
        "same-size binder swaps must be observable"
    );
    let tokens =
        compiler::try_tokenize_with_locations(std::str::from_utf8(ARBITRARY_MATCH).unwrap(), None)
            .unwrap();
    assert!(
        compiler::parse_with_locations(tokens).is_err(),
        "arbitrary constructors must not be passed off as Rust's common grammar"
    );
}

#[test]
fn poison_tail_recovers_logical_records_and_rejects_partial_writes() {
    let mut bytes = vec![0xa5; 64];
    let encoded: Vec<u8> = COMPACT_NODES
        .iter()
        .flatten()
        .flat_map(|word| word.to_le_bytes())
        .collect();
    bytes[..encoded.len()].copy_from_slice(&encoded);
    let capture = Capture {
        result: 80,
        counters: [0; 5],
        events: vec![],
        owners: vec![Owner {
            id: 1,
            bytes: bytes.clone(),
        }],
    };
    assert_eq!(capture.records::<4>(1), COMPACT_NODES);
    bytes[48] = 0;
    let partial = Capture {
        result: 80,
        counters: [0; 5],
        events: vec![],
        owners: vec![Owner { id: 1, bytes }],
    };
    assert!(
        std::panic::catch_unwind(|| partial.records::<4>(1)).is_err(),
        "one written tail byte must not disappear into capacity"
    );
    let mut bytes = vec![0xa5; 64];
    bytes[..12].fill(0);
    let partial = Capture {
        result: 80,
        counters: [0; 5],
        events: vec![],
        owners: vec![Owner { id: 1, bytes }],
    };
    assert!(
        std::panic::catch_unwind(|| partial.records::<4>(1)).is_err(),
        "three complete words are not one complete node"
    );
}

#[test]
fn canonical_occurrence_budget_is_derived_from_rust_structure() {
    let source = h1a_source();
    let projected = rust_projection(source.as_bytes());
    let count = occurrence_count(&projected);
    assert!(
        count <= 65_536,
        "frozen representation exceeds its existing node budget: {count}"
    );
    println!(
        "independently derived canonical connected AST: {} functions, {count} nodes/origins",
        projected.len()
    );
    let ingestion = module_ingest(source.as_bytes());
    assert!(source.len() <= H1A_SOURCE_BOUND);
    assert!(ingestion.tokens.len() <= H1A_TOKEN_BOUND);
    assert!(ingestion.names.len() <= H1A_NAME_BOUND);
    println!(
        "source bytes={} md5={:x} token_count={} name_count={}",
        source.len(),
        md5::compute(source.as_bytes()),
        ingestion.tokens.len(),
        ingestion.names.len()
    );
    for function in &projected {
        let (physical_controls, max_depth) = control_budget(&function.body, 0);
        assert!(physical_controls <= 65_536);
        println!(
            "{} params={} representation_nodes={} physical_controls={} max_control_depth={}",
            function.name,
            function.parameters.len(),
            occurrence_count(std::slice::from_ref(function)),
            physical_controls,
            max_depth
        );
    }
    let physical_controls: usize = projected
        .iter()
        .map(|function| control_budget(&function.body, 0).0)
        .sum();
    assert!(physical_controls <= 65_536);
    println!(
        "derived physical continuation records={physical_controls}; no structural-count claim is made for other operational stores"
    );
    // These expression work stores retain their previous push/pop algorithms.
    // Structural appends (including Match's subject/binder/arm/wrapper records)
    // do not push values, operators or calls. Derive those costs separately
    // with the preserved predecessor expression model on these exact bytes.
    let operational =
        oracle::binding_parser_stop(&ingestion, source.as_bytes(), &oracle::Caps::UNBOUNDED);
    assert_eq!(operational.status, 0, "unchanged admitted grammar");
    let (values, operators, _historical_blocks, calls) = operational.counts;
    for (label, records) in [
        ("values", values),
        ("operators", operators),
        ("calls", calls),
    ] {
        assert!(
            records <= 65_536,
            "{label} exceeds unchanged record ceiling"
        );
    }
    println!(
        "separately derived expression work stores: values={values} operators={operators} calls={calls} parameters={}",
        operational.parameters.len()
    );
    let brace_pairs = ingestion
        .tokens
        .iter()
        .filter(|token| token[0] == 12)
        .count();
    let connected = successor_parser(source.as_bytes(), &module_caps());
    assert_eq!(connected.status, 0);
    let closure_nodes = connected
        .nodes
        .iter()
        .filter(|node| {
            matches!(node[0], 30 | 37)
                || (node[0] == 19 && connected.nodes[(node[2] - 1) as usize][0] == 18)
        })
        .count();
    assert_eq!(
        brace_pairs, closure_nodes,
        "one lexical brace pair per closing occurrence"
    );
    assert!(brace_pairs <= count && brace_pairs <= 65_536);
    println!(
        "independent auth costs: starts_words={count} starts_bytes={} end_words={count} end_bytes={} brace_history_records={brace_pairs} brace_history_bytes={} range_records={count} range_bytes={}",
        count * 4,
        count * 4,
        brace_pairs * 8,
        count * 8
    );
}

#[test]
fn successor_model_preserves_hand_graphs_and_legacy_grammar_priority() {
    let fixtures = [
        (COMPACT, COMPACT_NODES),
        (PARAMETERS, PARAMETER_NODES),
        (MUTATION, MUTATION_NODES),
        (NESTED_IF, NESTED_IF_NODES),
        (ELSE_IF, ELSE_IF_NODES),
        (NESTED_WHILE, NESTED_WHILE_NODES),
        (ARBITRARY_MATCH, MATCH_NODES),
        (RESULT_MATCH, MATCH_NODES),
        (TWO_FUNCTIONS, TWO_FUNCTION_NODES),
        (BORROWED_CALL, BORROWED_CALL_NODES),
    ];
    for (source, nodes) in fixtures {
        let successor = successor_parser(source, &module_caps());
        assert_eq!(successor.status, 0, "{}", String::from_utf8_lossy(source));
        assert_eq!(successor.nodes, nodes);
        validate_origins(source, &successor.nodes, &successor.origins);
    }
    for source in [
        &b"fn f(a:int,b:Result<int,int>)->int{return 1;} x"[..],
        &b"fn f(a:int,b:Result<int,int>)->int{return 1;"[..],
        &b"fn f()->int{let x:Result<int,int>=1;x=2;return 3;} x"[..],
        &b"fn f()->int{if 1{return 2;}else if 3{return 4;}return 5;} x"[..],
        &b"fn f()->int{return match x{Alpha(y)=>1,Beta(z)=>2,};} x"[..],
        &b"fn f()->int{return 1;return 2;}"[..],
        &b"fn f(a:ByteBuffer)->int{return 1;}"[..],
        &b"fn f()->int{while 1{}return 2;}"[..],
    ] {
        let old = oracle::binding_parser_stop(&module_ingest(source), source, &module_caps());
        let successor = successor_parser(source, &module_caps());
        assert_eq!(
            (
                successor.status,
                successor.error_offset,
                successor.error_line,
                successor.error_column,
                successor.diagnostic_code,
                successor.diagnostic_actual
            ),
            (
                old.status,
                old.error_offset,
                old.error_line,
                old.error_column,
                old.diagnostic_code,
                old.diagnostic_actual
            ),
            "{}",
            String::from_utf8_lossy(source)
        );
        assert_eq!(successor.parameters, old.parameters);
        assert_eq!(
            (successor.counts.0, successor.counts.1, successor.counts.3),
            (old.counts.0, old.counts.1, old.counts.3)
        );
    }
}

#[test]
fn successor_canonical_model_derives_complete_append_order_and_origins() {
    let canonical = h1a_source();
    let model = successor_parser(canonical.as_bytes(), &module_caps());
    assert_eq!(model.status, 0);
    assert_eq!(
        model.nodes.len(),
        occurrence_count(&rust_projection(canonical.as_bytes()))
    );
    validate_origins(canonical.as_bytes(), &model.nodes, &model.origins);
    for index in 3775..3785 {
        let origin = model.origins[index];
        println!(
            "independent canonical prefix node{} {:?} {:?}: {}",
            index + 1,
            model.nodes[index],
            origin,
            &canonical[origin[1] as usize..(origin[1] as usize + 80).min(canonical.len())]
        );
    }
}

/// The predecessor's 6.5-token-per-block proof applies only to its discarded
/// three-word work records. Successor control records instead inject into
/// distinct structural occurrences: initial -> condition expression root,
/// await-else -> completed then Block30, optional else -> that Block's final
/// Cell29. These roles are disjoint, even before the parser finishes its item;
/// an allocation/capacity failure can only prevent a later append. Therefore
/// physical control records <= appended nodes, with the node guard preceding
/// the corresponding control append. No token/record floor is borrowed from
/// the historical representation and no existing ceiling is widened.
#[test]
fn successor_control_cost_uses_physical_replacements_not_historical_blocks() {
    let mut source = b"fn f() -> int { ".to_vec();
    for _ in 0..650 {
        source.extend_from_slice(b"if 1 { a = 1; } else { a = 1; } ");
    }
    source.extend_from_slice(b"return 1; } x");
    let historical = oracle::module_parser_stop(&module_ingest(&source), &source, &module_caps());
    assert_eq!(historical.counts.2, 1300, "unchanged historical model");
    let successor = successor_parser(&source, &module_caps());
    assert_eq!(successor.status, 10);
    assert_eq!(
        successor.counts.2,
        650 * 3,
        "initial/await-else/else replacement"
    );
    assert!(successor.counts.2 <= successor.nodes.len());
    let exact = oracle::Caps {
        blocks: 1950,
        ..oracle::Caps::UNBOUNDED
    };
    let short = oracle::Caps {
        blocks: 1949,
        ..oracle::Caps::UNBOUNDED
    };
    assert_eq!(successor_parser(&source, &exact).status, 10);
    let stopped = successor_parser(&source, &short);
    assert_eq!((stopped.status, stopped.diagnostic_code), (15, 1949));
    // Literal-rich grammar still receives a full independent successor vector
    // at the existing block-storage native call site, not parser-only80.
}

#[test]
fn frozen_origins_authenticate_real_roles_and_reject_same_kind_anchor_swaps() {
    let fixture_origins = vec![
        (COMPACT, COMPACT_NODES, origins(&[19, 12, 0], &[2, 6, 3])),
        (
            PARAMETERS,
            PARAMETER_NODES,
            origins(
                &[7, 5, 13, 11, 42, 35, 35, 34, 0, 0],
                &[1, 1, 1, 1, 2, 6, 6, 12, 3, 3],
            ),
        ),
        (
            MUTATION,
            MUTATION_NODES,
            origins(
                &[22, 26, 12, 12, 30, 28, 28, 39, 32, 32, 11, 0, 0],
                &[1, 2, 4, 4, 2, 1, 1, 2, 6, 6, 12, 3, 3],
            ),
        ),
        (NESTED_IF, NESTED_IF_NODES, nested_if_origins()),
        (
            ELSE_IF,
            ELSE_IF_NODES,
            origins(
                &[
                    15, 24, 17, 17, 16, 35, 43, 47, 37, 37, 56, 49, 49, 36, 71, 64, 64, 63, 32, 32,
                    12, 12, 12, 81, 74, 74, 11, 0, 0,
                ],
                &[
                    2, 2, 6, 6, 12, 2, 1, 2, 4, 4, 2, 6, 6, 12, 2, 6, 6, 12, 7, 7, 7, 7, 7, 2, 6,
                    6, 12, 3, 3,
                ],
            ),
        ),
        (
            NESTED_WHILE,
            NESTED_WHILE_NODES,
            origins(
                &[
                    18, 26, 35, 28, 28, 27, 20, 20, 45, 38, 38, 19, 12, 12, 55, 48, 48, 11, 0, 0,
                ],
                &[
                    2, 2, 2, 6, 6, 12, 9, 9, 2, 6, 6, 12, 9, 9, 2, 6, 6, 12, 3, 3,
                ],
            ),
        ),
        (
            ARBITRARY_MATCH,
            MATCH_NODES,
            origins(
                &[
                    7, 5, 42, 50, 54, 44, 61, 65, 67, 66, 56, 36, 36, 29, 29, 28, 0, 0,
                ],
                &[1, 1, 1, 1, 1, 1, 1, 2, 1, 21, 1, 1, 1, 6, 6, 12, 3, 3],
            ),
        ),
        (
            RESULT_MATCH,
            MATCH_NODES,
            origins(
                &[
                    7, 5, 42, 47, 51, 44, 57, 61, 63, 62, 53, 36, 36, 29, 29, 28, 0, 0,
                ],
                &[1, 1, 1, 1, 1, 1, 1, 2, 1, 21, 1, 1, 1, 6, 6, 12, 3, 3],
            ),
        ),
        (
            TWO_FUNCTIONS,
            TWO_FUNCTION_NODES,
            origins(
                &[
                    7, 5, 24, 17, 17, 16, 0, 0, 35, 33, 53, 51, 70, 63, 63, 62, 28, 28,
                ],
                &[1, 1, 2, 6, 6, 12, 3, 3, 1, 1, 1, 1, 2, 6, 6, 12, 3, 3],
            ),
        ),
        (
            BORROWED_CALL,
            BORROWED_CALL_NODES,
            origins(
                &[22, 21, 29, 24, 31, 32, 32, 32, 20, 12, 0],
                &[1, 37, 1, 37, 2, 11, 11, 11, 10, 6, 3],
            ),
        ),
    ];
    for (source, nodes, origin_rows) in &fixture_origins {
        validate_origins(source, nodes, origin_rows);
    }
    // All substituted anchors are still real tokens of the same kind. Their
    // source roles/owning nodes differ, which a token-kind-only check misses.
    for (fixture, target, replacement) in [(3, 10, 18), (3, 4, 8), (8, 16, 6), (7, 6, 8), (7, 5, 3)]
    {
        let (source, nodes, rows) = &fixture_origins[fixture];
        let mut corrupted = rows.clone();
        let origin = rows[replacement];
        corrupted[target][1..].copy_from_slice(&origin[1..]);
        assert!(std::panic::catch_unwind(|| validate_origins(source, nodes, &corrupted)).is_err());
    }
    let mut wrong_parameter_type = PARAMETER_NODES.to_vec();
    wrong_parameter_type[0][1] = 3;
    assert!(
        std::panic::catch_unwind(|| validate_shapes(&wrong_parameter_type, 5)).is_err(),
        "ByteBuffer must not become an admitted parameter annotation"
    );
    let mut wrong_child = MUTATION_NODES.to_vec();
    wrong_child[2][0] = 35;
    assert!(
        std::panic::catch_unwind(|| validate_shapes(&wrong_child, 3)).is_err(),
        "legal backward edges do not make a let annotation a pattern binder"
    );
    let identical = b"fn f(a:int)->int{return 1;} fn g(a:int)->int{return 2;}";
    let identical_nodes = &[
        [24, 1, 0, 0],
        [25, 2, 1, 0],
        [1, 1, 0, 0],
        [18, 0, 3, 0],
        [29, 0, 0, 4],
        [30, 0, 5, 0],
        [38, 1, 2, 6],
        [19, 1, 7, 0],
        [24, 1, 0, 0],
        [25, 2, 9, 0],
        [1, 2, 0, 0],
        [18, 0, 11, 0],
        [29, 0, 0, 12],
        [30, 0, 13, 0],
        [38, 1, 10, 14],
        [19, 4, 15, 8],
    ];
    let identical_origins = origins(
        &[7, 5, 24, 17, 17, 16, 0, 0, 35, 33, 52, 45, 45, 44, 28, 28],
        &[1, 1, 2, 6, 6, 12, 3, 3, 1, 1, 2, 6, 6, 12, 3, 3],
    );
    validate_origins(identical, identical_nodes, &identical_origins);
    let mut wrong_occurrence = identical_origins.clone();
    wrong_occurrence[8][1..].copy_from_slice(&identical_origins[0][1..]);
    wrong_occurrence[9][1..].copy_from_slice(&identical_origins[1][1..]);
    assert!(
        std::panic::catch_unwind(|| validate_origins(
            identical,
            identical_nodes,
            &wrong_occurrence
        ))
        .is_err(),
        "same name/type adjacency in the wrong function header must fail occurrence ownership"
    );
}

#[test]
fn forwarding_observer_preserves_reallocation_failure_and_poisoned_tail() {
    let runtime = repository_path("../../src/compiler/runtime/aero_test_runtime.c");
    let observer = OBSERVER.replace(
        "RUNTIME_PATH",
        &runtime.to_string_lossy().replace('\\', "/"),
    );
    let main = r#"
int main(int argc, char **argv) {
    if (argc != 2 || aero_test_reset(2) != 1) return 70;
    unsigned char *owner = (unsigned char *)aero_alloc(8);
    if (owner == NULL) return 71;
    for (int i = 0; i < 8; i++) if (owner[i] != 0xa5) return 72;
    owner[0] = 7; owner[1] = 0; owner[2] = 0; owner[3] = 0;
    unsigned char before[16]; memcpy(before, owner, 8);
    owner = (unsigned char *)aero_realloc(owner, 8, 16);
    if (owner == NULL || memcmp(before, owner, 8) != 0) return 73;
    for (int i = 8; i < 16; i++) if (owner[i] != 0xa5) return 74;
    if (ast_owners[0].pointer != owner || ast_owners[0].size != 16 || !ast_owners[0].live) return 75;
    memcpy(before, owner, 16);
    if (aero_realloc(owner, 16, 32) != NULL) return 76;
    if (memcmp(before, owner, 16) != 0 || ast_owners[0].pointer != owner || ast_owners[0].size != 16) return 77;
    if (aero_alloc(8) != NULL || aero_test_live_allocations() != 1) return 78;
    aero_dealloc(owner, 16);
    if (aero_test_alloc_calls() != 2 || aero_test_realloc_calls() != 2 || aero_test_dealloc_calls() != 1) return 79;
    if (aero_test_live_allocations() != 0 || aero_test_size_mismatch_calls() != 0) return 81;
    if (!ast_write_capture(argv[1], 80)) return 82;
    return 91;
}
"#;
    for optimization in ["-O0", "-O2"] {
        let workspace = TestWorkspace::new("connected-ast-observer-failure");
        let harness = workspace.write("forwarding.c", format!("{observer}\n{main}"));
        let executable = clang_link(&workspace, "forwarding", optimization, &[harness.as_path()]);
        let artifact = workspace.root.join("allocator-control.bin");
        let output = Command::new(executable).arg(&artifact).output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(91),
            "observer calibration {optimization}: {output:?}"
        );
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
        let captured = Capture::read(&fs::read(artifact).unwrap());
        assert_eq!(captured.counters, [2, 2, 1, 0, 0]);
        assert_eq!(captured.owners.len(), 1);
        assert_eq!(captured.owner(1).len(), 16);
        assert_eq!(captured.records::<1>(1), vec![[7]]);
        assert_eq!(
            captured
                .events
                .iter()
                .map(|event| (event.kind, event.owner, event.succeeded))
                .collect::<Vec<_>>(),
            vec![
                (1, 1, true),
                (2, 1, true),
                (2, 1, false),
                (1, 0, false),
                (3, 1, true)
            ]
        );
    }
}
