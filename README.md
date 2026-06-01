# MParse
A simple library for parsing and evaluating basic mathematical expressions from plaintext.  
Please don't use this for any serious uses (i have no idea what bugs lay in here), i mainly just made this for fun.   

## Usage
MParse takes in a single (preferrably ASCII but Unicode works) string for evaluation. All whitespace is ignored. Here are just some examples of valid inputs.
```bash
mparse "5 * 3 + 12"
```
```bash
mparse "sqrt(90) % 3"
```
```bash
mparse "5cos(pi/2)"
```  

### Operators
All operators are considered single character symbols that can act as unary or binary operations.  
See [operators.rs](./src/operators.rs) and the Operation enum for all of the valid operators.

### Constants
Constants can be interpreted from plaintext (like "pi" or "e"). See [constants.rs](./src/constants.rs) for all of the constants.  

### Functions
Also self explanatory, MParse includes some built in functions you can call, like trigonometric functions.  
Specifically for trigonometric functions, you can pass the -d or --deg flag to evaluate trig functions in degree mode.  
All functions must also have opening and closing brackets. 

#### Bases
Some functions like root(...) and log(...) can take in bases in the form func_#(...), where the base has to be a sinulgar primitive number, no inner expressions or constants (for now).  
Here is an example of a valid usage of bases.  
  
Acts as log base 5.
```bash
mparse "log_5(20)"
```
In this case we are taking the 4th root of 16.  
```bash
mparse "root_4(16)"
```
See the FunctionType enum in  [functions.rs](./src/functions.rs) for all of the functions built into MParse.

### Implied Multiplication
MParse supports expressions like "4pi" or "2sqrt(9)" and the parser will assume to preform a multiplication if nessecary. It also works on the right hand side (i.e. "ln(9.5)8" ) (its kind of ugly though).  

## Credits
Thanks to [this](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) blog post from matklad because I had no idea how to implement the AST before reading it.
