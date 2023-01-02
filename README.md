# PostGIS Diesel
A [PostGIS Diesel fork](https://github.com/vitaly-m/postgis-diesel) with simpler SRID support.

# Main Differences
1. Replace `srid` fields in all geometry types with `const generics`, for example, the definition of `Point`:
```rust
// Original version
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub srid: u32,
}

// Forked version
pub struct Point<const SRID: u32> {
    pub x: f64,
    pub y: f64,
}
```

2. Implement some useful traits for all geometry types, e.g., `Default`, `FromIterator`.
3. Implement `FromSql` and `ToSql` for `GeometryContainer`.
4. `Polygon` is represented by `line strings` instead of vectors of points.  
5. Reorganize tests. Add `srid_test` and tests for 3D geometry types. 

# Motivation
Geometry types with `srid` field are more flexible, for example, types with different `SRIDs` can be used together easily. However, it requires more work and caution:
1. Every time we create a new instance, its `SRID` need to be set manually.
2. `FromIterator` is hard to implement properly. Because it needs to check whether the `SRIDs` of all items are the same. In the original version, this trait is not implemented. The crate [rust-postgis](https://github.com/andelf/rust-postgis), which is an extension to `rust-postgres`, does support `FromIterator`, but the `SRID` of generated instance is always zero, which is counter-intuitive.
3. It is hard to check whether a geometry type has the expected `srid` values. In contraction, our fork version makes it become a type constraint, which can be checked easily. For example, the implementation of trait `FromSql` checks whether the rust side `SRID` (though const generic) is equal to the `SRID` in the database side, if not, a deserialization error will be returned. See [srid_test](https://github.com/WeissP/postgis-diesel/blob/generic-SRID/tests/from_to_sql_test.rs#L10) for more details.

# Comparsion with examples
## Struct Definition
Original: 
```rust
use postgis_diesel::types::*;

struct NewGeometrySample {
    point: Point,
    polygon: polygon<Point>,
}
```

Fork:
```rust
use postgis_diesel::types::*;

struct Sample {
    point: Point<4326>,
    polygon: Polygon<4326, Point<4326>>,
}
```

Or we can import geometry types in `gps`, which exports type aliases where all `SRIDs` are 4326:
```rust
use postgis_diesel::gps::*;

struct Sample {
    point: Point,
    polygon: Polygon,
}
```

## New Instance

To crate an instance of `Sample` in the original version (copied from [integration_test.rs](https://github.com/vitaly-m/postgis-diesel/blob/master/tests/integration_test.rs) in the original repository):

```rust
fn new_line(points: Vec<(f64, f64)>) -> LineString<Point> {
    let mut l_points = Vec::with_capacity(points.len());
    for p in points {
        l_points.push(Point {
            x: p.0,
            y: p.1,
            srid: Option::Some(4326),
        });
    }
    LineString {
        points: l_points,
        srid: Option::Some(4326),
    }
}

fn new_sample() -> Sample {
    Sample {
        point: Point::new(0., 0., Some(4326)),
        linestring: new_line(vec![(0., 0.), (1., 1.)]),,
    }
}

```

In our fork version:
```rust
fn new_sample() -> Sample {
    Sample {
        point: Point::new(0., 0.),
        linestring: vec![Point::new(0., 0.), Point::new(1., 1.)]
            .into_iter()
            .collect(),
    }
}
```

# Usage 
See the [original docment](https://github.com/vitaly-m/postgis-diesel) as there is no big differences between them. 
