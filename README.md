# emulate-server

## How to run

We provide a Dockerfile to run the server. To build the image, please use the following command:

```bash
docker build -t emulate-server:latest -f Dockerfile .
```

To run the postgres docker container, please use the following command:

```bash
docker pull postgres:latest
docker run -it --name=quantum-emulator-pg -e POSTGRES_PASSWORD=quantum-emulator -e POSTGRES_USER=quantum-emulator -e POSTGRES_DB=quantum-emulator -p 5433:5432 --restart always -v pg-data:/var/lib/postgresql/data postgres:latest
```

To run the emulate-server, please use the following command:

```bash
docker run -d --network=host --name=emulate-server --restart=always emulate-server:latest
```

Then, you can use `emulate-client` to submit jobs to the server.

## Apply migrations

To apply migrations, please use the following command:

```bash
cd migration
# this will drop all tables from the database, then reapply all migrations
cargo run -- fresh
```

## Generate a new entity from the database

To generate a new entity from the database, please use the following command:

```bash
cd ../
sea-orm-cli generate entity -o src/entity/
```
