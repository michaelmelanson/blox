[
  "let"
  "def"
  "import"
  "if"
] @keyword

[
  (boolean_true)
  (boolean_false)
] @boolean

[
  "("
  ")"
  "{"
  "}"
] @punctuation.bracket

(number) @number
(string) @string

[
  (multiply)
  (divide)
  (concatenate)
  (add)
  (subtract)
  (equal)
  (not_equal)
  (greater_or_equal)
  (greater_than)
  (less_or_equal)
  (less_than)
  "="
] @operator

(definition
  name:(identifier) @function
  parameter:(identifier) @parameter
)

(function_call name:(identifier) @function.call)
