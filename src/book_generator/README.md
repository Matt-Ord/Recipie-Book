To generate the latex and md files use

```shell
cd src/book_generator
cargo run -- --from "../../recipes" --to-markdown "../markdown_book/generated" --to-latex "../latex_book/generated"
```
