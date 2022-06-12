publish:
	cd $(shell pwd)/chronos && cargo build --release
	sudo systemctl stop olympos.chronos
	sudo cp $(shell pwd)/chronos/target/release/chronos /usr/share/olympos/chronos/
	sudo systemctl start olympos.chronos
	sudo cp $(shell pwd)/cli/* /usr/share/olympos/chronos/

publish_cli:
	sudo cp $(shell pwd)/cli/* /usr/share/olympos/chronos/

publish_chronos:
	cd $(shell pwd)/chronos && cargo build --release
	sudo systemctl stop olympos.chronos
	sudo cp $(shell pwd)/chronos/target/release/chronos /usr/share/olympos/chronos/
	sudo systemctl start olympos.chronos

