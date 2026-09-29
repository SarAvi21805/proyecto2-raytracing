# Arquitectura del ray tracer

El programa separa la presentacion en ventana del calculo de la imagen. Raylib solo recibe un arreglo RGBA ya terminado; no calcula intersecciones, iluminacion, texturas ni efectos.

## Flujo de cada render

1. `Camera::ray_for_pixel` crea un rayo normalizado por pixel.
2. `VoxelWorld::intersect` recorre la rejilla con DDA y retorna la primera frontera visible.
3. `Renderer::trace` obtiene textura, normal perturbada e iluminacion.
4. Los rayos secundarios calculan reflexion y refraccion hasta `max_bounces`.
5. Si no existe impacto, `skybox` calcula el color del ambiente.
6. Varias filas se distribuyen entre hilos de CPU con `std::thread::scope`.
7. Raylib presenta el framebuffer RGBA en una textura de pantalla.

## Responsabilidades

| Modulo | Responsabilidad |
| --- | --- |
| `math.rs` | Vector 3D y operaciones matematicas |
| `ray.rs` | Origen, direccion y evaluacion de un rayo |
| `camera.rs` | Camara orbital, perspectiva y zoom |
| `texture.rs` | Ocho texturas procedurales calculadas en CPU |
| `material.rs` | Parametros fisicos y visuales de cada bloque |
| `world.rs` | Rejilla voxel, DDA y terreno procedural |
| `scene.rs` | Composicion del diorama y luces |
| `renderer.rs` | Sombreado, sombras, reflexion, refraccion y paralelismo |
| `main.rs` | Ventana, entradas y presentacion del framebuffer |

## Decisiones de rendimiento

- DDA visita solo los voxeles atravesados por el rayo; no prueba cada cubo.
- El framebuffer se divide por grupos de filas entre los nucleos disponibles.
- La imagen se vuelve a calcular solo cuando cambia la camara o el terreno.
- Las texturas son funciones pequenas y no hacen lecturas de disco durante el render.
- El perfil `release` habilita optimizacion, LTO delgado y una sola unidad de codigo.

