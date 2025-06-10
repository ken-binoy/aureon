(module
  (import "env" "log" (func $log))
  (func (export "main")
    ;; Simulate log call for "Summing all amounts"
    call $log
  )
)