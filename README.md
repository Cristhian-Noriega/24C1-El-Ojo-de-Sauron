# Taller de Programacion

## Grupo - El Ojo de Sauron
- 108666 - Alvarez, Mateo
- 102707 - Andresen, Joaquín
- 110119 - Gismondi, Maximo
- 109164 - Noriega, Cristhian David

## Documentación
Para ver la documentación se recomienda utilizar el siguiente comando:
```
cargo doc --no-deps --open
```

## Como usar
Todos los ejecutables requieren de un archivo de configuración.

### Server
```
cargo run --bin server <settings-toml-path>
```

### Monitor
```
cargo run --bin monitor <settings-toml-path>
```

### Camera System
```
cargo run --bin camera-system <config-json-path>
```

### Drone
```
cargo run --bin drone <config-json-path>
```


## Como testear
```
cargo test --manifest-path=project/Cargo.toml
```