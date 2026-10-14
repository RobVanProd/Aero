//! H1C-SCALAR-001: an independent recursive lexical-scope authority.
//! The product uses flat bounded append-only stores; this model deliberately
//! walks ownership edges recursively and uses ordinary Rust lexical maps.

use super::*;
use std::collections::BTreeMap;

const SCOPED_PRELUDE: &str = r#"
    // H1C-SCALAR-001: closed rich subset, then lexical names, then exact facts.
    // Compact modules retain the complete predecessor route below.
    let mut scoped_contexts: ByteBuffer = bytes_new();
    let mut scoped_bindings: ByteBuffer = bytes_new();
    let mut scoped_rich: int = 0;
    let mut scoped_index: int = 0;
    let mut scoped_kind: int = 0;
    let mut scoped_payload: int = 0;
    let mut scoped_left: int = 0;
    let mut scoped_right: int = 0;
    let mut scoped_function: int = 0;
    let mut scoped_scope: int = 0;
    let mut scoped_outer_scope: int = 0;
    let mut scoped_parent_scope: int = 0;
    let mut scoped_scope_start: int = 0;
    let mut scoped_pending_assignment: int = 0;
    let mut scoped_pending_start: int = 0;
    let mut scoped_event: int = 0;
    let mut scoped_field: int = 0;
    let mut scoped_word: int = 0;
    let mut scoped_byte: int = 0;
    let mut scoped_value: int = 0;
    let mut scoped_binding_count: int = 0;
    let mut scoped_binding_index: int = 0;
    let mut scoped_binding_name: int = 0;
    let mut scoped_binding_function: int = 0;
    let mut scoped_binding_scope: int = 0;
    let mut scoped_binding_node: int = 0;
    let mut scoped_binding_mutable: int = 0;
    let mut scoped_search_scope: int = 0;
    let mut scoped_search_name: int = 0;
    let mut scoped_use_node: int = 0;
    let mut scoped_current_only: int = 0;
    let mut scoped_found: int = 0;
    let mut scoped_action: int = 0;
    let mut scoped_target_node: int = 0;
    let mut scoped_target_kind: int = 0;
    let mut scoped_target_name: int = 0;
    let mut scoped_symbol_index: int = 0;
    let mut scoped_previous_symbol: int = 0;
    while status == 0 && semantic_status == 0 && scoped_index < node_count {
        word_offset = scoped_index * 16;
        scoped_kind = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        if scoped_kind >= 24 && scoped_kind <= 38 {
            scoped_rich = 1;
        }
        scoped_index = scoped_index + 1;
    }
    scoped_index = 0;
    while status == 0 && semantic_status == 0 && scoped_rich == 1
        && scoped_index < node_count {
        word_offset = scoped_index * 16;
        scoped_kind = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = scoped_index * 16 + 4;
        scoped_payload = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        if (scoped_kind >= 20 && scoped_kind <= 23)
            || (scoped_kind >= 34 && scoped_kind <= 37)
            || (scoped_kind == 24 && scoped_payload != 1) {
            semantic_status = 28;
            semantic_code = scoped_kind;
            semantic_node = scoped_index + 1;
            word_offset = (semantic_node - 1) * 20 + 4;
            semantic_offset = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            word_offset = (semantic_node - 1) * 20 + 8;
            semantic_line = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            word_offset = (semantic_node - 1) * 20 + 12;
            semantic_column = result_value(bytes_get(&origins, word_offset))
                + result_value(bytes_get(&origins, word_offset + 1)) * 256
                + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        }
        scoped_index = scoped_index + 1;
    }
    // Rich-route function namespace duplicates precede local-name analysis.
    while status == 0 && semantic_status == 0 && scoped_rich == 1
        && scoped_symbol_index < symbol_count {
        word_offset = scoped_symbol_index * 16 + 4;
        scoped_search_name = result_value(bytes_get(&symbols, word_offset))
            + result_value(bytes_get(&symbols, word_offset + 1)) * 256
            + result_value(bytes_get(&symbols, word_offset + 2)) * 65536
            + result_value(bytes_get(&symbols, word_offset + 3)) * 16777216;
        scoped_previous_symbol = 0;
        while semantic_status == 0 && scoped_previous_symbol < scoped_symbol_index {
            word_offset = scoped_previous_symbol * 16 + 4;
            scoped_binding_name = result_value(bytes_get(&symbols, word_offset))
                + result_value(bytes_get(&symbols, word_offset + 1)) * 256
                + result_value(bytes_get(&symbols, word_offset + 2)) * 65536
                + result_value(bytes_get(&symbols, word_offset + 3)) * 16777216;
            if scoped_binding_name == scoped_search_name {
                semantic_status = 30;
                semantic_code = 1;
                word_offset = scoped_symbol_index * 16 + 8;
                scoped_target_node = result_value(bytes_get(&symbols, word_offset))
                    + result_value(bytes_get(&symbols, word_offset + 1)) * 256
                    + result_value(bytes_get(&symbols, word_offset + 2)) * 65536
                    + result_value(bytes_get(&symbols, word_offset + 3)) * 16777216;
                semantic_node = scoped_target_node;
                word_offset = (semantic_node - 1) * 20 + 4;
                semantic_offset = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
                word_offset = (semantic_node - 1) * 20 + 8;
                semantic_line = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
                word_offset = (semantic_node - 1) * 20 + 12;
                semantic_column = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
            }
            scoped_previous_symbol = scoped_previous_symbol + 1;
        }
        scoped_symbol_index = scoped_symbol_index + 1;
    }
    // Descending postorder appends contexts at direct index N-node. A nested
    // Block records its parent before its body becomes active. Exact interval
    // starts restore every closed scope; the outer self-parent retains params.
    scoped_index = node_count - 1;
    while status == 0 && semantic_status == 0 && scoped_rich == 1 && scoped_index >= 0 {
        word_offset = (scoped_index + 1 - 1) * 16;
        scoped_kind = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (scoped_index + 1 - 1) * 16 + 4;
        scoped_payload = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (scoped_index + 1 - 1) * 16 + 8;
        scoped_left = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (scoped_index + 1 - 1) * 16 + 12;
        scoped_right = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        scoped_parent_scope = scoped_scope;
        while scoped_scope > 0 && scoped_scope != scoped_outer_scope
            && scoped_parent_scope > 0 {
            word_offset = (scoped_scope - 1) * 4;
            scoped_scope_start = result_value(bytes_get(&ast_starts, word_offset))
                + result_value(bytes_get(&ast_starts, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_starts, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_starts, word_offset + 3)) * 16777216;
            if scoped_index + 1 < scoped_scope_start {
                word_offset = (node_count - scoped_scope) * 12 + 4;
                scoped_parent_scope = result_value(bytes_get(&scoped_contexts, word_offset))
                    + result_value(bytes_get(&scoped_contexts, word_offset + 1)) * 256
                    + result_value(bytes_get(&scoped_contexts, word_offset + 2)) * 65536
                    + result_value(bytes_get(&scoped_contexts, word_offset + 3)) * 16777216;
                scoped_scope = scoped_parent_scope;
            } else {
                scoped_parent_scope = 0;
            }
        }
        if scoped_kind == 19 {
            scoped_function = scoped_index + 1;
            scoped_scope = 0;
            scoped_outer_scope = 0;
            scoped_pending_assignment = 0;
            word_offset = (scoped_left - 1) * 16;
            scoped_field = result_value(bytes_get(&nodes, word_offset))
                + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
            if scoped_field == 38 {
                word_offset = (scoped_left - 1) * 16 + 12;
                scoped_scope = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
                scoped_outer_scope = scoped_scope;
            }
        }
        if scoped_kind == 28 {
            scoped_pending_assignment = scoped_index + 1;
            word_offset = scoped_index * 4;
            scoped_pending_start = result_value(bytes_get(&ast_starts, word_offset))
                + result_value(bytes_get(&ast_starts, word_offset + 1)) * 256
                + result_value(bytes_get(&ast_starts, word_offset + 2)) * 65536
                + result_value(bytes_get(&ast_starts, word_offset + 3)) * 16777216;
        }
        scoped_event = 0;
        if scoped_pending_assignment > 0 && scoped_index + 1 == scoped_pending_start {
            scoped_event = scoped_pending_assignment;
            scoped_pending_assignment = 0;
        }
        semantic_node = scoped_index + 1;
        word_offset = (semantic_node - 1) * 20 + 4;
        semantic_offset = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (semantic_node - 1) * 20 + 8;
        semantic_line = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (semantic_node - 1) * 20 + 12;
        semantic_column = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        scoped_field = 0;
        while semantic_status == 0 && scoped_field < 3 {
            scoped_word = scoped_function;
            if scoped_field == 1 {
                scoped_word = scoped_scope;
            }             else if scoped_field == 2 {
                scoped_word = scoped_event;
            }
            scoped_byte = 0;
            while semantic_status == 0 && scoped_byte < 4 {
                scoped_value = word_byte_0(scoped_word);
                if scoped_byte == 1 {
                    scoped_value = word_byte_1(scoped_word);
                } else if scoped_byte == 2 {
                    scoped_value = word_byte_2(scoped_word);
                } else if scoped_byte == 3 {
                    scoped_value = word_byte_3(scoped_word);
                }
                push_result = result_value(bytes_push(&mut scoped_contexts, scoped_value));
                if push_result < 0 {
                    semantic_status = 26;
                    semantic_code = 8;
                }
                scoped_byte = scoped_byte + 1;
            }
            scoped_field = scoped_field + 1;
        }
        if scoped_kind == 30 {
            scoped_scope = scoped_index + 1;
        }
        scoped_index = scoped_index - 1;
    }
    if status == 0 && semantic_status == 0 && scoped_rich == 1
        && bytes_len(&scoped_contexts) != node_count * 12 {
        semantic_status = 27;
        semantic_code = 4;
        semantic_node = root;
        word_offset = (semantic_node - 1) * 20 + 4;
        semantic_offset = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (semantic_node - 1) * 20 + 8;
        semantic_line = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (semantic_node - 1) * 20 + 12;
        semantic_column = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
    }
    // Names-only pass: no facts exist until the complete name environment is
    // checked. Assignment's target event precedes its RHS; Let activates only
    // after its initializer. The table is immutable and append-only.
    scoped_index = 0;
    while status == 0 && semantic_status == 0 && scoped_rich == 1 && scoped_index < node_count {
        word_offset = (scoped_index + 1 - 1) * 16;
        scoped_kind = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (scoped_index + 1 - 1) * 16 + 4;
        scoped_payload = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (scoped_index + 1 - 1) * 16 + 8;
        scoped_left = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (scoped_index + 1 - 1) * 16 + 12;
        scoped_right = result_value(bytes_get(&nodes, word_offset))
            + result_value(bytes_get(&nodes, word_offset + 1)) * 256
            + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
            + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
        word_offset = (node_count - scoped_index - 1) * 12;
        scoped_function = result_value(bytes_get(&scoped_contexts, word_offset))
            + result_value(bytes_get(&scoped_contexts, word_offset + 1)) * 256
            + result_value(bytes_get(&scoped_contexts, word_offset + 2)) * 65536
            + result_value(bytes_get(&scoped_contexts, word_offset + 3)) * 16777216;
        word_offset = (node_count - scoped_index - 1) * 12 + 4;
        scoped_scope = result_value(bytes_get(&scoped_contexts, word_offset))
            + result_value(bytes_get(&scoped_contexts, word_offset + 1)) * 256
            + result_value(bytes_get(&scoped_contexts, word_offset + 2)) * 65536
            + result_value(bytes_get(&scoped_contexts, word_offset + 3)) * 16777216;
        word_offset = (node_count - scoped_index - 1) * 12 + 8;
        scoped_event = result_value(bytes_get(&scoped_contexts, word_offset))
            + result_value(bytes_get(&scoped_contexts, word_offset + 1)) * 256
            + result_value(bytes_get(&scoped_contexts, word_offset + 2)) * 65536
            + result_value(bytes_get(&scoped_contexts, word_offset + 3)) * 16777216;
        scoped_action = 0;
        while semantic_status == 0 && scoped_action < 2 {
            scoped_target_node = 0;
            scoped_target_kind = 0;
            scoped_target_name = 0;
            if scoped_action == 0 && scoped_event > 0 {
                scoped_target_node = scoped_event;
                scoped_target_kind = 28;
                word_offset = (scoped_event - 1) * 16 + 4;
                scoped_target_name = result_value(bytes_get(&nodes, word_offset))
                    + result_value(bytes_get(&nodes, word_offset + 1)) * 256
                    + result_value(bytes_get(&nodes, word_offset + 2)) * 65536
                    + result_value(bytes_get(&nodes, word_offset + 3)) * 16777216;
            } else if scoped_action == 1 && (scoped_kind == 2
                || scoped_kind == 25 || scoped_kind == 26 || scoped_kind == 27) {
                scoped_target_node = scoped_index + 1;
                scoped_target_kind = scoped_kind;
                scoped_target_name = scoped_payload;
            }
            if scoped_target_node > 0 {
                scoped_search_name = scoped_target_name;
                scoped_use_node = scoped_target_node;
                scoped_current_only = 0;
                if scoped_target_kind >= 25 && scoped_target_kind <= 27 {
                    scoped_current_only = 1;
                }
                scoped_found = 0;
                scoped_search_scope = scoped_scope;
                while scoped_found == 0 && scoped_search_scope > 0 {
                    scoped_binding_index = scoped_binding_count - 1;
                    while scoped_found == 0 && scoped_binding_index >= 0 {
                        word_offset = scoped_binding_index * 20;
                        scoped_binding_name = result_value(bytes_get(&scoped_bindings, word_offset))
                            + result_value(bytes_get(&scoped_bindings, word_offset + 1)) * 256
                            + result_value(bytes_get(&scoped_bindings, word_offset + 2)) * 65536
                            + result_value(bytes_get(&scoped_bindings, word_offset + 3)) * 16777216;
                        word_offset = scoped_binding_index * 20 + 4;
                        scoped_binding_function = result_value(bytes_get(&scoped_bindings, word_offset))
                            + result_value(bytes_get(&scoped_bindings, word_offset + 1)) * 256
                            + result_value(bytes_get(&scoped_bindings, word_offset + 2)) * 65536
                            + result_value(bytes_get(&scoped_bindings, word_offset + 3)) * 16777216;
                        word_offset = scoped_binding_index * 20 + 8;
                        scoped_binding_scope = result_value(bytes_get(&scoped_bindings, word_offset))
                            + result_value(bytes_get(&scoped_bindings, word_offset + 1)) * 256
                            + result_value(bytes_get(&scoped_bindings, word_offset + 2)) * 65536
                            + result_value(bytes_get(&scoped_bindings, word_offset + 3)) * 16777216;
                        word_offset = scoped_binding_index * 20 + 12;
                        scoped_binding_node = result_value(bytes_get(&scoped_bindings, word_offset))
                            + result_value(bytes_get(&scoped_bindings, word_offset + 1)) * 256
                            + result_value(bytes_get(&scoped_bindings, word_offset + 2)) * 65536
                            + result_value(bytes_get(&scoped_bindings, word_offset + 3)) * 16777216;
                        word_offset = scoped_binding_index * 20 + 16;
                        scoped_binding_mutable = result_value(bytes_get(&scoped_bindings, word_offset))
                            + result_value(bytes_get(&scoped_bindings, word_offset + 1)) * 256
                            + result_value(bytes_get(&scoped_bindings, word_offset + 2)) * 65536
                            + result_value(bytes_get(&scoped_bindings, word_offset + 3)) * 16777216;
                        if scoped_binding_name == scoped_search_name
                            && scoped_binding_function == scoped_function
                            && scoped_binding_scope == scoped_search_scope
                            && scoped_binding_node < scoped_use_node {
                            scoped_found = 1;
                        }
                        scoped_binding_index = scoped_binding_index - 1;
                    }
                    if scoped_found == 0 {
                        if scoped_current_only == 1 {
                            scoped_search_scope = 0;
                        } else {
                            word_offset = (node_count - scoped_search_scope) * 12 + 4;
                            scoped_parent_scope = result_value(bytes_get(&scoped_contexts, word_offset))
                                + result_value(bytes_get(&scoped_contexts, word_offset + 1)) * 256
                                + result_value(bytes_get(&scoped_contexts, word_offset + 2)) * 65536
                                + result_value(bytes_get(&scoped_contexts, word_offset + 3)) * 16777216;
                            if scoped_parent_scope == scoped_search_scope {
                                scoped_search_scope = 0;
                            } else {
                                scoped_search_scope = scoped_parent_scope;
                            }
                        }
                    }
                }
                if scoped_target_kind == 2 && scoped_found == 0 {
                    semantic_status = 17;
                    semantic_code = 2;
                } else if scoped_target_kind == 28 && scoped_found == 0 {
                    semantic_status = 30;
                    semantic_code = 3;
                } else if scoped_target_kind == 28 && scoped_binding_mutable != 1 {
                    semantic_status = 32;
                    semantic_code = 1;
                } else if scoped_current_only == 1 && scoped_found == 1 {
                    semantic_status = 30;
                    semantic_code = 2;
                }
                semantic_node = scoped_target_node;
                word_offset = (semantic_node - 1) * 20 + 4;
                semantic_offset = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
                word_offset = (semantic_node - 1) * 20 + 8;
                semantic_line = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
                word_offset = (semantic_node - 1) * 20 + 12;
                semantic_column = result_value(bytes_get(&origins, word_offset))
                    + result_value(bytes_get(&origins, word_offset + 1)) * 256
                    + result_value(bytes_get(&origins, word_offset + 2)) * 65536
                    + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
                if semantic_status == 0 && scoped_current_only == 1 {
                    scoped_binding_mutable = 0;
                    if scoped_target_kind == 27 {
                        scoped_binding_mutable = 1;
                    }
                    scoped_field = 0;
                    while semantic_status == 0 && scoped_field < 5 {
                        scoped_word = scoped_target_name;
                        if scoped_field == 1 {
                            scoped_word = scoped_function;
                        }                         else if scoped_field == 2 {
                            scoped_word = scoped_scope;
                        }                         else if scoped_field == 3 {
                            scoped_word = scoped_target_node;
                        }                         else if scoped_field == 4 {
                            scoped_word = scoped_binding_mutable;
                        }
                        scoped_byte = 0;
                        while semantic_status == 0 && scoped_byte < 4 {
                            scoped_value = word_byte_0(scoped_word);
                            if scoped_byte == 1 {
                                scoped_value = word_byte_1(scoped_word);
                            } else if scoped_byte == 2 {
                                scoped_value = word_byte_2(scoped_word);
                            } else if scoped_byte == 3 {
                                scoped_value = word_byte_3(scoped_word);
                            }
                            push_result = result_value(bytes_push(&mut scoped_bindings, scoped_value));
                            if push_result < 0 {
                                semantic_status = 26;
                                semantic_code = 9;
                            }
                            scoped_byte = scoped_byte + 1;
                        }
                        scoped_field = scoped_field + 1;
                    }
                    if semantic_status == 0 {
                        scoped_binding_count = scoped_binding_count + 1;
                    }
                }
            }
            scoped_action = scoped_action + 1;
        }
        scoped_index = scoped_index + 1;
    }
    if status == 0 && semantic_status == 0 && scoped_rich == 1
        && (bytes_len(&scoped_bindings) != scoped_binding_count * 20
        || scoped_binding_count > node_count) {
        semantic_status = 27;
        semantic_code = 4;
        semantic_node = root;
        word_offset = (semantic_node - 1) * 20 + 4;
        semantic_offset = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (semantic_node - 1) * 20 + 8;
        semantic_line = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (semantic_node - 1) * 20 + 12;
        semantic_column = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
    }

