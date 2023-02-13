use crate::types;

pub const SRID: u32 = 4326;

pub type MultiPointC<Pt> = types::MultiPoint<SRID, Pt>;
pub type LineStringC<Pt> = types::LineString<SRID, Pt>;
pub type MultiLineStringC<Pt> = types::MultiLineString<SRID, Pt>;
pub type PolygonC<Pt> = types::Polygon<SRID, Pt>;
pub type MultiPolygonC<Pt> = types::MultiPolygon<SRID, Pt>;
pub type GeometryContainerC<Pt> = types::GeometryContainer<SRID, Pt>;
pub type GeometryCollectionC<Pt> = types::GeometryCollection<SRID, Pt>;

pub type Point = types::Point<SRID>;
pub type MultiPoint = types::MultiPoint<SRID, Point>;
pub type LineString = types::LineString<SRID, Point>;
pub type MultiLineString = types::MultiLineString<SRID, Point>;
pub type Polygon = types::Polygon<SRID, Point>;
pub type MultiPolygon = types::MultiPolygon<SRID, Point>;
pub type GeometryContainer = types::GeometryContainer<SRID, Point>;
pub type GeometryCollection = types::GeometryCollection<SRID, Point>;

pub type PointZ = types::PointZ<SRID>;
pub type MultiPointZ = types::MultiPoint<SRID, PointZ>;
pub type LineStringZ = types::LineString<SRID, PointZ>;
pub type MultiLineStringZ = types::MultiLineString<SRID, PointZ>;
pub type PolygonZ = types::Polygon<SRID, PointZ>;
pub type MultiPolygonZ = types::MultiPolygon<SRID, PointZ>;
pub type GeometryContainerZ = types::GeometryContainer<SRID, PointZ>;
pub type GeometryCollectionZ = types::GeometryCollection<SRID, PointZ>;
