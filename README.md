```sh
mkdir -p ~/.config/wauth
touch ~/.config/wauth/config.toml
vim ~/.config/wauth/config.toml

aws_profile = "foo"
dynamodb_table_name = "bar"
```

```sh
make deploy
make completion
```
