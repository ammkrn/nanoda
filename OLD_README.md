## About nanoda

Enormous thanks to Leonard de Moura, Soonho Kong, Sebastian Ullrich, Gabriel Ebner, Floris van Doorn, Mario Carneiro, Kevin Buzzard, Chris Hughes, Patrick Massot, Jeremy Avigad, and Kenny Lau for their combined efforts in creating/documenting Lean, and/or for their willingness to share their knowledge with the unwashed masses on Lean's Zulip.

This project is based on Gabriel Ebner's [trepplein](https://github.com/gebner/trepplein)

--- 

Nanoda is a type checker for the Lean theorem prover, specifically its [export format](https://github.com/leanprover/lean/blob/master/doc/export_format.md). It includes a pretty printer and a command line interface. 


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


このプロジェクトは Lean とうい証明支援システム・依存型プログラミング言語の型検査装置です。プリティープリンターもCLIも含む。


### 使い方


** バージョン0.1.1現在, デフォールトで用いられるアロケーターは[mimalloc](https://github.com/microsoft/mimalloc.git)ですが`--no-default-features`フラグを渡すことでmimallocの代わりにシステムのデフォールトが使える。

1. cargo (ラスト言語のパケージマネジャー)をインストールして下さい。
2. このリポジトリーをクローンして。
3. このリポのルートフォルダーから、`cargo build --release` にして下さい。`--release` の分がなければ非常に遅くなるので忘れないで下さい。
4. 作られた実行形式は /target/release/nanoda に位置しているはずですので、そこから普通のように実行出来ます(`./nanoda --help` で詳しいことが見える)。cargo でも実行できますが、構文はちょっと長たらしくなって : `cargo run --release -- <options/flags> <export_files>` っていうように、例えば `cargo run --release -- --threads 8 --print mathlib_export.out`。


