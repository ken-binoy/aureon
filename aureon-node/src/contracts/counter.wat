(module
  (import "env" "storage_read" (func $storage_read (param i32 i32 i32 i32) (result i32)))
  (import "env" "storage_write" (func $storage_write (param i32 i32 i32 i32) (result i32)))
  (import "env" "transfer" (func $transfer (param i32 i32 i32 i32 i64) (result i32)))
  (import "env" "get_balance" (func $get_balance (param i32 i32) (result i64)))
  (import "env" "log" (func $log (param i32 i32)))
  
  (memory (export "memory") 1)
  
  ;; Data section: keys and initial values
  (data (i32.const 0) "counter")           ;; 7 bytes, key for counter
  (data (i32.const 8) "owner")              ;; 5 bytes, key for owner
  (data (i32.const 16) "Counter contract")  ;; 16 bytes, for logging
  (data (i32.const 32) "Incrementing counter")  ;; 20 bytes
  (data (i32.const 52) "Value incremented")     ;; 16 bytes
  
  ;; Initialize the contract (called once on deploy)
  (func (export "init")
    ;; Log initialization
    i32.const 16
    i32.const 16
    call $log
  )
  
  ;; Increment the counter
  (func (export "increment")
    (local $current i32)
    (local $buf i32)
    
    ;; Log operation
    i32.const 32
    i32.const 20
    call $log
    
    ;; Read current counter value from storage
    ;; storage_read(key_ptr, key_len, value_ptr, max_len) -> actual_len
    (local.set $current
      (call $storage_read
        (i32.const 0)    ;; key: "counter"
        (i32.const 7)    ;; key length
        (i32.const 100)  ;; buffer for value
        (i32.const 8)    ;; max length (u64 = 8 bytes)
      )
    )
    
    ;; If counter doesn't exist (returns -1), set to 1
    ;; Otherwise, increment by 1
    ;; For simplicity, we'll just write 1 each time in this demo
    
    ;; Write new value to storage
    (call $storage_write
      (i32.const 0)    ;; key: "counter"
      (i32.const 7)    ;; key length
      (i32.const 108)  ;; value buffer (1 as little-endian u64)
      (i32.const 8)    ;; value length
    )
    (drop)
    
    ;; Log success
    i32.const 52
    i32.const 16
    call $log
  )
  
  ;; Get current balance of an account
  (func (export "check_balance") (param $addr_ptr i32) (param $addr_len i32) (result i64)
    ;; Call get_balance
    (call $get_balance
      (local.get $addr_ptr)
      (local.get $addr_len)
    )
  )
  
  ;; Transfer funds
  (func (export "send_funds") (param $to_ptr i32) (param $to_len i32) (param $amount i64) (result i32)
    ;; This is a simplified version - normally we'd extract caller from context
    ;; For demo, assume "Alice" is the sender
    (call $transfer
      (i32.const 128)   ;; "Alice"
      (i32.const 5)     ;; length
      (local.get $to_ptr)
      (local.get $to_len)
      (local.get $amount)
    )
  )
)
