#[macro_use]
extern crate diesel;

mod common;
use common::*;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use postgis_diesel::types::*;

#[test]
fn srid_test() -> () {
    let mut conn = initialize();
    let sample: NewGeometrySample2D<4326> = NewGeometrySampleG::mock("srid").into();
    let point_from_db: GeometrySample<4326, Point<4326>> =
        diesel::insert_into(geometry_samples::table)
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
fn smoke_test() {
    let mut conn = initialize();

    let sample: NewGeometrySample2D<4326> = NewGeometrySampleG::mock("smoke_test").into();

    let point_from_db: GeometrySample<4326, Point<4326>> =
        diesel::insert_into(geometry_samples::table)
            .values(&sample)
            .get_result(&mut conn)
            .expect("Error saving geometry sample");

    assert_eq!(sample.name, point_from_db.name);
    assert_eq!(sample.point, point_from_db.point);
    assert_eq!(sample.linestring, point_from_db.linestring);
    assert_eq!(sample.polygon, point_from_db.polygon);
    assert_eq!(sample.multipoint, point_from_db.multipoint);
    assert_eq!(sample.multiline, point_from_db.multiline);
    assert_eq!(sample.multipolygon, point_from_db.multipolygon);
    assert_eq!(sample.geometrycollection, point_from_db.geometrycollection);

    let _ =
        diesel::delete(geometry_samples::table.filter(geometry_samples::id.eq(point_from_db.id)))
            .execute(&mut conn);

    let sample_3d: NewGeometrySample3D<4326> = NewGeometrySampleG::mock("smoke_test_3d").into();

    let point_from_db: GeometrySample<4326, PointZ<4326>> =
        diesel::insert_into(geometry_samples::table)
            .values(&sample_3d)
            .get_result(&mut conn)
            .expect("Error saving geometry sample");

    assert_eq!(sample_3d.name, point_from_db.name);
    assert_eq!(sample_3d.point, point_from_db.point);
    assert_eq!(sample_3d.linestring, point_from_db.linestring);
    assert_eq!(sample_3d.polygon, point_from_db.polygon);
    assert_eq!(sample_3d.multipoint, point_from_db.multipoint);
    assert_eq!(sample_3d.multiline, point_from_db.multiline);
    assert_eq!(sample_3d.multipolygon, point_from_db.multipolygon);
    assert_eq!(
        sample_3d.geometrycollection,
        point_from_db.geometrycollection
    );

    let _ =
        diesel::delete(geometry_samples::table.filter(geometry_samples::id.eq(point_from_db.id)))
            .execute(&mut conn);
}

#[test]
fn geo_container_test() {
    let mut conn = initialize();
    let sample: NewGeometrySample3D<4326> = NewGeometrySampleG::mock("geo_container_test").into();

    let point_from_db: GeometrySample<4326, PointZ<4326>> =
        diesel::insert_into(geometry_samples::table)
            .values(&sample)
            .get_result(&mut conn)
            .expect("Error saving geometry sample");

    macro_rules! get {
        ($field:ident) => {
            geometry_samples::table
                .filter(geometry_samples::id.eq(point_from_db.id))
                .select(geometry_samples::$field)
                .first::<GeometryContainer<4326, PointZ<4326>>>(&mut conn)
                .expect("could not get from sample")
        };
    }

    macro_rules! check {
        ($field:ident, $container:ident) => {
            assert_eq!(get!($field), GeometryContainer::$container(sample.$field));
        };
    }

    assert_eq!(sample.name, point_from_db.name);
    check!(point, Point);
    check!(linestring, LineString);
    check!(polygon, Polygon);
    check!(multipoint, MultiPoint);
    check!(multiline, MultiLineString);
    check!(multipolygon, MultiPolygon);
    check!(geometrycollection, GeometryCollection);

    let _ =
        diesel::delete(geometry_samples::table.filter(geometry_samples::id.eq(point_from_db.id)))
            .execute(&mut conn);
}
