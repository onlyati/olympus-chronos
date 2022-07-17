publish:
	cd $(shell pwd)/chronos && cargo build --release
	sudo systemctl stop olympus.chronos
	sudo cp $(shell pwd)/chronos/target/release/chronos /usr/share/olympus/chronos/
	sudo systemctl start olympus.chronos
	cd $(shell pwd)/cli && cargo build --release
	sudo cp $(shell pwd)/cli/target/release/cli /usr/share/olympus/chronos/

publish_cli:
	cd $(shell pwd)/cli && cargo build --release
	sudo cp $(shell pwd)/cli/target/release/cli /usr/share/olympus/chronos/

publish_chronos:
	cd $(shell pwd)/chronos && cargo build --release
	sudo systemctl stop olympus.chronos
	sudo cp $(shell pwd)/chronos/target/release/chronos /usr/share/olympus/chronos/
	sudo systemctl start olympus.chronos

