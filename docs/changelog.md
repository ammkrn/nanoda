## + 23/12/2019

+ Version bump from 0.1.x to 0.2.x.

+ 0.2.x marks Nanoda's move to be more like lean4's type checker. 0.2.0 is functional but sketchy in parts; there are partial functions and `unimplemented!()`s abound, particularly around places where lean4 uses `FVar`s, `Theorem`s, `Projection`s, and anything to do with `local_ctx` since I have no idea how that works. I subbed `Local`s for local_ctx items in some places, and I'm not sure yet if I've stepped on any correctness landmines by doing that.

+ Fixed a pretty printer bug where all parameters would print with a `@` prefix.

+ Removal of the `reduction` module in favor of recursors (this ended up being a rewrite of a majority of the source code).

+ 0.2.0 is serial only; no parallel checking until I figure out a parallel setup I'm more happy with.

+ removed dependency on stacker; new tc module does not have stack growth problems.

+ Signifcant performance increase on large files. Although core takes slightly longer to check (~5s compared to 0.1.x's ~3.5s), mathlib is down from ~460s to ~280s on one thread. The amount of memory being used also seems to be about 30% less.

## + 31/08/2019

Made changes to allow the stack to grow when executing recursive definitional equality checks. This is rarely used, but is necessary to check definitions like `pi_gt_314` without experiencing stack overflow. Some other changes to related parts of the type checker are planned to bring it more in line with Lean 4 which will hopefully give some insight into how the stack growth can be alleviated.

--- 

## + 24/08/2019

Switched to [mimalloc](https://github.com/microsoft/mimalloc.git) as the default global allocator. Can be disabled by pasing the `--no-default-features` flag when running the executable.