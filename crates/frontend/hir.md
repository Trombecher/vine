# High-Level Intermediate Representation (HIR)

This is a sketch document.

```
fn id__1 () -> () {
    let o = (.core.num.U8.add by .core.ops.Add<U8, Output=U8>) (left = 8, right = 9)

    .std.io.println(o)
}
```