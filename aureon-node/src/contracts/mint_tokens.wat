(module
  (import "env" "log" (func $log))
  (func (export "main")
    ;; Simulate "Tokens minted successfully"
    call $log
  )
)