"#;

const SCOPED_VALUE: &str = r#"
        if semantic_status == 0 && scoped_rich == 1 && semantic_kind == 2 {
            semantic_rule_found = 1;
            word_offset = (node_count - semantic_node) * 12;
            scoped_function = result_value(bytes_get(&scoped_contexts, word_offset))
                + result_value(bytes_get(&scoped_contexts, word_offset + 1)) * 256
                + result_value(bytes_get(&scoped_contexts, word_offset + 2)) * 65536
                + result_value(bytes_get(&scoped_contexts, word_offset + 3)) * 16777216;
            word_offset = (node_count - semantic_node) * 12 + 4;
            scoped_scope = result_value(bytes_get(&scoped_contexts, word_offset))
                + result_value(bytes_get(&scoped_contexts, word_offset + 1)) * 256
                + result_value(bytes_get(&scoped_contexts, word_offset + 2)) * 65536
                + result_value(bytes_get(&scoped_contexts, word_offset + 3)) * 16777216;
            scoped_search_name = semantic_payload;
            scoped_use_node = semantic_node;
            scoped_current_only = 0;
            scoped_found = 0;
            scoped_search_scope = scoped_scope;
            while scoped_found == 0 && scoped_search_scope > 0 {
                scoped_binding_index = scoped_binding_count - 1;
                while scoped_found == 0 && scoped_binding_index >= 0 {
                    word_offset = scoped_binding_index * 20;
                    scoped_binding_name = result_value(bytes_get(&scoped_bindings, word_offset))
                        + result_value(bytes_get(&scoped_bindings, word_offset + 1)) * 256
                        + result_value(bytes_get(&scoped_bindings, word_offset + 2)) * 65536
                        + result_value(bytes_get(&scoped_bindings, word_offset + 3)) * 16777216;
                    word_offset = scoped_binding_index * 20 + 4;
                    scoped_binding_function = result_value(bytes_get(&scoped_bindings, word_offset))
                        + result_value(bytes_get(&scoped_bindings, word_offset + 1)) * 256
                        + result_value(bytes_get(&scoped_bindings, word_offset + 2)) * 65536
                        + result_value(bytes_get(&scoped_bindings, word_offset + 3)) * 16777216;
                    word_offset = scoped_binding_index * 20 + 8;
                    scoped_binding_scope = result_value(bytes_get(&scoped_bindings, word_offset))
                        + result_value(bytes_get(&scoped_bindings, word_offset + 1)) * 256
                        + result_value(bytes_get(&scoped_bindings, word_offset + 2)) * 65536
                        + result_value(bytes_get(&scoped_bindings, word_offset + 3)) * 16777216;
                    word_offset = scoped_binding_index * 20 + 12;
                    scoped_binding_node = result_value(bytes_get(&scoped_bindings, word_offset))
                        + result_value(bytes_get(&scoped_bindings, word_offset + 1)) * 256
                        + result_value(bytes_get(&scoped_bindings, word_offset + 2)) * 65536
                        + result_value(bytes_get(&scoped_bindings, word_offset + 3)) * 16777216;
                    word_offset = scoped_binding_index * 20 + 16;
                    scoped_binding_mutable = result_value(bytes_get(&scoped_bindings, word_offset))
                        + result_value(bytes_get(&scoped_bindings, word_offset + 1)) * 256
                        + result_value(bytes_get(&scoped_bindings, word_offset + 2)) * 65536
                        + result_value(bytes_get(&scoped_bindings, word_offset + 3)) * 16777216;
                    if scoped_binding_name == scoped_search_name
                        && scoped_binding_function == scoped_function
                        && scoped_binding_scope == scoped_search_scope
                        && scoped_binding_node < scoped_use_node {
                        scoped_found = 1;
                    }
                    scoped_binding_index = scoped_binding_index - 1;
                }
                if scoped_found == 0 {
                    if scoped_current_only == 1 {
                        scoped_search_scope = 0;
                    } else {
                        word_offset = (node_count - scoped_search_scope) * 12 + 4;
                        scoped_parent_scope = result_value(bytes_get(&scoped_contexts, word_offset))
                            + result_value(bytes_get(&scoped_contexts, word_offset + 1)) * 256
                            + result_value(bytes_get(&scoped_contexts, word_offset + 2)) * 65536
                            + result_value(bytes_get(&scoped_contexts, word_offset + 3)) * 16777216;
                        if scoped_parent_scope == scoped_search_scope {
                            scoped_search_scope = 0;
                        } else {
                            scoped_search_scope = scoped_parent_scope;
                        }
                    }
                }
            }
            if scoped_found == 0 {
                semantic_status = 17;
                semantic_code = 2;
            } else {
                // Every live binding was admitted from annotation24/int1;
                // the declaration is structural, its resolved use is Copy int.
                semantic_complete_type = 1;
                semantic_ownership = 1;
            }
        }
