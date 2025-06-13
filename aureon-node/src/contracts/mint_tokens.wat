(module
  (import "env" "log" (func $log (param i32 i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "Tokens minted successfully")

  (func (export "run")
    i32.const 0
    i32.const 26
    call $log
  )
)