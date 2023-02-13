#[macro_use]
extern crate diesel;

mod common;
use common::*;
use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};
use postgis_diesel::{operators::*, types::*};

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
            let sample = NewGeometrySample2D::<4326> {
                name: String::from(stringify!($t)),
                point: new_point(71.0, 63.0),
                linestring: new_line(vec![(72.0, 64.0), (73.0, 64.0)]),
                polygon: polygon,
                multipoint: MultiPoint {
                    points: vec![new_point(72.0, 64.0), new_point(73.0, 64.0)],
                },
                multiline: multiline,
                multipolygon: multipolygon,
                geometrycollection: GeometryCollection::new(),
            };
            let _ = diesel::insert_into(geometry_samples::table)
                .values(&sample)
                .get_result::<GeometrySample<4326, Point<4326>>>(&mut conn)
                .expect("Error saving geometry sample");
            let found = geometry_samples::table
                .filter($f(geometry_samples::linestring, $find))
                .filter(geometry_samples::name.eq(stringify!($t)))
                .get_result::<GeometrySample<4326, Point<4326>>>(&mut conn)
                .expect("Error getting geometry");

            assert_eq!(sample.point, found.point);
            assert_eq!(sample.linestring, found.linestring);

            let not_found: QueryResult<GeometrySample<4326, Point<4326>>> = geometry_samples::table
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