"#;

const SCOPED_STRUCTURAL: &str = r#"
        if semantic_status == 0 && scoped_rich == 1
            && ((semantic_kind >= 24 && semantic_kind <= 33) || semantic_kind == 38) {
            semantic_rule_found = 1;
            if (semantic_kind == 26 || semantic_kind == 27) && semantic_right_type != 1 {
                semantic_status = 31;
                semantic_code = 1;
                semantic_expected = 1;
                semantic_actual = semantic_right_type;
            } else if semantic_kind == 28 && semantic_left_type != 1 {
                semantic_status = 31;
                semantic_code = 2;
                semantic_expected = 1;
                semantic_actual = semantic_left_type;
            } else if (semantic_kind == 32 || semantic_kind == 33) && semantic_left_type != 2 {
                semantic_status = 31;
                semantic_code = 3;
                semantic_expected = 2;
                semantic_actual = semantic_left_type;
            }
        }
"#;

const SCOPED_ROOT: &str = r#"
    if status == 0 && semantic_status == 0 && scoped_rich == 1 {
        // Typed scalar meaning is complete, but checked lowering is not owned
        // by this slice. A nonzero barrier prevents every later phase.
        semantic_status = 29;
        semantic_code = 1;
        semantic_node = root;
        word_offset = (semantic_node - 1) * 20 + 4;
        semantic_offset = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (semantic_node - 1) * 20 + 8;
        semantic_line = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
        word_offset = (semantic_node - 1) * 20 + 12;
        semantic_column = result_value(bytes_get(&origins, word_offset))
            + result_value(bytes_get(&origins, word_offset + 1)) * 256
            + result_value(bytes_get(&origins, word_offset + 2)) * 65536
            + result_value(bytes_get(&origins, word_offset + 3)) * 16777216;
    }

