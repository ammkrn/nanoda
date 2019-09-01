
## + 31/08/2019

Made changes to allow the stack to grow when executing recursive definitional equality checks. This is rarely used, but is necessary to check definitions like `pi_gt_314` without experiencing stack overflow. Some other changes to related parts of the type checker are planned to bring it more in line with Lean 4 which will hopefully give some insight into how the stack growth can be alleviated.

--- 

## + 24/08/2019

Switched to [mimalloc](https://github.com/microsoft/mimalloc.git) as the default global allocator. Can be disabled by pasing the `--no-default-features` flag when running the executable.