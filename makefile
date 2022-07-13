

run-secur-server:
	cargo run --all-features --bin server amqp://guest:guest@0.0.0.0:5672 hello2 8000

run-secure-client:
	cargo run --all-features --bin client wss://yorker:8000/thisisatest amqp://guest:guest@0.0.0.0:5672 hello

run-server:
	cargo run --bin server yorker.pfx yorker amqp://guest:guest@0.0.0.0:5672 hello2 8000

run-client:
	cargo run --bin client ws://0.0.0.0:8000/thisisatest amqp://guest:guest@0.0.0.0:5672 hello

run-tester:
	cargo run --bin tester amqp://guest:guest@0.0.0.0:5672 hello hello2

run-docker:
	docker-run --rm -it -d --hostname my-rabbit --name some-rabbit -p 15672:15672 -p 5672:5672 mesh/rabbitmq:3-management

build:
	cargo build --release --all-features


gen-cert:
	openssl req -nodes -x509 -newkey rsa:2048 -subj '/CN=yorker' -keyout yorker.key -out yorker.crt -days 365
	openssl pkcs12 -export -nodes -inkey yorker.key -in yorker.crt -out yorker.pfx

add-cert:
	sudo cp yorker.crt /use/local/share/ca-certificates
	sudo update-ca-certificates


pem-to-pfx:
	openssl pkcs12 -inkey bob_key.pem -in bob_cert.cert -export -out bob_pfx.pfx

