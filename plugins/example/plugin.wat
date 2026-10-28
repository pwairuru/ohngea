;; Reference guest for the Extism sandbox (used by the `example`-route hook).
;; Source of truth: compiled to `plugin.wasm` (checked in); the
;; `guest_wasm_matches_wat` test fails if they drift apart.
;; Pattern: official Extism WAT guest — host fns from `extism:host/user`,
;; memory plumbing via `extism:host/env`, `(result i32)` 0 = success.
(module
  (import "extism:host/user" "pbx_log" (func $log (param i64)))
  (import "extism:host/user" "db_get" (func $db_get (param i64) (result i64)))
  (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
  (import "extism:host/env" "store_u8" (func $store_u8 (param i64 i32)))
  (import "extism:host/env" "output_set" (func $output_set (param i64 i64)))
  (import "extism:host/env" "length" (func $length (param i64) (result i64)))
  (memory $mem (export "memory") (export "mem") 1)
  (data $mem (i32.const 0) "ok:example")
  (func $copy (param $off i64) (param $n i64)
    (local $i i32)
    (block $end (loop $loop
      (br_if $end (i32.ge_u (local.get $i) (i32.wrap_i64 (local.get $n))))
      (call $store_u8
        (i64.add (local.get $off) (i64.extend_i32_u (local.get $i)))
        (i32.load8_u (local.get $i)))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $loop))))
  (func (export "handle") (result i32)
    (local $off i64)
    (local.set $off (call $alloc (i64.const 10)))
    (call $copy (local.get $off) (i64.const 10))
    (call $log (local.get $off))
    (call $output_set (local.get $off) (call $length (local.get $off)))
    (i32.const 0))
  (func (export "check_db") (result i32)
    (if (i64.ne (call $db_get (i64.const 1)) (i64.const 1))
      (then unreachable))
    (if (i64.ne (call $db_get (i64.const 999)) (i64.const 0))
      (then unreachable))
    (i32.const 0)))
