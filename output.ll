; ModuleID = "aero_compiler"
source_filename = "aero_compiler"
define i64 @main() {
entry:
  %x = alloca i64, align 8
  store i64 15, i64* %x, align 8
  ret i64 0
}


