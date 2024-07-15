# Taller de Programacion

## Grupo - El Ojo de Sauron

- 108666 - Alvarez, Mateo
- 102707 - Andresen, Joaquín
- 110119 - Gismondi, Maximo
- 109164 - Noriega, Cristhian David

## Documentación

Para ver la documentación se recomienda utilizar el siguiente comando:

```sh
cargo doc --no-deps --open
```

## Preparación
Para poder utilizar la función de reconocimiento de imagenes en el sistema de camaras es necesario tener configuradas las credenciales de AWS

```sh
aws configure
```

Podemos comprobar nuestras credenciales con

```sh
cat ~/.aws/credentials 
```

## Como usar

Una manera rápida de ejecutar todo el sistema es utilizando el ejecutable run.sh siendo n la cantidad de drones.

```sh
chmod +x run.sh
./run.sh <n>
```

Si se desea correr cada componente por separado es importante tener en cuenta que por parámetro se deben pasar los archivos de configuración que correspondan.

### Server

```sh
cargo run --bin server <settings-toml-path>
```

### Monitor

```sh
cargo run --bin monitor <config-json-path>
```

### Camera System

```sh
cargo run --bin camera-system <config-json-path>
```

### Drone

```sh
cargo run --bin drone <config-json-path>
```

## Como testear

```sh
cargo test --manifest-path=project/Cargo.toml
```
