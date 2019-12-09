## About nanoda

Enormous thanks to Leonard de Moura, Soonho Kong, Sebastian Ullrich, Gabriel Ebner, Floris van Doorn, Mario Carneiro, Kevin Buzzard, Chris Hughes, Patrick Massot, Jeremy Avigad, and Kenny Lau for their combined efforts in creating/documenting Lean, and/or for their willingness to share their knowledge with the unwashed masses on Lean's Zulip.

This project is based on Gabriel Ebner's [trepplein](https://github.com/gebner/trepplein)

--- 

Nanoda is a type checker for the Lean theorem prover, specifically its [export format](https://github.com/leanprover/lean/blob/master/doc/export_format.md). It includes a pretty printer and a command line interface. 


### Future plans

This was written mostly just to try and learn more about dependent type theory and math in general, so I would like to continue expanding it with features that make it more informative/educational in the hopes that other people will find it helpful. 

+ In the short term, I want to add the ability to isolate/print components of definitions to the pretty printer interface, IE if you want to see specifically the motive and minor premises for a type, you get just that information from the command line instead of looking through the source code and debug printing it. I would also like to add highlighting/bracket coloring to the pretty printer output.

+ Also short term, I'll be looking into whether or not the ability to visualize expression graphs with graphviz is actually helpful or not. It's pretty easy to do, but most of the expressions are so large that it might not be very helpful.

+ In the mid-long term, I'd like to add a time-travelling debugger style interface that allows users to walk forward and backward through the steps of type inference and unification one at a time with an annotation that explains what each step is doing. I'm still thinking about the best way to go about this part.

+ I would really like to be able to either add sections to the doc comments, or make a separate markdown book that ties portions of the implementation to more information about their basis in math and dependent type theory, or annotations that just say "this implementation is essentially axiomatic, it does what it does because it needs to, the end". This is (IMO) the hardest/experts only part that will (if it happens) likely come from outside sources who know more about this than I do.

### Rest

If you'd like to contribute, have ideas for features or documentation, or want to tell me I have no idea what I'm doing and call me names, feel free to contact me.

---

### How to use

** As of 0.1.1, [mimalloc](https://github.com/microsoft/mimalloc.git) is the default global allocator, but you can disable this by passing the `--no-default-features` flag when running the executable. Thanks (again) to Sebastian Ullrich for this suggestion.

1. Install cargo (Rust's package manager) if you don't already have it.
2. Clone this repository.
3. From this repository's root folder, execute `cargo build --release` (it will be incredibly slow without the release flag, so don't forget that). 
4. The built binary will be in /target/release/nanoda, so you can either run it from there (use `./nanoda --help` to see options), or you can run it through cargo, but the syntax is a little weird : `cargo run --release -- <options/flags> <export_files>`. For example `cargo run --release -- --threads 8 --print mathlib_export.out`

---

## nanoda について

Leonard de Moura, Soonho Kong, Sebastian Ullrich, Gabriel Ebner, Floris van Doorn, Mario Carneiro, Kevin Buzzard, Chris Hughes, Patrick Massot, Jeremy Avigad, と Kenny Lau にすごく感謝してます。

元々Gabriel Epnerの[trepplein](https://github.com/gebner/trepplein)を参照して作られたものです。


※ 日本語で書かれているコメントは `jp_comments` とうい枝で見えます。

このプロジェクトは Lean とうい証明支援システム・依存型プログラミング言語の型検査装置です。プリティープリンターもCLIも含む。


### 将来

このプロジェクトの主たる目的は型論理の教育道具になることですから、そのためのフィーチャーをこれからも追加していきたいんです。

+ 近いうちに、定義・環境にあるアイテムから特定の成分を抜き出せる機能を実装していくつもりです。例えば、ある定義の motive と minor premises だけ見たい時、直接にコマンドラインオプションから出来るようにすることです。プリティープリンターがにはいライティングなどのことも有効にしたいんだと思います。

+ それから、time travelling debugger のようなインタフェースがある型推論・ユニフィケーションのステップを一歩ずつで進んでいけるモードを作っていきたいんです。もっともよい実装する方法にすいてまだ考えているんですけど。

+ ドックコメント、もしくは自立のドキュメンテーションに、実装の具体的な成分はどうやって数学・依存型論理と繋がっているかってことを説明する文もあったらすごくいいと思いますが、この部分は私より型論理をしっかり抑えている方から来たほうがぜったいに良かったです。

### 残り

何かの投稿・日本語の修正・いいアイディア・フィーチャーなどがある方々なら、ぜひご連絡して下さい。

### 使い方


** バージョン0.1.1現在, デフォールトで用いられるアロケーターは[mimalloc](https://github.com/microsoft/mimalloc.git)ですが`--no-default-features`フラグを渡すことでmimallocの代わりにシステムのデフォールトが使える。

1. cargo (ラスト言語のパケージマネジャー)をインストールして下さい。
2. このリポジトリーをクローンして。
3. このリポのルートフォルダーから、`cargo build --release` にして下さい。`--release` の分がなければ非常に遅くなるので忘れないで下さい。
4. 作られた実行形式は /target/release/nanoda に位置しているはずですので、そこから普通のように実行出来ます(`./nanoda --help` で詳しいことが見える)。cargo でも実行できますが、構文はちょっと長たらしくなって : `cargo run --release -- <options/flags> <export_files>` っていうように、例えば `cargo run --release -- --threads 8 --print mathlib_export.out`。


