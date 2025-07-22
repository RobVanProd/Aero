; ModuleID = "aero_compiler"
source_filename = "aero_compiler"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

declare i32 @printf(i8*, ...)

define i32 @main() {
entry:
  %ptr0 = alloca double, align 8
  store double 0x4014000000000000, double* %ptr0, align 8
  %ptr1 = alloca double, align 8
  store double 0x4024000000000000, double* %ptr1, align 8
  ret i32 0
}

