; ModuleID = "aero_compiler"
source_filename = "aero_compiler"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"
define i32 @main() {
entry:
  %ptr0 = alloca i64, align 8
  store i64 21, i64* %ptr0, align 8
  %reg0 = load i64, i64* %ptr0, align 8
  %reg1 = trunc i64 %reg0 to i32
  ret i32 %reg1
}


