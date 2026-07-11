# Third-party notices

## TrumpScript

Crazy Race downloads the archived TrumpScript interpreter during its Docker build.

- Repository: `https://github.com/samshadwell/TrumpScript`
- Pinned commit: `3793b905925b55c0296b066586c7d612c7220ca0`
- License: MIT
- Copyright: the TrumpScript contributors listed by the upstream project

The Dockerfile applies compatibility-only changes to `src/trumpscript/parser.py` so the 2016 interpreter can execute on modern Python: it supplies `type_ignores=[]` to `ast.Module` and maps the removed `Num`, `Str`, and `NameConstant` constructor aliases to `ast.Constant`. These changes do not alter the TrumpScript grammar or game-specific behavior.

The upstream project's license remains applicable to the downloaded interpreter.
