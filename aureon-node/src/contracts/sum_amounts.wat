(module
  (import "env" "log" (func $log (param i32 i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "Summing all amounts")

  (func (export "run")
    i32.const 0
    i32.const 19
    call $log
  )
)