"#;

/// Frozen additive source identity over accepted connected AST head 6d98b07.
/// This reproducibility check is separate from independent scope/type meaning.
pub(super) fn expected_scoped_source(predecessor: &str) -> String {
    assert_eq!(predecessor.len(), 464_089);
    assert_eq!(
        format!("{:x}", md5::compute(predecessor)),
        "d4f39816b59cd84329502f7c7bf1e348"
    );
    let mut source = predecessor.to_owned();
    for (before, after) in [
        ("    // Match the accepted semantic phase order: all identifier uses first.\n    let mut semantic_index: int = 0;\n    let mut semantic_kind: int = 0;\n    let mut semantic_payload: int = 0;\n    let mut semantic_left: int = 0;\n    let mut semantic_right: int = 0;\n    while status == 0 && semantic_status == 0 && semantic_index < node_count {", format!("{}{}", SCOPED_PRELUDE.trim_start_matches('\n'), "    // Match the accepted semantic phase order: all identifier uses first.\n    let mut semantic_index: int = 0;\n    let mut semantic_kind: int = 0;\n    let mut semantic_payload: int = 0;\n    let mut semantic_left: int = 0;\n    let mut semantic_right: int = 0;\n    while scoped_rich == 0 && status == 0 && semantic_status == 0 && semantic_index < node_count {")),
        ("    // Connected structure is authenticated syntax, not invented scalar meaning.\n    // Keep the identifier-use refusal above first, then refuse the earliest new\n    // structural kind with its fully authenticated origin and no facts.\n    semantic_index = 0;\n    while status == 0 && semantic_status == 0 && semantic_index < node_count {", "    // Connected structure is authenticated syntax, not invented scalar meaning.\n    // Keep the identifier-use refusal above first, then refuse the earliest new\n    // structural kind with its fully authenticated origin and no facts.\n    semantic_index = 0;\n    while scoped_rich == 0 && status == 0 && semantic_status == 0 && semantic_index < node_count {".to_owned()),
        ("        if semantic_status == 0 && semantic_kind == 1 {\n            semantic_rule_found = 1;\n            semantic_complete_type = 1;\n            semantic_ownership = 1;\n        }\n", format!("{}{}", "        if semantic_status == 0 && semantic_kind == 1 {\n            semantic_rule_found = 1;\n            semantic_complete_type = 1;\n            semantic_ownership = 1;\n        }\n", SCOPED_VALUE.trim_start_matches('\n'))),
        ("        if semantic_status == 0 && semantic_kind == 18 {", format!("{}        if semantic_status == 0 && semantic_kind == 18 {{", SCOPED_STRUCTURAL.trim_start_matches('\n'))),
        ("    // Independently checksum the complete provenance/symbol/fact result.", format!("{}    // Independently checksum the complete provenance/symbol/fact result.", SCOPED_ROOT.trim_start_matches('\n'))),
    ] {
        assert_eq!(source.matches(before).count(), 1, "unique scoped source anchor");
        source = source.replacen(before, &after, 1);
    }
    source
}

#[derive(Clone, Copy)]
struct Binding {
    mutable: bool,
}

fn locate(stop: &mut oracle::SemanticStop, model: &oracle::Ingestion, id: i32) {
    let origin = model.origins[(id - 1) as usize];
    stop.node = id;
    stop.offset = origin[1];
    stop.line = origin[2];
    stop.column = origin[3];
}

/// New meaning is an explicit successor; the old representation-only refusal
/// and all historical tables stay untouched.
pub(super) fn scalar_meaning(model: &oracle::Ingestion) -> oracle::SemanticStop {
    if model.nodes.len() > 1024 {
        // The admitted grammar has no new depth cutoff. Independent recursive
        // evidence uses a bounded larger oracle stack, not the product stack.
        return std::thread::scope(|scope| {
            std::thread::Builder::new()
                .stack_size(64 * 1024 * 1024)
                .spawn_scoped(scope, || scalar_meaning_inner(model))
                .expect("bounded independent oracle thread")
                .join()
                .expect("independent scalar oracle completed")
        });
    }
    scalar_meaning_inner(model)
}

