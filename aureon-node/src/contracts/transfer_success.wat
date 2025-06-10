(module
  (import "env" "log" (func $log))
  (func (export "main")
    ;; Simulate "Transfer executed"
    call $log
  )
)