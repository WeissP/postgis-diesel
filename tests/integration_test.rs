#[macro_use]
extern crate diesel;

use std::{env, sync::Once};

// use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{pg::PgConnection, Connection, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};
use dotenv::dotenv;

use postgis_diesel::{gps, operators::*, types::*};

static INIT: Once = Once::new();

#[derive(Insertable)]
#[diesel(table_name = geometry_samples)]
struct NewGeometrySample<const SRID: u32> {
    name: String,
    point: Point<SRID>,
    point_z: PointZ<SRID>,
    point_m: PointM<SRID>,
    point_zm: PointZM<SRID>,
    linestring: LineString<SRID, Point<SRID>>,
    polygon: Polygon<SRID, Point<SRID>>,
    multipoint: MultiPoint<SRID, Point<SRID>>,
    multiline: MultiLineString<SRID, Point<SRID>>,
    multipolygon: MultiPolygon<SRID, Point<SRID>>,
    gemetrycollection: GeometryCollection<SRID, Point<SRID>>,
}

#[derive(Insertable)]
#[diesel(table_name = distance_samples)]
struct NewDistanceSample<const SRID: u32> {
    name: String,
    point: Point<SRID>,
    polygon: Polygon<SRID, Point<SRID>>,
}

#[derive(Queryable, Debug, PartialEq)]
struct GeometrySample<const SRID: u32> {
    id: i32,
    name: String,
    point: Point<SRID>,
    point_z: PointZ<SRID>,
    point_m: PointM<SRID>,
    point_zm: PointZM<SRID>,
    linestring: LineString<SRID, Point<SRID>>,
    polygon: Polygon<SRID, Point<SRID>>,
    multipoint: MultiPoint<SRID, Point<SRID>>,
    multiline: MultiLineString<SRID, Point<SRID>>,
    multipolygon: MultiPolygon<SRID, Point<SRID>>,
    gemetrycollection: GeometryCollection<SRID, Point<SRID>>,
}

#[derive(Queryable, Debug, PartialEq)]
#[diesel(table_name = distance_samples)]
struct DistanceSample<const SRID: u32> {
    id: i32,
    name: String,
    point: Point<SRID>,
    polygon: Polygon<SRID, Point<SRID>>,
}

