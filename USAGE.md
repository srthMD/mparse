## Constants

Constants can be interpreted from plaintext (like "pi" or "e"). See [constants.rs](./src/constants.rs) for all of the constants.

## Functions

MParse includes many different common mathematical functions for the user to call, like trig functions.
Specifically for trigonometric functions (and angle), you can pass the -d or --deg flag to evaluate trig functions in degree mode. All functions must also have opening and closing brackets.

See the FunctionType enum in [functions.rs](./src/functions.rs) for all of the functions built into MParse.

#### RNG Function

MParse has a builtin rand() function that is just a wrapper for the random() function in the [rand](https://crates.io/crates/rand) crate, however this feature is optional and MParse can
be disabled by disabling the rand feature.

#### Bases

Some functions like root(...) and log(...) can take in bases in the form func\_#(...), where the base has to be a sinulgar primitive number, no inner expressions or constants (for now).  
Here is an example of a valid usage of bases.

Acts as log base 5.

```bash
mparse "log_5(20)"
```

In this case we are taking the 4th root of 16.

```bash
mparse "root_4(16)"
```

## Objects and Typechecking
MParse also has the ability to work on library-defined structs (Vec2D for example), along with floats, where all non-numerical objects are constructed via functions. MParse will also check all operations and function calls beforehand to check for type compatibility, i.e. something like "vec2(10, 5) * vec3(12, 8, -2)" will throw a type error since you cannot multiply a 2D vector with a 3D one.

### Fields
Some objects have fields that you can index with the dot ('.') operator, just like many other programming languages.

```bash
  mparse "(5*vec2(-5, 2.5)).x"
```

You can also use indexing as a shorthand for some function calls.

```bash
  mparse "vec3(-2, 5, 2).magnitude"
```
Indexing an object with no fields (like a number) will throw an error, and indexing an object with an invalid field will also throw an error.

## Implied Multiplication

MParse supports expressions like "4pi" or "2sqrt(9)" and the parser will assume to preform a multiplication if nessecary. It also works on the right hand side (i.e. "ln(9.5)8" ) (its kind of ugly though).
