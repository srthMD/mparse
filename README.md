# MParse

A simple library for parsing and evaluating basic mathematical expressions from plaintext.  
Please don't use this for any serious uses, I mainly just made this for fun.

## Usage

MParse takes in a single (preferrably ASCII but Unicode works) string for evaluation. All whitespace is ignored. Here are just some examples of valid inputs.

```bash
mparse "5 * 3 + 12"
```

```bash
mparse "(12 * 0.10)/5"
```

```bash
mparse "5cos(pi/2)"
```

See [USAGE.md](./USAGE.md) for more information on all of the features of MParse.

## Credits

Thanks to [this](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) blog post from matklad because I had no idea how to implement the AST before reading it.