table! {
    use postgis_diesel::sql_types::*;
    use diesel::sql_types::*;
    geometry_samples (id) {
        id -> Int4,
        name -> Text,
        point -> Geometry,
        point_z -> Geometry,
        point_m -> Geometry,
        point_zm -> Geometry,
        linestring -> Geometry,
        polygon -> Geometry,
        multipoint -> Geometry,
        multiline -> Geometry,
        multipolygon -> Geometry,
        gemetrycollection -> Geometry,
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

fn initialize() -> PgConnection {
    let mut conn = establish_connection();
    INIT.call_once(|| {
        let _ = diesel::sql_query("CREATE EXTENSION IF NOT EXISTS postgis").execute(&mut conn);
        let _ = diesel::sql_query("DROP TABLE geometry_samples").execute(&mut conn);
        let _ = diesel::sql_query("DROP TABLE distance_samples").execute(&mut conn);

        let _ = diesel::sql_query(
            "CREATE TABLE geometry_samples
(
    id                SERIAL PRIMARY KEY,
    name              text,
    point             geometry(Point,4326) NOT NULL,
    point_z           geometry(PointZ,4326) NOT NULL,
    point_m           geometry(PointM,4326) NOT NULL,
    point_zm          geometry(PointZM,4326) NOT NULL,
    linestring        geometry(Linestring,4326) NOT NULL,
    polygon           geometry(Polygon,4326) NOT NULL,
    multipoint        geometry(MultiPoint,4326) NOT NULL,
    multiline         geometry(MultiLineString,4326) NOT NULL,
    multipolygon      geometry(MultiPolygon,4326) NOT NULL,
    gemetrycollection geometry(GeometryCollection,4326) NOT NULL
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

fn new_line<const SRID: u32>(points: Vec<(f64, f64)>) -> LineString<SRID, Point<SRID>> {
    let mut l_points = Vec::with_capacity(points.len());
    for p in points {
        l_points.push(Point { x: p.0, y: p.1 });
    }
    LineString { points: l_points }
}

fn new_line_4326(points: Vec<(f64, f64)>) -> gps::LineString {
    new_line(points)
}

fn new_point<const SRID: u32>(x: f64, y: f64) -> Point<SRID> {
    Point { x, y }
}

fn new_point_z<const SRID: u32>(x: f64, y: f64, z: f64) -> PointZ<SRID> {
    PointZ { x, y, z }
}

fn new_point_m<const SRID: u32>(x: f64, y: f64, m: f64) -> PointM<SRID> {
    PointM { x, y, m }
}

fn new_point_zm<const SRID: u32>(x: f64, y: f64, z: f64, m: f64) -> PointZM<SRID> {
    PointZM { x, y, z, m }
}

fn new_geometry_collection<const SRID: u32>() -> GeometryCollection<SRID, Point<SRID>> {
    let mut polygon = Polygon::new();
    polygon.add_points(&vec![
        new_point(72.0, 64.0),
        new_point(73.0, 65.0),
        new_point(71.0, 62.0),
        new_point(72.0, 64.0),
    ]);
    let mut multiline = MultiLineString::new();
    multiline.add_points(&vec![new_point(72.0, 64.0), new_point(73.0, 65.0)]);
    multiline.add_line();
    multiline.add_points(&vec![new_point(71.0, 62.0), new_point(72.0, 64.0)]);
    let mut multipolygon = MultiPolygon::new();
    multipolygon
        .add_empty_polygon()
        .add_points(&vec![
            new_point(72.0, 64.0),
            new_point(73.0, 65.0),
            new_point(71.0, 62.0),
            new_point(72.0, 64.0),
        ])
        .add_empty_polygon()
        .add_points(&vec![
            new_point(75.0, 64.0),
            new_point(74.0, 65.0),
            new_point(74.0, 62.0),
            new_point(75.0, 64.0),
        ]);
    let mut gc = GeometryCollection::new();
    gc.geometries
        .push(GeometryContainer::Point(new_point(73.0, 64.0)));
    gc.geometries
        .push(GeometryContainer::LineString(new_line(vec![
            (72.0, 64.0),
            (73.0, 64.0),
        ])));
    gc.geometries.push(GeometryContainer::Polygon(polygon));
    gc.geometries
        .push(GeometryContainer::MultiPoint(MultiPoint {
            points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
        }));
    gc.geometries
        .push(GeometryContainer::MultiLineString(multiline));
    gc.geometries
        .push(GeometryContainer::MultiPolygon(multipolygon));
    let mut inner_gc = GeometryCollection::new();
    inner_gc
        .geometries
        .push(GeometryContainer::Point(new_point(74.0, 64.0)));
    gc.geometries
        .push(GeometryContainer::GeometryCollection(inner_gc));
    gc
}

#[test]
fn srid_test() -> () {
    let mut conn = initialize();
    let sample: NewGeometrySample<4326> = NewGeometrySample {
        name: String::from("smoke_test"),
        point: new_point(72.0, 64.0),
        point_z: new_point_z(72.0, 64.0, 10.0),
        point_m: new_point_m(72.0, 64.0, 11.0),
        point_zm: new_point_zm(72.0, 64.0, 10.0, 11.0),
        linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
        polygon: Polygon::new(),
        multipoint: MultiPoint {
            points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
        },
        multiline: MultiLineString::new(),
        multipolygon: MultiPolygon::new(),
        gemetrycollection: new_geometry_collection(),
    };
    let point_from_db: GeometrySample<4326> = diesel::insert_into(geometry_samples::table)
        .values(&sample)
        .get_result(&mut conn)
        .expect("Error saving geometry sample");

    macro_rules! get_point {
        ($srid:expr) => {
            geometry_samples::table
                .filter(geometry_samples::id.eq(point_from_db.id))
                .select(geometry_samples::point)
                .first::<Point<$srid>>(&mut conn)
        };
    }

    let point_4326 = get_point!(4326).unwrap();
    assert_eq!(point_4326, sample.point);

    let point_9999 = get_point!(9999);

    assert!(point_9999.is_err());
    assert_eq!(
        point_9999.unwrap_err().to_string(),
        "Wrong SRID in database: Some(4326), Expected: 9999"
    );

    let _ =
        diesel::delete(geometry_samples::table.filter(geometry_samples::id.eq(point_from_db.id)))
            .execute(&mut conn);
}

#[test]
fn from_iter_test() -> () {
    use gps::*;
    let gen_points = || vec![Point::new(1., 0.), Point::new(0., 1.)];
    let multi_p: MultiPoint = gen_points().into_iter().collect();
    assert_eq!(multi_p.points, gen_points());

    let ls: LineString = gen_points().into_iter().collect();
    assert_eq!(ls.points, gen_points());

    let multi_ls : MultiLineString = vec![ls.clone()].into_iter().collect();
    assert_eq!(multi_ls.lines, vec![ls]);

    let poly : Polygon = vec![gen_points()].into_iter().collect();
    assert_eq!(poly.rings, vec![gen_points()]);

    let multi_poly : MultiPolygon = vec![poly.clone()].into_iter().collect();
    assert_eq!(multi_poly.polygons, vec![poly]);
}

#[test]
fn smoke_test() {
    let mut conn = initialize();
    let mut polygon = Polygon::new();
    polygon.add_points(&vec![
        new_point(72.0, 64.0),
        new_point(73.0, 65.0),
        new_point(71.0, 62.0),
        new_point(72.0, 64.0),
    ]);
    let mut multiline = MultiLineString::new();
    multiline.add_points(&vec![new_point(72.0, 64.0), new_point(73.0, 65.0)]);
    multiline.add_line();
    multiline.add_points(&vec![new_point(71.0, 62.0), new_point(72.0, 64.0)]);
    let mut multipolygon = MultiPolygon::new();
    multipolygon
        .add_empty_polygon()
        .add_points(&vec![
            new_point(72.0, 64.0),
            new_point(73.0, 65.0),
            new_point(71.0, 62.0),
            new_point(72.0, 64.0),
        ])
        .add_empty_polygon()
        .add_points(&vec![
            new_point(75.0, 64.0),
            new_point(74.0, 65.0),
            new_point(74.0, 62.0),
            new_point(75.0, 64.0),
        ]);
    let sample = NewGeometrySample {
        name: String::from("smoke_test"),
        point: new_point(72.0, 64.0),
        point_z: new_point_z(72.0, 64.0, 10.0),
        point_m: new_point_m(72.0, 64.0, 11.0),
        point_zm: new_point_zm(72.0, 64.0, 10.0, 11.0),
        linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
        polygon: polygon,
        multipoint: MultiPoint {
            points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
        },
        multiline: multiline,
        multipolygon: multipolygon,
        gemetrycollection: new_geometry_collection(),
    };
    let point_from_db: GeometrySample<4326> = diesel::insert_into(geometry_samples::table)
        .values(&sample)
        .get_result(&mut conn)
        .expect("Error saving geometry sample");

    assert_eq!(sample.name, point_from_db.name);
    assert_eq!(sample.point, point_from_db.point);
    assert_eq!(sample.point_z, point_from_db.point_z);
    assert_eq!(sample.point_m, point_from_db.point_m);
    assert_eq!(sample.point_zm, point_from_db.point_zm);
    assert_eq!(sample.linestring, point_from_db.linestring);
    assert_eq!(sample.polygon, point_from_db.polygon);
    assert_eq!(sample.multipoint, point_from_db.multipoint);
    assert_eq!(sample.multiline, point_from_db.multiline);
    assert_eq!(sample.multipolygon, point_from_db.multipolygon);
    assert_eq!(sample.gemetrycollection, point_from_db.gemetrycollection);

    let _ =
        diesel::delete(geometry_samples::table.filter(geometry_samples::id.eq(point_from_db.id)))
            .execute(&mut conn);
}

#[test]
fn distance_2d_test() {
    let mut conn = initialize();
    let m_point = Point::new(37.618423, 55.751244);
    let v_point = Point::new(39.2088823, 51.6754966);
    let mut m_polygon = Polygon::new();
    m_polygon
        .add_ring()
        .add_point(Point::new(34.22922934177507, 57.98223781625711))
        .add_point(Point::new(34.22922934177507, 57.08802327022599))
        .add_point(Point::new(36.18872506469961, 57.08802327022599))
        .add_point(Point::new(36.18872506469961, 57.98223781625711))
        .add_point(Point::new(34.22922934177507, 57.98223781625711));
    let mut v_polygon = Polygon::new();
    v_polygon
        .add_ring()
        .add_point(Point::new(39.732117473214146, 53.3129374517664))
        .add_point(Point::new(39.732117473214146, 52.658032976474146))
        .add_point(Point::new(40.79294127961202, 52.658032976474146))
        .add_point(Point::new(40.79294127961202, 53.3129374517664))
        .add_point(Point::new(39.732117473214146, 53.3129374517664));

    let m_sample: NewDistanceSample<4326> = NewDistanceSample {
        name: String::from("Moscow"),
        point: m_point,
        polygon: m_polygon,
    };
    let v_sample: NewDistanceSample<4326> = NewDistanceSample {
        name: String::from("Voronezh"),
        point: v_point,
        polygon: v_polygon,
    };
    let records = vec![m_sample, v_sample];
    let r = diesel::insert_into(distance_samples)
        .values(records)
        .execute(&mut conn);
    assert_eq!(true, r.is_ok(), "can't insert data");

    use self::distance_samples::dsl::*;

    let found_sample: DistanceSample<4326> = distance_samples
        .order_by(distance_2d(
            point,
            Point::<4326>::new(38.495490805803115, 52.62169972015738),
        ))
        .limit(1)
        .get_result(&mut conn)
        .expect("nothing found");
    assert_eq!("Voronezh", found_sample.name);
    let found_sample: DistanceSample<4326> = distance_samples
        .order_by(distance_2d(
            point,
            Point::<4326>::new(37.184130751959486, 55.988642876744535),
        ))
        .limit(1)
        .get_result(&mut conn)
        .expect("nothing found");
    assert_eq!("Moscow", found_sample.name);
}

macro_rules! operator_test {
    ($t:ident; $f:ident; $find:expr; $not_find:expr) => {
        #[test]
        fn $t() {
            let mut conn = initialize();
            let mut polygon = Polygon::new();
            polygon.add_points(&vec![
                new_point(72.0, 64.0),
                new_point(73.0, 65.0),
                new_point(71.0, 62.0),
                new_point(72.0, 64.0),
            ]);
            let mut multiline = MultiLineString::new();
            multiline.add_points(&vec![new_point(72.0, 64.0), new_point(73.0, 65.0)]);
            multiline.add_line();
            multiline.add_points(&vec![new_point(71.0, 62.0), new_point(72.0, 64.0)]);
            let mut multipolygon = MultiPolygon::new();
            multipolygon
                .add_empty_polygon()
                .add_points(&vec![
                    new_point(72.0, 64.0),
                    new_point(73.0, 65.0),
                    new_point(71.0, 62.0),
                    new_point(72.0, 64.0),
                ])
                .add_empty_polygon()
                .add_points(&vec![
                    new_point(75.0, 64.0),
                    new_point(74.0, 65.0),
                    new_point(74.0, 62.0),
                    new_point(75.0, 64.0),
                ]);
            let sample = NewGeometrySample {
                name: String::from(stringify!($t)),
                point: new_point(71.0, 63.0),
                point_z: new_point_z(72.0, 64.0, 10.0),
                point_m: new_point_m(72.0, 64.0, 11.0),
                point_zm: new_point_zm(72.0, 64.0, 10.0, 11.0),
                linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
                polygon: polygon,
                multipoint: MultiPoint {
                    points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
                },
                multiline: multiline,
                multipolygon: multipolygon,
                gemetrycollection: new_geometry_collection(),
            };
            let _ = diesel::insert_into(geometry_samples::table)
                .values(&sample)
                .get_result::<GeometrySample<4326>>(&mut conn)
                .expect("Error saving geometry sample");
            let found = geometry_samples::table
                .filter($f(geometry_samples::linestring, $find))
                .filter(geometry_samples::name.eq(stringify!($t)))
                .get_result::<GeometrySample<4326>>(&mut conn)
                .expect("Error getting geometry");

            assert_eq!(sample.point, found.point);
            assert_eq!(sample.linestring, found.linestring);

            let not_found: QueryResult<GeometrySample<4326>> = geometry_samples::table
                .filter($f(geometry_samples::linestring, $not_find))
                .filter(geometry_samples::name.eq(stringify!($t)))
                .get_result(&mut conn);
            assert_eq!(not_found, Err(diesel::result::Error::NotFound));
        }
    };
}

// line (72.0, 64.0) --> (73.0, 64.0)
operator_test!(intersects_2d_test; intersects_2d; new_line_4326(vec![(72.0, 63.0), (72.0, 65.0)]); new_line_4326(vec![(71.0, 63.0), (71.0, 65.0)]));
operator_test!(overlap_or_left_test; overlaps_or_left; new_line_4326(vec![(74.0, 63.0), (74.0, 65.0)]); new_line_4326(vec![(71.0, 63.0), (71.0, 65.0)]));
operator_test!(overlap_or_left_overlaps_test; overlaps_or_left; new_line_4326(vec![(72.5, 64.0), (74.0, 64.0)]); new_line_4326(vec![(71.0, 63.0), (71.0, 65.0)]));
operator_test!(overlap_or_below_test; overlaps_or_below; new_line_4326(vec![(72.0, 65.0), (73.0, 65.0)]); new_line_4326(vec![(71.0, 62.0), (71.0, 62.0)]));
operator_test!(overlap_or_below_overlaps_test; overlaps_or_below; new_line_4326(vec![(72.5, 64.0), (74.0, 64.0)]); new_line_4326(vec![(71.0, 62.0), (71.0, 62.0)]));
operator_test!(overlap_or_right_test; overlaps_or_right; new_line_4326(vec![(70.0, 64.0), (71.0, 64.0)]); new_line_4326(vec![(74.0, 62.0), (75.0, 62.0)]));
operator_test!(overlap_or_right_overlaps_test; overlaps_or_right; new_line_4326(vec![(71.0, 64.0), (73.0, 64.0)]); new_line_4326(vec![(74.0, 62.0), (75.0, 62.0)]));
operator_test!(stricly_left_test; strictly_left; new_line_4326(vec![(74.0, 63.0), (74.0, 65.0)]); new_line_4326(vec![(71.0, 63.0), (71.0, 65.0)]));
operator_test!(stricly_below_test; strictly_below; new_line_4326(vec![(72.0, 65.0), (73.0, 65.0)]); new_line_4326(vec![(71.0, 62.0), (71.0, 62.0)]));
operator_test!(g_same_test; g_same; new_line_4326(vec![(72.0, 64.0), (73.0, 64.0)]); new_line_4326(vec![(73.0, 64.0), (72.0, 64.0)]));
operator_test!(strictly_right_test; strictly_right; new_line_4326(vec![(70.0, 64.0), (71.0, 64.0)]); new_line_4326(vec![(74.0, 62.0), (75.0, 62.0)]));
operator_test!(contained_by_test; contained_by; new_line_4326(vec![(71.0, 64.0), (74.0, 64.0)]); new_line_4326(vec![(74.0, 62.0), (75.0, 62.0)]));
operator_test!(overlap_or_above_test; overlaps_or_above; new_line_4326(vec![(72.0, 63.0), (73.0, 63.0)]); new_line_4326(vec![(71.0, 65.0), (71.0, 65.0)]));
operator_test!(overlap_or_above_overlaps_test; overlaps_or_above; new_line_4326(vec![(72.5, 64.0), (74.0, 64.0)]); new_line_4326(vec![(71.0, 65.0), (71.0, 65.0)]));
operator_test!(strictly_above_test; strictly_above; new_line_4326(vec![(72.0, 63.0), (73.0, 63.0)]); new_line_4326(vec![(71.0, 65.0), (71.0, 65.0)]));
operator_test!(contains_test; contains; new_line_4326(vec![(72.1, 64.0), (72.9, 64.0)]); new_line_4326(vec![(71.0, 64.0), (75.0, 64.0)]));
operator_test!(bb_same_test; bb_same; new_line_4326(vec![(73.0, 64.0), (72.0, 64.0)]); new_line_4326(vec![(71.0, 64.0), (75.0, 64.0)]));
