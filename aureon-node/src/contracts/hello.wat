(module
  (import "env" "log" (func $log (param i32 i32)))
  (memory 1)
  (export "memory" (memory 0))
  (data (i32.const 0) "Hello from WASM!")
  (func (export "run")
    i32.const 0    ;; pointer to string
    i32.const 16   ;; string length
    call $log
  )
)