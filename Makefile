.PHONY: deploy
deploy:
	cargo build
	sudo cp ./target/debug/wauth /usr/local/bin/
	chmod +x /usr/local/bin/wauth

.PHONY: completion
completion:
	cargo run -- generate-fish-completion > ~/.config/fish/completions/wauth.fish
