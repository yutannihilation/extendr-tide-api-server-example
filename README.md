⚠WIP⚠ An example using extendr inside a simple API server ⚠WIP⚠
====================================================

## Usage

### Server

```sh
cargo run
```

Disclaimer: I don't find how to stop this server properly yet. Currently I do `pkill extendr-tide-ap`.

### Client

Plot any number of points,

```sh
curl localhost:8080/plot/point -d '{ "x": 6, "y": 2, "radius": 4 }'
curl localhost:8080/plot/point -d '{ "x": 1, "y": 2, "radius": 1 }'
curl localhost:8080/plot/point -d '{ "x": 1, "y": 3, "radius": 0.3 }'
```

and view the result on browser on <http://localhost:8080/plot/result>.
Note that currently you can view it only once. The server doesn't accept the second view.
