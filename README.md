# emulate-server

## How to deploy the server

### Build the docker image

We provide a Dockerfile to deploy the server. To build the image, please use the following command:

```bash
docker build -t emulate-server:latest -f Dockerfile .
```

### Use postgres docker container to store data

To run the postgres docker container, please use the following command:

```bash
docker pull postgres:latest
# the user name, password and database name are all "quantum-emulator", and the port is 5433
# it will create a volume named "pg-data" to store the data
docker run -d --name=quantum-emulator-pg -e POSTGRES_PASSWORD=quantum-emulator -e POSTGRES_USER=quantum-emulator -e POSTGRES_DB=quantum-emulator -p 5433:5432 --restart always -v pg-data:/var/lib/postgresql/data postgres:latest
```

### Run the emulate-server using host network

To run the emulate-server, please use the following command:

```bash
docker run -d --network=host --name=emulate-server --env QSCHED_CONFIG=/qsched.json --env LOG_CONFIG=/log4rs.yaml -v /path/to/qsched:/qsched.json -v /path/to/agent/file:/agent.json -v /path/to/log4rs:/log4rs.yaml --restart=always emulate-server:latest
```

You can use `QSCHED_CONFIG` to specify the path of the configuration file for the quantum scheduler. The default value is `/qsched.json`. The agent file path is specified by `agent_file` in thee configuration file of the quantum scheduler. Please make sure the configuration file and the agent file are accessible by the server.

**NOTE**: The agent file is mainly for develop and testing. If you don't want to use the agent file, please set the value of `agent_file` to `""`.

The database url is specified by `db_url` in the configuration file of the quantum scheduler. Please make sure the database url is accessible by the server.

You can use `LOG_CONFIG` to specify the path of the configuration file for log system. The default value is `/log4rs.yaml`. Please make sure the log configuration file are accessible by the server. The default log path is `/log/requests.log`.

Then, you can use `emulate-client` to submit jobs to the server.

## Use docker compose to start the server

You can use following command to run the emulator server:

```bash
docker compose -p emulator-server up -d
```

Previous command will start the postgres, emulator server and two agents. It will use self defined docker network `emulator-server` and use the volume `pgdata` to store the data of the postgres. And the agent file `config/agents-compose.json` (The only difference with `config/agents` is that it will use hostname to add agent) will be used. The server will use the port 3000 to listen the request.

Use following command to stop the emulator server:

```bash
docker compose -p emulator-server down
```

Use following command to stop the emulator server and remove all named volumes:

```bash
docker compose -p emulator-server down -v
```

## How to develop the server

### Apply migrations after changing the schema

To apply migrations, please use the following command:

```bash
cd migration
# This will drop all tables from the database, then reapply all migrations
# The dabase url is same with the one in the configuration file of the quantum scheduler, that means the databse you previously created
cargo run -- -u database_url fresh
```

**NOTE:** This operation may be dangerous, please make sure you have a backup of the database.

### Generate a new entity from the database

To generate a new entity from the database, please use the following command:

```bash
# go to the root directory of the project
cd ../
# This will generate the entity from the database, and the entity will be stored in the src/entity directory
# The database url is same with the one in the configuration file of the quantum scheduler, that means the databse you previously created
sea-orm-cli generate entity -o src/entity/ --with-serde=both  -u database_url
```

## Generate the documentation

To generate the documentation, please use the following command:

```bash
cargo doc --bins --document-private-items --no-deps
```

## Use k8s to deploy the server

You can use the following command to deploy the server:

```bash
kubectl apply -f emulator-server.yaml
```

The file include following resources:
- emulator-server-db-secret: A config map to store the environment variables for the postgres
- emulator-server-sched-conf: A config map to store the configuration file for the quantum scheduler, which is a json file.
- emulator-server-db-pvc: A persistent volume claim to store the data of the postgres。
- emulator-server: A pod including the server and the database, the database can not be accessed from outside. The server will use the port 3000 (can not access directly) to listen the request.
- emulator-server-sv: A service to expose the emulator server to the outside using NodePort, the port is 30001.

You can use the following command to delete the resources:

```bash
kubectl delete -f emulator-server.yaml
```

This will delete all the resources created by the previous command.

You can use `kubectl` get the pv of emulator server log, then you can go to that path to check the log file.

## Use k8s to deploy the agent:

You can use the following command to deploy the agent:

```bash
kubectl apply -f emulator-agent.yaml
```

The file include following resources:
- emulator-agent: A pod including the agent, the agent will use the port 3003 (can not access direcyly) to listen the request.
- emulator-agent-sv: A service to expose the agent to cluster using ClusterIP, the port is 3003.

You can use the following command to delete the resources:

```bash
kubectl delete -f emulator-agent.yaml
```

You can use the following command to add the agent to the server:

```bash
# The hostname is the name of the agent service, the address (-a) is the cluster ip of the emulator-server service, you can use kubectl get servce to check it
# You can also use the node ip and the node port to add the agent
cargo run -- -m add-agent --agent-hostname emulator-agent-sv-1 --agent-port 3003 --agent-qubit-count 20 --agent-circuit-depth 20 -a 10.108.202.16:3000
```

You can use the following command to submit a task to the server:

```bash
# The address (-a) is the node ip and the node port of the emulator-server service
cargo run -- -m emulate -f examples/bell.qasm -s 2000 -a 192.168.1.196:30001
```