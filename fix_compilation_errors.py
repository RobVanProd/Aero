#!/usr/bin/env python3
"""
Script to fix compilation errors in the Aero compiler
"""

import os
import re

def fix_file(filepath, replacements):
    """Apply replacements to a file"""
    if not os.path.exists(filepath):
        print(f"File not found: {filepath}")
        return
    
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    for old, new in replacements:
        content = content.replace(old, new)
    
    if content != original_content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Fixed {filepath}")
    else:
        print(f"No changes needed for {filepath}")

def main():
    # Common replacements for all files
    common_replacements = [
        # Expression variants
        ('Expression::Number(', 'Expression::IntegerLiteral('),
        ('Expression::Float(', 'Expression::FloatLiteral('),
        
        # Binary expression fields
        ('{ op, lhs, rhs, ty }', '{ op, left, right, ty }'),
        ('{ op, lhs, rhs, .. }', '{ op, left, right, .. }'),
        
        # UnaryOp variants
        ('UnaryOp::Minus', 'UnaryOp::Negate'),
        
        # Let statement patterns
        ('Statement::Let { name, value }', 'Statement::Let { name, mutable: _, type_annotation: _, value }'),
        
        # Type field access
        ('.param_type.name', '.param_type'),
        ('ast_type.name.as_str()', 'match ast_type { Type::Named(name) => name.as_str() }'),
    ]
    
    # File-specific fixes
    files_to_fix = {
        'src/compiler/src/semantic_analyzer.rs': [
            # Handle Option<Expression> properly
            ('self.infer_and_validate_expression(value)?', 
             'if let Some(ref mut val) = value { self.infer_and_validate_expression(val)? } else { Ty::Int }'),
            ('self.check_expression_initialization(expr)?', 
             'if let Some(ref val) = expr { self.check_expression_initialization(val)? }'),
            ('self.infer_and_validate_expression_immutable(value)?', 
             'if let Some(ref val) = value { self.infer_and_validate_expression_immutable(val)? } else { Ty::Int }'),
            
            # Fix binary operation type inference
            ('infer_binary_type(op, &lhs_type, &rhs_type)?', 
             'infer_binary_type(&format!("{:?}", op).to_lowercase(), &lhs_type, &rhs_type)?'),
        ],
        
        'src/compiler/src/ir_generator.rs': [
            # Handle Option<Expression> properly
            ('self.generate_expression_ir(value, current_function)', 
             'if let Some(val) = value { self.generate_expression_ir(val, current_function) } else { (Value::ImmInt(0), Ty::Int) }'),
            ('self.generate_expression_ir(expr, current_function)', 
             'if let Some(val) = expr { self.generate_expression_ir(val, current_function) } else { (Value::ImmInt(0), Ty::Int) }'),
            ('self.generate_expression_ir_for_function(value, function_body)', 
             'if let Some(val) = value { self.generate_expression_ir_for_function(val, function_body) } else { (Value::ImmInt(0), Ty::Int) }'),
            ('self.generate_expression_ir_for_function(expr, function_body)', 
             'if let Some(val) = expr { self.generate_expression_ir_for_function(val, function_body) } else { (Value::ImmInt(0), Ty::Int) }'),
            
            # Fix binary operation handling
            ('self.try_constant_fold(&op, &promoted_lhs, &promoted_rhs, &result_type)', 
             'self.try_constant_fold(&format!("{:?}", op).to_lowercase(), &promoted_lhs, &promoted_rhs, &result_type)'),
            ('op.as_str()', 'format!("{:?}", op).to_lowercase().as_str()'),
            
            # Fix Type field access
            ('p.param_type.name.clone()', 'match &p.param_type { Type::Named(name) => name.clone() }'),
            ('param.param_type.name.as_str()', 'match &param.param_type { Type::Named(name) => name.as_str() }'),
        ],
        
        'src/compiler/src/parser.rs': [
            # Add missing ty field to Binary expressions
            ('Expression::Binary {\n                op,\n                left: Box::new(left),\n                right: Box::new(right),\n            }', 
             'Expression::Binary {\n                op,\n                left: Box::new(left),\n                right: Box::new(right),\n                ty: None,\n            }'),
        ]
    }
    
    # Apply common replacements to all Rust files
    for root, dirs, files in os.walk('src/compiler/src'):
        for file in files:
            if file.endswith('.rs'):
                filepath = os.path.join(root, file)
                fix_file(filepath, common_replacements)
    
    # Apply file-specific fixes
    for filepath, replacements in files_to_fix.items():
        fix_file(filepath, replacements)

if __name__ == '__main__':
    main()