fn scalar_meaning_inner(model: &oracle::Ingestion) -> oracle::SemanticStop {
    assert_eq!(model.status, 0);
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
    for (index, node) in model.nodes.iter().enumerate() {
        if node[0] == 19 {
            stop.symbol_words
                .extend_from_slice(&[1, node[1], index as i32 + 1, 1]);
            stop.symbols += 1;
        }
    }
    let rich = model.nodes.iter().any(|node| (24..=38).contains(&node[0]));
    if !rich {
        // Unchanged compact arithmetic, unknown names, calls and references
        // retain the complete historical semantic/LLVM contract.
        return oracle::module_semantic_meaning(model);
    }
    // Classify the entire authenticated rich module before assigning meaning.
    if let Some((index, node)) =
        model.nodes.iter().enumerate().find(|(_, node)| {
            matches!(node[0], 20..=23 | 34..=37) || (node[0] == 24 && node[1] != 1)
        })
    {
        locate(&mut stop, model, index as i32 + 1);
        stop.status = 28;
        stop.code = node[0];
        return stop;
    }
    let mut functions = BTreeMap::new();
    for (index, node) in model.nodes.iter().enumerate() {
        if node[0] == 19 && functions.insert(node[1], index).is_some() {
            locate(&mut stop, model, index as i32 + 1);
            stop.status = 30;
            stop.code = 1;
            return stop;
        }
    }

    struct Names<'a> {
        model: &'a oracle::Ingestion,
        scopes: Vec<BTreeMap<i32, Binding>>,
        resolved: BTreeMap<i32, Binding>,
        failure: Option<(i32, i32, i32)>,
    }
    impl Names<'_> {
        fn binding(&self, name: i32) -> Option<Binding> {
            self.scopes
                .iter()
                .rev()
                .find_map(|scope| scope.get(&name).copied())
        }
        fn fail(&mut self, id: i32, status: i32, code: i32) {
            if self.failure.is_none() {
                self.failure = Some((id, status, code));
            }
        }
        fn define(&mut self, id: i32, name: i32, mutable: bool) {
            let scope = self.scopes.last_mut().expect("function scope exists");
            if scope.contains_key(&name) {
                self.fail(id, 30, 2);
            } else {
                scope.insert(name, Binding { mutable });
            }
        }
        fn visit(&mut self, id: i32, outer_body: bool) {
            if id == 0 || self.failure.is_some() {
                return;
            }
            let [kind, payload, left, right] = self.model.nodes[(id - 1) as usize];
            match kind {
                2 => match self.binding(payload) {
                    Some(binding) => {
                        self.resolved.insert(id, binding);
                    }
                    None => self.fail(id, 17, 2),
                },
                25 => {
                    self.visit(right, false);
                    if self.failure.is_none() {
                        self.define(id, payload, false);
                    }
                }
                26 | 27 => {
                    // Initializer uses the old environment, including a shadowed
                    // outer declaration, before this declaration becomes live.
                    self.visit(right, false);
                    if self.failure.is_none() {
                        self.define(id, payload, kind == 27);
                    }
                }
                28 => {
                    match self.binding(payload) {
                        None => self.fail(id, 30, 3),
                        Some(binding) if !binding.mutable => self.fail(id, 32, 1),
                        Some(binding) => {
                            self.resolved.insert(id, binding);
                        }
                    }
                    self.visit(left, false);
                }
                30 => {
                    if !outer_body {
                        self.scopes.push(BTreeMap::new());
                    }
                    self.visit(left, false);
                    if !outer_body {
                        self.scopes.pop();
                    }
                }
                // Statement sequence is source order; a branch's lexical scopes
                // close independently rather than leaking into the next arm.
                29 | 31 | 32 | 33 => {
                    self.visit(left, false);
                    self.visit(right, false);
                }
                38 => {
                    self.visit(left, false);
                    self.visit(right, true);
                }
                19 => unreachable!("function environment is entered by the module"),
                _ => {
                    self.visit(left, false);
                    self.visit(right, false);
                }
            }
        }
    }
    let mut names = Names {
        model,
        scopes: Vec::new(),
        resolved: BTreeMap::new(),
        failure: None,
    };
    for node in &model.nodes {
        if node[0] == 19 {
            names.scopes.push(BTreeMap::new());
            names.visit(node[2], false);
            names.scopes.pop();
            if names.failure.is_some() {
                break;
            }
        }
    }
    if let Some((id, status, code)) = names.failure {
        locate(&mut stop, model, id);
        stop.status = status;
        stop.code = code;
        return stop;
    }
    // The scalar type fold is postorder, not the recursive lexical traversal.
    // Each failing owner retains exactly the earlier complete fact prefix.
    for (index, node) in model.nodes.iter().enumerate() {
        let id = index as i32 + 1;
        let [kind, _, left, right] = *node;
        let ty = |child: i32| {
            if child == 0 {
                0
            } else {
                stop.fact_words[(child - 1) as usize * 3 + 1]
            }
        };
        let (left_type, right_type) = (ty(left), ty(right));
        let mut value_type = 0;
        let error = match kind {
            1 => {
                value_type = 1;
                None
            }
            2 => {
                assert!(names.resolved.contains_key(&id));
                value_type = 1;
                None
            }
            3 | 4 => {
                let expected = if kind == 3 { 1 } else { 2 };
                if left_type == expected {
                    value_type = expected;
                    None
                } else {
                    Some((if kind == 3 { 24 } else { 23 }, kind, expected, left_type))
                }
            }
            5 | 6 | 8 | 9 => {
                if left_type == 1 && right_type == 1 {
                    value_type = 1;
                    None
                } else {
                    Some((
                        19,
                        kind,
                        1,
                        if left_type != 1 {
                            left_type
                        } else {
                            right_type
                        },
                    ))
                }
            }
            7 => Some((18, 7, 0, 0)),
            10..=15 => {
                if left_type > 0 && left_type == right_type {
                    value_type = 2;
                    None
                } else {
                    Some((20, kind, left_type, right_type))
                }
            }
            16 | 17 => {
                if left_type != 2 {
                    Some((21, kind, 2, left_type))
                } else if right_type != 2 {
                    Some((22, kind, 2, right_type))
                } else {
                    value_type = 2;
                    None
                }
            }
            18 if left_type != 1 => Some((25, 18, 1, left_type)),
            26 | 27 if right_type != 1 => Some((31, 1, 1, right_type)),
            28 if left_type != 1 => Some((31, 2, 1, left_type)),
            32 | 33 if left_type != 2 => Some((31, 3, 2, left_type)),
            18 | 19 | 24..=33 | 38 => None,
            _ => panic!("closed subset was preclassified, unexpected kind {kind}"),
        };
        if let Some((status, code, expected, actual)) = error {
            locate(&mut stop, model, id);
            stop.status = status;
            stop.code = code;
            stop.expected = expected;
            stop.actual = actual;
            return stop;
        }
        stop.fact_words
            .extend_from_slice(&[id, value_type, i32::from(value_type > 0)]);
        stop.facts += 1;
    }
    locate(&mut stop, model, model.root);
    stop.status = 29;
    stop.code = 1;
    stop.root_type = 1;
    stop
}

fn model(source: &[u8]) -> oracle::Ingestion {
    let parsed = connected_ast::successor_parser(source, &module_caps());
    assert_eq!(parsed.status, 0, "{}", String::from_utf8_lossy(source));
    parsed
}

