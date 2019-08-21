

## About nanoda

Enormous thanks to Leonard de Moura, Soonho Kong, Sebastian Ullrich, Gabriel Ebner, Floris van Doorn, Mario Carneiro, Kevin Buzzard, Chris Hughes, Patrick Massot, Jeremy Avigad, and Kenny Lau for their combined efforts in creating/documenting Lean, and/or for their willingness to share their knowledge with the unwashed masses on Lean's Zulip.

This project is based on Gabriel Ebner's [trepplein](https://github.com/gebner/trepplein)

--- 

Nanoda is a type checker for the Lean theorem prover, specifically its [export format](https://github.com/leanprover/lean/blob/master/doc/export_format.md). It includes a pretty printer and a command line interface. It was written with primarily as a learning exercise, and I would eventually like to be able to tie it to something like a markdown book that explores that ties the implementation to the math/theory parts of dependent type theory. I got a pretty good handle on the former in working on this, but the latter it still largely a mystery to me, so if you're an expert and are interested in doing something, hit me up.

---

### How to use

1. Install cargo (Rust's package manager) if you don't already have it.
2. Clone this repository.
3. From this repository's root folder, execute `cargo build --release` (it will be incredibly slow without the release flag, so don't forget that)
4. The built binary will be in /target/release/nanoda, so you can either run it from there (use `./nanoda --help` to see options), or you can run it through cargo, but the syntax is a little weird : `cargo run --release -- <options/flags> <export_files>`. For example `cargo run --release -- --threads 8 --print mathlib_export.out`

---

## nanoda について

Leonard de Moura, Soonho Kong, Sebastian Ullrich, Gabriel Ebner, Floris van Doorn, Mario Carneiro, Kevin Buzzard, Chris Hughes, Patrick Massot, Jeremy Avigad, と Kenny Lau にすごく感謝してます。

元々Gabriel Epnerの[trepplein](https://github.com/gebner/trepplein)を参照して作られたものです。


※ 日本語で書かれているコメントは `jp_comments` とうい枝で見えます。

このプロジェクトは Lean とうい証明支援システム・依存型プログラミング言語の型検査装置です。プリティープリンターもCLIも含む。将来、このコードベースに基づく「依存型の検査装置を数学・コンピュータ科学両面から調査して見よう」って感じの markdown 本と組んでいきたいんですが、どうやって進んでいけばいいかってことが分かりません。依存型システムの具体的な部分をこのプロジェクトでよく分かってきたんだと思いますが、理論・数学との形式的な繋がりっていままでもよくわからない所が沢山あり。このプロジェクトへ何か投稿したい・いいアイディアがある・日本語の修正がある方々ならぜひご連絡して下さい。

### 使い方

1. cargo (ラスト言語のパケージマネジャー)をインストールして下さい。
2. このリポジトリーをクローンして。
3. このリポのルートフォルダーから、`cargo build --release` にして下さい。`--release` の分がなければ非常に遅くなるので忘れないで下さい。
4. 作られた実行形式は /target/release/nanoda に位置しているはずですので、そこから普通のように実行出来ます(`./nanoda --help` で詳しいことが見える)。cargo でも実行できますが、構文はちょっと長たらしくなって : `cargo run --release -- <options/flags> <export_files>` っていうように、例えば `cargo run --release -- --threads 8 --print mathlib_export.out`。


