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
docker run -d --network=host --name=emulate-server emulate-server:latest
```

Then, you can use `emulate-client` to submit jobs to the server.