#[test]
fn scalar_source_identity_and_independent_canonical_budget_precede_native() {
    let old = connected_ast::expected_successor_source(&expected_h1a_source());
    let expected = expected_scoped_source(&old);
    let current = h1a_source();
    let first_difference = current
        .bytes()
        .zip(expected.bytes())
        .position(|(a, b)| a != b);
    assert_eq!(
        (current.len(), first_difference),
        (expected.len(), None),
        "frozen source transform differs before any changed native replay"
    );
    let nodes = connected_ast::independent_occurrence_count(current.as_bytes());
    assert!(
        nodes <= 65_536,
        "STOP: unchanged node capacity would overflow: {nodes}"
    );
    let ingested = module_ingest(current.as_bytes());
    assert_eq!(ingested.status, 0);
    assert!(current.len() <= H1A_SOURCE_BOUND);
    assert!(ingested.tokens.len() <= H1A_TOKEN_BOUND);
    assert!(ingested.names.len() <= H1A_NAME_BOUND);
    println!(
        "independent scalar successor: source={} md5={:x} nodes={} tokens={} names={}",
        current.len(),
        md5::compute(current.as_bytes()),
        nodes,
        ingested.tokens.len(),
        ingested.names.len()
    );
}

fn deeply_nested_bool_while_model(depth: usize) -> (Vec<u8>, oracle::Ingestion) {
    assert!(depth > 0);
    let mut source = b"fn f()->int{".to_vec();
    for _ in 0..depth {
        source.extend_from_slice(b"while 1==1{");
    }
    source.extend_from_slice(b"return 1;");
    source.extend(std::iter::repeat_n(b'}', depth));
    source.extend_from_slice(b"return 2;}");
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
    let mut conditions = Vec::new();
    for level in 0..depth {
        let left = append(&mut data, 1, 1, 0, 0, 18 + 11 * level, 2);
        let right = append(&mut data, 1, 1, 0, 0, 21 + 11 * level, 2);
        conditions.push(append(&mut data, 14, 0, left, right, 19 + 11 * level, 26));
    }
    let inner = 12 + 11 * depth;
    let literal = append(&mut data, 1, 1, 0, 0, inner + 7, 2);
    let returned = append(&mut data, 18, 0, literal, 0, inner, 6);
    let cell = append(&mut data, 29, 0, 0, returned, inner, 6);
    let mut body = append(&mut data, 30, 0, cell, 0, 22 + 11 * (depth - 1), 12);
    let mut outer = 0;
    for level in (0..depth).rev() {
        outer = append(
            &mut data,
            33,
            0,
            conditions[level],
            body,
            12 + 11 * level,
            9,
        );
        if level > 0 {
            let cell = append(&mut data, 29, 0, 0, outer, 12 + 11 * level, 9);
            body = append(&mut data, 30, 0, cell, 0, 22 + 11 * (level - 1), 12);
        }
    }
    let first = append(&mut data, 29, 0, 0, outer, 12, 9);
    let final_return = 21 + 12 * depth;
    let literal = append(&mut data, 1, 2, 0, 0, final_return + 7, 2);
    let returned = append(&mut data, 18, 0, literal, 0, final_return, 6);
    let cell = append(&mut data, 29, 0, first, returned, final_return, 6);
    let block = append(&mut data, 30, 0, cell, 0, 11, 12);
    let descriptor = append(&mut data, 38, 1, 0, block, 0, 3);
    data.root = append(&mut data, 19, 1, descriptor, 0, 0, 3);
    data.counts = (2 * depth + 2, depth, depth, 0);
    (source, data)
}

#[test]
fn scalar_large_valid_bool_depth_has_complete_hand_facts() {
    const DEPTH: usize = 10_000;
    let (source, parsed) = deeply_nested_bool_while_model(DEPTH);
    assert_eq!(
        (source.len(), parsed.tokens.len(), parsed.nodes.len()),
        (120_031, 60_015, 60_009)
    );
    assert_eq!(parsed.counts, (20_002, 10_000, 10_000, 0));
    let meaning = scalar_meaning(&parsed);
    assert_eq!(
        (
            meaning.status,
            meaning.code,
            meaning.facts,
            meaning.root_type
        ),
        (29, 1, 60_009, 1)
    );
    assert_eq!(lexical_context_records(&parsed).len() * 12, 720_108);
    for index in 0..DEPTH {
        assert_eq!(
            &meaning.fact_words[index * 9..index * 9 + 9],
            &[
                index as i32 * 3 + 1,
                1,
                1,
                index as i32 * 3 + 2,
                1,
                1,
                index as i32 * 3 + 3,
                2,
                1
            ]
        );
    }
}

#[test]
fn scalar_native_large_valid_bool_depth_authenticates_all_facts_at_o0_o2() {
    let (source, parsed) = deeply_nested_bool_while_model(10_000);
    // The completed pure preflight derives all these quantities independently
    // before the producer is invoked on this unchanged legal-depth grammar.
    assert_eq!((source.len(), parsed.nodes.len()), (120_031, 60_009));
    let meaning = scalar_meaning(&parsed);
    let expected = oracle::refused_expectation_vector(&source, &parsed, &meaning);
    for optimization in ["-O0", "-O2"] {
        assert_eq!(
            connected_ast::run_cached_vector(&source, &expected, optimization),
            91
        );
    }
}

/// Context expectations follow ownership edges, independent of the product's
/// descending interval-restoration loop. Records are finally reversed solely
/// for the frozen direct-index storage ABI.
fn lexical_context_records(model: &oracle::Ingestion) -> Vec<[i32; 3]> {
    if model.nodes.len() > 1024 {
        return std::thread::scope(|scope| {
            std::thread::Builder::new()
                .stack_size(64 * 1024 * 1024)
                .spawn_scoped(scope, || lexical_context_records_inner(model))
                .expect("bounded independent context oracle thread")
                .join()
                .expect("independent context oracle completed")
        });
    }
    lexical_context_records_inner(model)
}

fn lexical_context_records_inner(model: &oracle::Ingestion) -> Vec<[i32; 3]> {
    fn first(model: &oracle::Ingestion, id: i32) -> i32 {
        let node = model.nodes[(id - 1) as usize];
        [node[2], node[3]]
            .into_iter()
            .filter(|&child| child > 0)
            .map(|child| first(model, child))
            .fold(id, i32::min)
    }
    fn walk(
        model: &oracle::Ingestion,
        records: &mut [[i32; 3]],
        id: i32,
        function: i32,
        scope: i32,
        outer_body: bool,
    ) {
        if id == 0 {
            return;
        }
        let [kind, _, left, right] = model.nodes[(id - 1) as usize];
        records[(id - 1) as usize][0..2].copy_from_slice(&[function, scope]);
        if kind == 28 {
            records[(first(model, left) - 1) as usize][2] = id;
        }
        if kind == 30 {
            if outer_body {
                records[(id - 1) as usize][1] = id;
            }
            walk(model, records, left, function, id, false);
        } else if kind == 38 {
            walk(model, records, left, function, right, false);
            walk(model, records, right, function, right, true);
        } else {
            walk(model, records, left, function, scope, false);
            walk(model, records, right, function, scope, false);
        }
    }
    let mut records = vec![[0; 3]; model.nodes.len()];
    for (index, node) in model.nodes.iter().enumerate() {
        if node[0] != 19 {
            continue;
        }
        let function = index as i32 + 1;
        let body = model.nodes[(node[2] - 1) as usize];
        let scope = if body[0] == 38 { body[3] } else { 0 };
        records[index][0..2].copy_from_slice(&[function, scope]);
        walk(model, &mut records, node[2], function, scope, false);
    }
    records.reverse();
    records
}

fn lexical_binding_records(model: &oracle::Ingestion) -> Vec<[i32; 5]> {
    let contexts = lexical_context_records(model);
    model
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            matches!(node[0], 25..=27).then(|| {
                let context = contexts[model.nodes.len() - index - 1];
                [
                    node[1],
                    context[0],
                    context[1],
                    index as i32 + 1,
                    i32::from(node[0] == 27),
                ]
            })
        })
        .collect()
}

