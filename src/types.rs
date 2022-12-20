use std::fmt;

use crate::sql_types::Geometry;

/// Error which may be returned if point cinstructed without required fields or has some unexpected fields for type.
/// ```
/// use postgis_diesel::types::{PointZ, Point, PointConstructorError, PointT};
/// let point = PointZ::<4326>::new_point(72.0, 63.0, None, None);
/// assert!(point.is_err());
/// assert_eq!(Result::Err(PointConstructorError{reason:"Z is not defined, but mandatory for PointZ".to_string()}), point);
/// let point= Point::<4326>::new_point(72.0, 63.0, Some(10.0), None);
/// assert!(point.is_err());
/// assert_eq!(Result::Err(PointConstructorError{reason:"unexpectedly defined Z Some(10.0) or M None for Point".to_string()}), point);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PointConstructorError {
    pub reason: String,
}

impl fmt::Display for PointConstructorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "can't construct point: {}", self.reason)
    }
}

impl std::error::Error for PointConstructorError {}

/// Use that structure in `Insertable` or `Queryable` struct if you work with Point geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::Point;
/// #[derive(Queryable)]
/// struct QueryablePointExample {
///     id: i32,
///     point: Point<4326>,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct Point<const SRID: u32> {
    pub x: f64,
    pub y: f64,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with PointZ geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::PointZ;
/// #[derive(Queryable)]
/// struct QueryablePointZExample {
///     id: i32,
///     point: PointZ<4326>,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct PointZ<const SRID: u32> {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with PointM geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::PointM;
/// #[derive(Queryable)]
/// struct QueryablePointMExample {
///     id: i32,
///     point: PointM<4326>,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct PointM<const SRID: u32> {
    pub x: f64,
    pub y: f64,
    pub m: f64,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with PointZM geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::PointZM;
/// #[derive(Queryable)]
/// struct QueryablePointZMExample {
///     id: i32,
///     point: PointZM<4326>,
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct PointZM<const SRID: u32> {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m: f64,
}

pub trait PointT<const SRID: u32> {
    fn new_point(
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
    ) -> Result<Self, PointConstructorError>
    where
        Self: Sized;
    fn get_x(&self) -> f64;
    fn get_y(&self) -> f64;
    fn get_z(&self) -> Option<f64>;
    fn get_m(&self) -> Option<f64>;
    fn dimension(&self) -> u32;
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with MultiPoint geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{MultiPoint,Point};
/// #[derive(Queryable)]
/// struct QueryableMultiPointExample {
///     id: i32,
///     multipoint: MultiPoint<4326, Point<4326>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiPoint<const SRID: u32, T: PointT<SRID>> {
    pub points: Vec<T>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with LineString geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{LineString,Point};
/// #[derive(Queryable)]
/// struct QueryableLineStringExample {
///     id: i32,
///     linestring: LineString<4326, Point<4326>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct LineString<const SRID: u32, T: PointT<SRID>> {
    pub points: Vec<T>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with MultiLineString geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{MultiLineString, LineString,Point};
/// #[derive(Queryable)]
/// struct QueryableMultiLineStringExample {
///     id: i32,
///     multilinestring: MultiLineString<4326, Point<4326>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiLineString<const SRID: u32, T: PointT<SRID>> {
    pub lines: Vec<LineString<SRID, T>>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with Polygon geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{Polygon,Point};
/// #[derive(Queryable)]
/// struct QueryablePolygonExample {
///     id: i32,
///     polygon: Polygon<4326, Point<4326>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct Polygon<const SRID: u32, T: PointT<SRID>> {
    pub rings: Vec<Vec<T>>,
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with MultiPolygon geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{MultiPolygon, Polygon,Point};
/// #[derive(Queryable)]
/// struct QueryableMultiPolygonExample {
///     id: i32,
///     multipolygon: MultiPolygon<4326,  Point<4326>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct MultiPolygon<const SRID: u32, T: PointT<SRID>> {
    pub polygons: Vec<Polygon<SRID, T>>,
}

#[derive(Clone, Debug, PartialEq, FromSqlRow)]
pub enum GeometryContainer<const SRID: u32, T: PointT<SRID>> {
    Point(T),
    LineString(LineString<SRID, T>),
    Polygon(Polygon<SRID, T>),
    MultiPoint(MultiPoint<SRID, T>),
    MultiLineString(MultiLineString<SRID, T>),
    MultiPolygon(MultiPolygon<SRID, T>),
    GeometryCollection(GeometryCollection<SRID, T>),
}

/// Use that structure in `Insertable` or `Queryable` struct if you work with GeometryCollection geometry.
/// ```
/// #[macro_use] extern crate diesel;
/// use postgis_diesel::types::{GeometryCollection, GeometryContainer, Point};
/// #[derive(Queryable)]
/// struct QueryableGeometryCollectionExample {
///     id: i32,
///     geometrycollection: GeometryCollection<4326, Point<4326>>,
/// }
/// ```
#[derive(Clone, Debug, PartialEq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Geometry)]
pub struct GeometryCollection<const SRID: u32, T: PointT<SRID>> {
    pub geometries: Vec<GeometryContainer<SRID, T>>,
}
