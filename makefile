

run-secur-server:
	cargo run --all-features --bin server amqp://guest:guest@0.0.0.0:5672 hello2 8000

run-secure-client:
	cargo run --all-features --bin client wss://yorker:8000/thisisatest amqp://guest:guest@0.0.0.0:5672 hello

run-server:
	cargo run --bin server yorker.pfx yorker amqp://guest:guest@0.0.0.0:5672 hello2 8000

run-client:
	cargo run --bin client ws://yorker:8000/thisisatest amqp://guest:guest@0.0.0.0:5672 hello

run-tster:
	cargo run --bin tester amqp://guest:guest@0.0.0.0:5672 hello hello2

build:
	cargo build --release --all-features