const CASES: &[(&str, &[u8])] = &[
    ("parameter", b"fn f(x:int)->int{return x;}"),
    ("mutation", b"fn f()->int{let mut x:int=1;x=2;return x;}"),
    ("shadow", b"fn f(x:int)->int{if x==1{let x:int=x;return x;}return x;}"),
    ("scope-exit", b"fn f()->int{if 1==1{let y:int=2;return y;}return y;}"),
    ("duplicate", b"fn f(x:int)->int{let x:int=1;return x;}"),
    ("initializer-visible", b"fn f()->int{let x:int=x;return x;}"),
    ("condition", b"fn f(x:int)->int{if x{return 1;}return 2;}"),
    ("let-mismatch", b"fn f()->int{let x:int=1==1;return 2;}"),
    ("assignment-mismatch", b"fn f()->int{let mut x:int=1;x=1==1;return 2;}"),
    ("immutable-parameter", b"fn f(x:int)->int{x=1;return x;}"),
    ("immutable-let", b"fn f()->int{let x:int=1;x=2;return x;}"),
    ("function-isolation", b"fn f(x:int)->int{return x;}fn g(x:int)->int{return x;}"),
    ("function-scope-exit", b"fn f(x:int)->int{return x;}fn g(y:int)->int{return x;}"),
    ("nested-loops", b"fn f(x:int)->int{let mut y:int=x;while y>0{if y==2{let y:int=y;return y;}else if y==3{return y;}else{return 4;}y=y-1;}return y;}"),
    ("duplicate-function", b"fn f(x:int)->int{return x;}fn f(y:int)->int{return y;}"),
    ("assignment-target-first", b"fn f()->int{missing=unknown;return 1;}"),
    ("copy-repeated", b"fn f(x:int)->int{return x+x;}"),
    ("nearest-immutable-shadow", b"fn f()->int{let mut x:int=1;if x==1{let x:int=x;x=2;return x;}return x;}"),
    ("nearest-mutable-shadow", b"fn f()->int{let x:int=1;if x==1{let mut x:int=x;x=2;return x;}return x;}"),
    ("sibling-else-isolated", b"fn f()->int{if 1==1{let x:int=1;return x;}else{return x;}return 1;}"),
    ("compact-before-rich", b"fn c()->int{return 1;}fn f(x:int)->int{return x;}"),
    ("rich-before-compact", b"fn f(x:int)->int{return x;}fn c()->int{return 1;}"),
    ("duplicate-parameters", b"fn f(x:int,x:int)->int{return x;}"),
    ("compact-literal-call", b"fn f()->int{return g(1);}"),
];

#[test]
fn scalar_hand_parameter_and_mutation_facts_are_exact() {
    let parameter = scalar_meaning(&model(CASES[0].1));
    assert_eq!(
        (
            parameter.status,
            parameter.code,
            parameter.facts,
            parameter.root_type
        ),
        (29, 1, 8, 1)
    );
    assert_eq!(
        parameter.fact_words,
        vec![
            1, 0, 0, 2, 0, 0, 3, 1, 1, 4, 0, 0, 5, 0, 0, 6, 0, 0, 7, 0, 0, 8, 0, 0
        ]
    );
    let mutation = scalar_meaning(&model(CASES[1].1));
    assert_eq!((mutation.status, mutation.facts), (29, 13));
    let expected_types = [0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0];
    let expected: Vec<_> = expected_types
        .into_iter()
        .enumerate()
        .flat_map(|(index, ty)| [index as i32 + 1, ty, i32::from(ty > 0)])
        .collect();
    assert_eq!(mutation.fact_words, expected);
}

#[test]
fn scalar_context_and_binding_storage_have_hand_derived_identity() {
    let parameter = model(CASES[0].1);
    assert_eq!(lexical_context_records(&parameter), vec![[8, 6, 0]; 8]);
    assert_eq!(lexical_binding_records(&parameter), vec![[2, 8, 6, 2, 0]]);
    let mutation = model(CASES[1].1);
    let mut expected = vec![[13, 11, 0]; 13];
    expected[13 - 5][2] = 6;
    assert_eq!(lexical_context_records(&mutation), expected);
    assert_eq!(lexical_binding_records(&mutation), vec![[3, 13, 11, 3, 1]]);
    let shadow = model(CASES[2].1);
    assert_eq!(shadow.nodes.len(), 22);
    assert_eq!(
        lexical_binding_records(&shadow),
        vec![[2, 22, 20, 2, 0], [2, 22, 13, 8, 0]]
    );
    assert_eq!(
        scalar_meaning(&shadow).fact_words[4 * 3 + 1],
        2,
        "hand comparison node5 is Copy Bool, never an annotation-derived type"
    );
    let nested = model(CASES[13].1);
    let contexts = lexical_context_records(&nested);
    for (index, node) in nested.nodes.iter().enumerate() {
        if node[0] != 30 {
            continue;
        }
        let scope = contexts[nested.nodes.len() - index - 1][1];
        // A nested block records its parent while its statement subtree gets
        // its own scope. Outer body alone is the terminating self-parent.
        assert!(scope >= index as i32 + 1);
    }
}

#[test]
fn scalar_maximum_legal_nested_context_has_no_new_depth_cutoff() {
    const DEPTH: usize = 16_381;
    let (_, parsed) = connected_ast::deeply_nested_while_model(DEPTH);
    assert_eq!(parsed.nodes.len(), 65_533);
    let contexts = lexical_context_records(&parsed);
    assert_eq!(contexts.len() * 12, 786_396);
    assert!(lexical_binding_records(&parsed).is_empty());
    let meaning = scalar_meaning(&parsed);
    assert_eq!(
        (meaning.status, meaning.code, meaning.node, meaning.facts),
        (31, 3, DEPTH as i32 + 5, DEPTH as i32 + 4)
    );
    assert_eq!((meaning.expected, meaning.actual), (2, 1));
}

#[test]
fn scalar_lexical_shadow_scope_exit_and_activation_are_independent() {
    assert_eq!(scalar_meaning(&model(CASES[2].1)).status, 29);
    for index in [3, 5] {
        let meaning = scalar_meaning(&model(CASES[index].1));
        assert_eq!((meaning.status, meaning.code, meaning.facts), (17, 2, 0));
    }
    let duplicate = scalar_meaning(&model(CASES[4].1));
    assert_eq!(
        (duplicate.status, duplicate.code, duplicate.facts),
        (30, 2, 0)
    );
    // Frozen new-route diagnostic priority is deliberately not a claim of
    // Rust's mixed-invalid diagnostic parity: initializer names run before
    // completed-declaration duplicate detection in this bounded name pass.
    let mixed = scalar_meaning(&model(
        b"fn f()->int{let x:int=1;let x:int=missing;return x;}",
    ));
    assert_eq!((mixed.status, mixed.code, mixed.facts), (17, 2, 0));
}

#[test]
fn scalar_exact_type_failures_preserve_hand_fact_prefixes() {
    for (index, code) in [(6, 3), (7, 1), (8, 2)] {
        let meaning = scalar_meaning(&model(CASES[index].1));
        assert_eq!((meaning.status, meaning.code), (31, code));
        assert_eq!(meaning.fact_words.len(), meaning.facts as usize * 3);
        assert_eq!(meaning.facts, meaning.node - 1);
        assert_eq!(meaning.expected, if code == 3 { 2 } else { 1 });
        assert_eq!(meaning.actual, if code == 3 { 1 } else { 2 });
    }
}

#[test]
fn scalar_assignment_checks_target_before_rhs_and_never_rebinds() {
    for index in [9, 10] {
        let meaning = scalar_meaning(&model(CASES[index].1));
        assert_eq!((meaning.status, meaning.code, meaning.facts), (32, 1, 0));
    }
    let source = b"fn f()->int{missing=unknown;return 1;}";
    let meaning = scalar_meaning(&model(source));
    assert_eq!(
        (meaning.status, meaning.code, meaning.offset, meaning.facts),
        (30, 3, 12, 0)
    );
}

