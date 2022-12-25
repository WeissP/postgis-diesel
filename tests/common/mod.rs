use std::{env, sync::Once};

use diesel::{pg::PgConnection, Connection, RunQueryDsl};
use dotenv::dotenv;
use postgis_diesel::{gps, types::*};

static INIT: Once = Once::new();

pub struct NewGeometrySampleG<const SRID: u32, P: PointT<SRID>> {
    pub name: String,
    pub point: P,
    pub linestring: LineString<SRID, P>,
    pub polygon: Polygon<SRID, P>,
    pub multipoint: MultiPoint<SRID, P>,
    pub multiline: MultiLineString<SRID, P>,
    pub multipolygon: MultiPolygon<SRID, P>,
    pub geometrycollection: GeometryCollection<SRID, P>,
}

#[derive(Insertable)]
#[diesel(table_name = geometry_samples)]
pub struct NewGeometrySample2D<const SRID: u32> {
    pub name: String,
    pub point: Point<SRID>,
    pub linestring: LineString<SRID, Point<SRID>>,
    pub polygon: Polygon<SRID, Point<SRID>>,
    pub multipoint: MultiPoint<SRID, Point<SRID>>,
    pub multiline: MultiLineString<SRID, Point<SRID>>,
    pub multipolygon: MultiPolygon<SRID, Point<SRID>>,
    pub geometrycollection: GeometryCollection<SRID, Point<SRID>>,
}

#[derive(Insertable)]
#[diesel(table_name = geometry_samples)]
pub struct NewGeometrySample3D<const SRID: u32> {
    pub name: String,
    pub point: PointZ<SRID>,
    pub linestring: LineString<SRID, PointZ<SRID>>,
    pub polygon: Polygon<SRID, PointZ<SRID>>,
    pub multipoint: MultiPoint<SRID, PointZ<SRID>>,
    pub multiline: MultiLineString<SRID, PointZ<SRID>>,
    pub multipolygon: MultiPolygon<SRID, PointZ<SRID>>,
    pub geometrycollection: GeometryCollection<SRID, PointZ<SRID>>,
}

#[derive(Queryable, Debug, PartialEq)]
pub struct GeometrySample<const SRID: u32, P: PointT<SRID>> {
    pub id: i32,
    pub name: String,
    pub point: P,
    pub linestring: LineString<SRID, P>,
    pub polygon: Polygon<SRID, P>,
    pub multipoint: MultiPoint<SRID, P>,
    pub multiline: MultiLineString<SRID, P>,
    pub multipolygon: MultiPolygon<SRID, P>,
    pub geometrycollection: GeometryCollection<SRID, P>,
}

#[derive(Insertable)]
#[diesel(table_name = distance_samples)]
pub struct NewDistanceSample<const SRID: u32> {
    pub name: String,
    pub point: Point<SRID>,
    pub polygon: Polygon<SRID, Point<SRID>>,
}

#[derive(Queryable, Debug, PartialEq)]
#[diesel(table_name = distance_samples)]
pub struct DistanceSample<const SRID: u32> {
    pub id: i32,
    pub name: String,
    pub point: Point<SRID>,
    pub polygon: Polygon<SRID, Point<SRID>>,
}

impl<const SRID: u32> Into<NewGeometrySample2D<SRID>> for NewGeometrySampleG<SRID, Point<SRID>> {
    fn into(self) -> NewGeometrySample2D<SRID> {
        let NewGeometrySampleG {
            name,
            point,
            linestring,
            polygon,
            multipoint,
            multiline,
            multipolygon,
            geometrycollection,
        } = self;
        NewGeometrySample2D {
            name,
            point,
            linestring,
            polygon,
            multipoint,
            multiline,
            multipolygon,
            geometrycollection,
        }
    }
}

impl<const SRID: u32> Into<NewGeometrySample3D<SRID>> for NewGeometrySampleG<SRID, PointZ<SRID>> {
    fn into(self) -> NewGeometrySample3D<SRID> {
        let NewGeometrySampleG {
            name,
            point,
            linestring,
            polygon,
            multipoint,
            multiline,
            multipolygon,
            geometrycollection,
        } = self;
        NewGeometrySample3D {
            name,
            point,
            linestring,
            polygon,
            multipoint,
            multiline,
            multipolygon,
            geometrycollection,
        }
    }
}

pub trait FromTuple {
    fn from_tuple(t: (f64, f64, f64)) -> Self;
}

