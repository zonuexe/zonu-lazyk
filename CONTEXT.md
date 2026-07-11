# zonu-lazyk

A Rust interpreter for **Lazy K**, a pure, lazy, functional language whose only primitives are the SKI combinators. A program is a single combinator term that maps an input byte stream to an output byte stream. See <https://tromp.github.io/cl/lazy-k.html>.

## Language

**Program**:
A single combinator term. Running it means applying it to the input stream and forcing the resulting output stream.
_Avoid_: script, expression (the whole thing is one term)

**Combinator**:
A primitive of the calculus: `S`, `K`, `I`. `S f g x = f x (g x)`, `K x y = x`, `I x = x`.
_Avoid_: builtin, function

**Term**:
A node in the program: a combinator or an **Application** of one term to another. The entire program is one term.
_Avoid_: expression, AST node

**Application**:
Juxtaposition of two terms — the operator applied to the operand.
_Avoid_: call, invocation

**Notation**:
One of the four surface syntaxes the parser accepts, freely mixable: **Unlambda** (`` ` `` apply; `s` `k` `i`), **Combinatory Logic / CC** (`S` `K` `I`, parens, juxtaposition), **Iota** (`*` apply; `i` iota), **Jot** (binary `0`/`1`).
_Avoid_: syntax, dialect, format

**Church numeral**:
The encoding of a natural number as a term (`n f x = f (f (… x))`). Bytes on the input and output streams are Church numerals.
_Avoid_: integer, number (reserve those for host-language `usize`/`u8`)

**Byte stream**:
A lazy cons-list of Church numerals, each a byte `0..=255`. Input and output are both byte streams.
_Avoid_: list, sequence, string

**EOF**:
End of stream, signalled by a numeral `>= 256`. On input the stream is terminated by 256; on output, forcing an element `>= 256` stops the program.
_Avoid_: nil, terminator, sentinel

## Reduction

**Redex**:
A reducible application whose head is a combinator with enough arguments on the spine to fire its rewrite rule.
_Avoid_: reducible, hot spot

**Normal-order reduction**:
The evaluation order — reduce the leftmost-outermost redex first. This is what makes Lazy K lazy; only what the output stream demands gets evaluated.
_Avoid_: lazy evaluation (name the order), call-by-need

**WHNF**:
Weak head normal form — a term reduced far enough to expose its outermost combinator/constructor. The reducer forces a term to WHNF, not full normal form.
_Avoid_: normal form, evaluated

**Extended combinator**:
A derived combinator (`B`, `C`, `S'`, `B'`, `C'`, …) the compiler introduces to shrink the graph. `B f g x = f (g x)`, `C f g x = f x g`, `S' c f g x = c (f x) (g x)`. Semantically derivable from SKI; used only to cut reduction steps.
_Avoid_: primitive (these are derived), builtin

**Peephole optimization**:
The compile-time pass that rewrites local SKI patterns into **extended combinators** (`S(Kp)(Kq) → K(pq)`, `S(Kp)I → p`, `S(Kp)q → B p q`, `Sp(Kq) → C p q`, …) before execution.
_Avoid_: rewriting, simplification (be specific)