#[test]
fn scalar_closed_subset_refuses_result_before_identifier_meaning() {
    let source = b"fn f(x:Result<int,int>)->int{return missing;}";
    let parsed = model(source);
    let old = connected_ast::successor_refusal(&parsed);
    assert_eq!((old.status, old.code), (17, 2));
    let meaning = scalar_meaning(&parsed);
    assert_eq!(
        (meaning.status, meaning.code, meaning.node, meaning.facts),
        (28, 24, 1, 0)
    );
}

#[test]
fn scalar_function_environments_reset_and_nested_control_does_not_leak() {
    for index in [11, 13] {
        let source = CASES[index].1;
        let parsed = model(source);
        let meaning = scalar_meaning(&parsed);
        assert_eq!(
            (meaning.status, meaning.facts),
            (29, parsed.nodes.len() as i32)
        );
        check_program(std::str::from_utf8(source).unwrap(), io_options())
            .expect("independent Rust semantic overlap accepts the scalar witness");
    }
    assert_eq!(scalar_meaning(&model(CASES[12].1)).status, 17);
    let duplicate = scalar_meaning(&model(CASES[14].1));
    assert_eq!(
        (duplicate.status, duplicate.code, duplicate.facts),
        (30, 1, 0)
    );
}

#[test]
fn scalar_compact_success_and_refusals_preserve_the_predecessor() {
    for source in [
        CANONICAL_INPUT,
        &b"fn f()->int{return unknown;}"[..],
        &b"fn f()->int{return g(&x,&mut y,1);}"[..],
        &b"fn f()->int{return g(1);}"[..],
    ] {
        let parsed = model(source);
        let old = oracle::module_semantic_meaning(&parsed);
        let new = scalar_meaning(&parsed);
        assert_eq!(
            (new.status, new.node, new.code, new.facts, new.root_type),
            (old.status, old.node, old.code, old.facts, old.root_type)
        );
        assert_eq!(new.fact_words, old.fact_words);
        assert_eq!(
            oracle::refused_semantic_checksum(&parsed.origins, &new),
            oracle::refused_semantic_checksum(&parsed.origins, &old)
        );
    }
    let literal_call = scalar_meaning(&model(CASES[23].1));
    assert_eq!(
        (literal_call.status, literal_call.code, literal_call.facts),
        (27, 2, 1),
        "a compact call with no Identifier2 retains its old nonempty fact prefix"
    );
}

#[test]
fn scalar_copy_shadow_mutability_sibling_and_mixed_module_authorities() {
    for index in [16, 18, 20, 21] {
        let parsed = model(CASES[index].1);
        let meaning = scalar_meaning(&parsed);
        assert_eq!(
            (meaning.status, meaning.facts),
            (29, parsed.nodes.len() as i32)
        );
        check_program(std::str::from_utf8(CASES[index].1).unwrap(), io_options())
            .expect("independent Rust checker accepts scoped scalar meaning");
    }
    for (index, status, code) in [(17, 32, 1), (19, 17, 2), (22, 30, 2)] {
        let meaning = scalar_meaning(&model(CASES[index].1));
        assert_eq!(
            (meaning.status, meaning.code, meaning.facts),
            (status, code, 0)
        );
    }
}

#[test]
fn scalar_native_observed_tables_facts_and_ownership_trace_are_exact() {
    for (index, context_owner, binding_owner, fact_owner, cleanup) in [
        (
            0,
            13,
            14,
            15,
            &[14, 13, 11, 9, 10, 8, 15, 12, 5, 7, 4, 6, 2, 3, 1][..],
        ),
        (
            1,
            12,
            13,
            14,
            &[13, 12, 10, 8, 9, 7, 14, 11, 4, 6, 5, 2, 3, 1][..],
        ),
        (
            2,
            15,
            16,
            17,
            &[16, 15, 13, 11, 12, 10, 17, 14, 5, 8, 7, 9, 4, 6, 2, 3, 1][..],
        ),
    ] {
        let source = CASES[index].1;
        let parsed = model(source);
        let meaning = scalar_meaning(&parsed);
        let expected = oracle::refused_expectation_vector(source, &parsed, &meaning);
        let fact_records: Vec<[i32; 3]> = meaning
            .fact_words
            .chunks_exact(3)
            .map(|record| record.try_into().unwrap())
            .collect();
        for optimization in ["-O0", "-O2"] {
            let capture =
                connected_ast::run_fault_vector(source, &expected, optimization, 0, "", None);
            capture.validate_trace_at(cleanup, 91);
            assert_eq!(
                capture.records::<3>(context_owner),
                lexical_context_records(&parsed)
            );
            assert_eq!(
                capture.records::<5>(binding_owner),
                lexical_binding_records(&parsed)
            );
            assert_eq!(capture.records::<3>(fact_owner), fact_records);
        }
    }
}

#[test]
fn scalar_native_context_and_binding_heap_failures_are_located_and_clean() {
    let source = CASES[0].1;
    let parsed = model(source);
    let before_contexts = connected_ast::growth_events_before_starts(source)
        + connected_ast::growth_events(parsed.nodes.len() * 4) * 2
        + connected_ast::growth_events(8)
        + connected_ast::growth_events(parsed.nodes.len() * 8)
        + connected_ast::growth_events(16);
    let before_bindings = before_contexts + connected_ast::growth_events(parsed.nodes.len() * 12);
    for (code, node, events, initial, owner) in [
        (8, 8, before_contexts, true, 13),
        (8, 8, before_contexts + 1, false, 13),
        (9, 2, before_bindings, true, 14),
        (9, 2, before_bindings + 1, false, 14),
    ] {
        let mut meaning = scalar_meaning(&parsed);
        locate(&mut meaning, &parsed, node);
        meaning.status = 26;
        meaning.code = code;
        meaning.facts = 0;
        meaning.root_type = 0;
        meaning.fact_words.clear();
        let expected = oracle::refused_expectation_vector(source, &parsed, &meaning);
        for optimization in ["-O0", "-O2"] {
            let capture = connected_ast::run_fault_vector(
                source,
                &expected,
                optimization,
                0,
                "",
                Some((events, initial)),
            );
            assert_eq!(
                capture.failure_descriptor(),
                if initial {
                    [1, 0, 0, 8]
                } else {
                    [2, owner as u64, 8, 16]
                }
            );
            assert_eq!(capture.counters[3..], [0, 0]);
            assert_eq!(
                capture.counters[0],
                capture.counters[2] + u64::from(initial)
            );
            if !initial {
                assert_eq!(
                    capture.records::<1>(owner),
                    if code == 8 {
                        vec![[8], [6]]
                    } else {
                        vec![[2], [8]]
                    }
                );
            }
        }
    }
}

#[test]
fn scalar_native_full_vectors_red_first_at_o0_o2() {
    // Finish every run before asserting, retaining both optimization-level reds
    // for every frozen behavior rather than stopping at the first mismatch.
    let mut failures = Vec::new();
    for &(label, source) in CASES {
        let parsed = model(source);
        let meaning = scalar_meaning(&parsed);
        let expected = oracle::refused_expectation_vector(source, &parsed, &meaning);
        // The signature parser appends its parameter-count ABI after the
        // original 67 pipeline fields; neither field family may be omitted.
        assert_eq!(expected.len(), 68);
        for optimization in ["-O0", "-O2"] {
            let actual = connected_ast::run_cached_vector(source, &expected, optimization);
            if actual != 91 {
                failures.push((label, optimization, actual));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "genuine scoped scalar full-vector mismatches: {failures:?}"
    );
}
