# Third-party notices

## TrumpScript

Crazy Race downloads the archived TrumpScript interpreter during its Docker build.

- Repository: `https://github.com/samshadwell/TrumpScript`
- Pinned commit: `3793b905925b55c0296b066586c7d612c7220ca0`
- License: MIT
- Copyright: the TrumpScript contributors listed by the upstream project

The Dockerfile applies one compatibility-only change to `src/trumpscript/parser.py`: it supplies `type_ignores=[]` when constructing `ast.Module` so that the 2016 interpreter can execute on a modern Python 3 runtime.

The upstream project's license remains applicable to the downloaded interpreter.
