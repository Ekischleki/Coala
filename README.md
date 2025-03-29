Docs abt the language will come soon. For now, here's some example code

```
sub or_problem(bool: a, bool: b) {
  let a_or_b = or(a, b)
  force a_or_b => false #Equivalent to forcing a and b to be false
}

sub not_problem(bool: a) {
  force not a => false
}

sub and(bool: a, bool: b) {
  let res = not or(not a, not b)
} = res

sub force_equal(bool a, bool b) {
  let both_true = and(a, b)
  let both_false = not or(a, b)
  force or(both_true, both_false) => true
}
```
