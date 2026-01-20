# C Specifications

This documents contain information about the specifications of C programming language to implement in the Cleo compiler.

### Literal Numbers

Integers:

| Base        | Prefix   | Example     |
|-------------|----------|-------------|
| Decimal     | none     | 1234        |
| Octal       | 0        | 01234       |
| Hexadecimal | 0x or 0X | 0x1A3F      |
| Binary      | 0b or 0B | 0b1010      |

Suffixes for integer literals to specify their type:
- `u/U`: unsigned
- `l/L`: long
- `ul/UL`: unsigned long
- `ll/LL`: long long
- `ull/ULL`: unsigned long long

Floating-point:
| Format          | Example        |
|-----------------|----------------|
| Decimal         | 123.456        |
| Scientific      | 1.23456e2      |
| Hexadecimal     | 0x1.91eb86p+1  |

Suffixes for floating-point literals to specify their type:
- `f/F`: float
- `l/L`: long double
