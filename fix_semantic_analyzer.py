#!/usr/bin/env python3

import re

# Read the file
with open('src/compiler/src/semantic_analyzer.rs', 'r') as f:
    content = f.read()

# Fix all the method calls that need .borrow()
patterns_to_fix = [
    (r'self\.type_manager\.validate_field_access\(', r'self.type_manager.borrow().validate_field_access('),
    (r'self\.type_manager\.get_method\(', r'self.type_manager.borrow().get_method('),
    (r'self\.type_manager\.get_struct\(', r'self.type_manager.borrow().get_struct('),
    (r'self\.type_manager\.get_enum\(', r'self.type_manager.borrow().get_enum('),
]

for pattern, replacement in patterns_to_fix:
    content = re.sub(pattern, replacement, content)

# Write the file back
with open('src/compiler/src/semantic_analyzer.rs', 'w') as f:
    f.write(content)

print("Fixed semantic analyzer method calls")