impl<const SRID: u32> FromTuple for Point<SRID> {
    fn from_tuple(t: (f64, f64, f64)) -> Self {
        let (x, y, _) = t;
        Self { x, y }
    }
}

impl<const SRID: u32> FromTuple for PointZ<SRID> {
    fn from_tuple(t: (f64, f64, f64)) -> Self {
        let (x, y, z) = t;
        Self { x, y, z }
    }
}

impl<const SRID: u32, P: PointT<SRID> + FromTuple + Copy> NewGeometrySampleG<SRID, P> {
    pub fn mock(name: &str) -> Self {
        let points_iter = || {
            [
                (1., 3., 1.),
                (2., 1., 2.),
                (3., 4., 3.),
                (4., 2., 4.),
                (1., 3., 1.),
            ]
            .iter()
            .copied()
            .map(P::from_tuple)
        };
        let points: Vec<_> = points_iter().collect();
        let linestring = || points_iter().collect();
        let polygon = || vec![&points].into_iter().cloned().collect();
        let mut res = NewGeometrySampleG {
            name: name.to_string(),
            point: points[0],
            linestring: linestring(),
            polygon: polygon(),
            multipoint: points_iter().collect(),
            multiline: vec![linestring(), linestring()].into_iter().collect(),
            multipolygon: vec![polygon(), polygon()].into_iter().collect(),
            geometrycollection: GeometryCollection::new(),
        };
        let gc = &mut res.geometrycollection;

        {
            use GeometryContainer::*;
            gc.geometries.push(Point(res.point));
            gc.geometries.push(MultiPoint(res.multipoint.clone()));
            gc.geometries.push(MultiLineString(res.multiline.clone()));
            gc.geometries.push(MultiPolygon(res.multipolygon.clone()));
        }

        res
    }
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geometry_samples (id) {
        id -> Int4,
        name -> Text,
        point -> Geometry,
        linestring -> Geometry,
        polygon -> Geometry,
        multipoint -> Geometry,
        multiline -> Geometry,
        multipolygon -> Geometry,
        geometrycollection -> Geometry,
    }
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    distance_samples (id) {
        id -> Int4,
        name -> Text,
        point -> Geometry,
        polygon -> Geometry,
    }
}

fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url =
        env::var("POSTGIS_DIESEL_DATABASE_URL").expect("POSTGIS_DIESEL_DATABASE_URL not set");

    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn initialize() -> PgConnection {
    let mut conn = establish_connection();
    INIT.call_once(|| {
        let _ = diesel::sql_query("CREATE EXTENSION IF NOT EXISTS postgis").execute(&mut conn);
        let _ = diesel::sql_query("DROP TABLE geometry_samples").execute(&mut conn);
        let _ = diesel::sql_query("DROP TABLE distance_samples").execute(&mut conn);
        let _ = diesel::sql_query("DROP TABLE geometry_collection_samples").execute(&mut conn);
        let _ = diesel::sql_query(
            "CREATE TABLE geometry_samples
(
    id                SERIAL PRIMARY KEY,
    name              text,
    point             geometry NOT NULL,
    linestring        geometry NOT NULL,
    polygon           geometry NOT NULL,
    multipoint        geometry NOT NULL,
    multiline         geometry NOT NULL,
    multipolygon      geometry NOT NULL,
    geometrycollection geometry NOT NULL
)",
        )
        .execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE distance_samples
(
    id                SERIAL PRIMARY KEY,
    name              text,
    point             geometry(Point,4326) NOT NULL,
    polygon           geometry(Polygon,4326) NOT NULL
)",
        )
        .execute(&mut conn);
    });
    conn
}

pub fn new_point<const SRID: u32>(x: f64, y: f64) -> Point<SRID> {
    Point { x, y }
}

pub fn new_line<const SRID: u32>(points: Vec<(f64, f64)>) -> LineString<SRID, Point<SRID>> {
    let mut l_points = Vec::with_capacity(points.len());
    for p in points {
        l_points.push(Point { x: p.0, y: p.1 });
    }
    LineString { points: l_points }
}

pub fn new_line_z<const SRID: u32>(points: Vec<(f64, f64, f64)>) -> LineString<SRID, PointZ<SRID>> {
    points.into_iter().map(PointZ::from_tuple).collect()
}

pub fn new_line_4326(points: Vec<(f64, f64)>) -> gps::LineString {
    new_line(points)
}
