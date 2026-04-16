# Rust/hm-pi-htmx

- Always collapse if statements per https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
- Always inline format! args when possible per https://rust-lang.github.io/rust-clippy/master/index.html#uninlined_format_args
- Use method references over closures when possible per https://rust-lang.github.io/rust-clippy/master/index.html#redundant_closure_for_method_calls
- When possible, make `match` statements exhaustive and avoid wildcard arms.
- Do not create small helper methods that are referenced only once.
- Do not leave silent errors or default values. You don't have to handle them gracefully, but you need to at least log it.
- Prioritize readable, explicit code.
- Make impossible states unrepresentable.


### 8.3.117 SGR - SELECT GRAPHIC RENDITION
Notation: (Ps...)
Representation: CSI Ps... 06/13
Parameter default value: Ps = 0
SGR is used to establish one or more graphic rendition aspects for subsequent text. The established aspects remain in effect until the next occurrence of SGR in the data stream, depending on the setting of the GRAPHIC RENDITION COMBINATION MODE (GRCM). Each graphic rendition aspect is specified by a parameter value: 
- 0 default rendition (implementation-defined), cancels the effect of any preceding occurrence of SGR in the data stream regardless of the setting of the GRAPHIC RENDITION COMBINATION MODE (GRCM)
- The rest present in `adjust_sgr()`