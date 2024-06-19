# Instructions

The instruction set of the Vine Virtual Machine (VVN) is split into features. Every program for the VVN must specify a set of essential instructions and a set of optional instructions it will use. Features group instructions.

## Control Flow

- `unreachable`
- `noop`

## Registers

There are three registers: the primary register _A_, the secondary register _B_ and the return register _R_. 

## Stack

- `stack.push_a`
- `stack.push_b`
- `stack.push_r`
- `stack.clear`
- `stack.pop`
- (`stack.pop_into_a`)
- (`stack.pop_into_b`)
- (`stack.pop_into_r`)
- `stack.top_into_a`
- `stack.top_into_b`
- `stack.top_into_r`
- `stack.swap`
- `stack.swap_a`
- `stack.swap_b`
- `stack.swap_r`
- `stack.duplicate`

## _Feature_ Standard IO

- `args`
- `stdout.write`
- `stdout.write_lf`
- `stderr.write`
- `stderr.write_lf`
- `read_line`
- `read`

## _Feature_ File IO

- `file.exists`
- `file.is_file`
- `file.is_directory`
- `file.create`
- `file.read`
- `file.write`
- `file.delete`
- `file.delete_empty`
- `file.size`
- `file.move`
- `file.copy`
- `file.get_created`
- `file.set_created`
- `file.get_modified`
- `file.set_modified`
- `file.is_temporary`
- `file.mark_temporary`
- `file.mark_permanent`
- `file.is_hidden`
- `file.mark_hidden`
- `file.mark_shown`
- `file.create_directory`
- `file.read_directory`
- `file.step_directory`

## Math

### Trigonometry

- `number.sine`
- `number.cosine`
- `number.tangent`
- `number.arcus_sine`
- `number.arcus_cosine`
- `number.arcus_tangent`
- `number.hyperbolic_sine`
- `number.hyperbolic_cosine`
- `number.hyperbolic_tangent`
- `number.hyperbolic_arcus_sine`
- `number.hyperbolic_arcus_cosine`
- `number.hyperbolic_arcus_tangent`
- `number.hypotenuse`

### More

- `e_to`
- `power`
- `log_e`
- `log_2`
- `log_10`
- `log_n`
- `random`
- `square_root`
- `cube_root`
- `nth_root`
- `abs`
- `ceil`
- `floor`
- `round`
- `max`
- `min`
- `sign`
- `truncate`
- `add`
- `sub`
- `mul`
- `rem`
- `div`
- `exp`

### Bitwise

- `number.invert`
- `number.leading_zeroes`
- `number.|`
- `number.&`
- `number.^`

### Comparative

- `equal`
- `not_equal`
- `less`
- `less_or_equal`
- `greater`
- `greater_or_equal`

## Objects

- `object.new`
- `object.cast_equivalent`
- `object.0`
- `object.1`
- `object.2`
- `object.3`
- `object.n`

## Strings

- `string_new`
- `string.to_upper_case`
- `string.to_lower_case`