use crate::types;

pub type Point = types::Point<4326>;
pub type MultiPoint = types::MultiPoint<4326, Point>;
pub type LineString = types::LineString<4326, Point>;
pub type MultiLineString = types::MultiLineString<4326, Point>;
pub type Polygon = types::Polygon<4326, Point>;
pub type MultiPolygon = types::MultiPolygon<4326, Point>;
pub type GeometryContainer = types::GeometryContainer<4326, Point>;
pub type GeometryCollection = types::GeometryCollection<4326, Point>;

pub type PointZ = types::PointZ<4326>;
pub type MultiPointZ = types::MultiPoint<4326, PointZ>;
pub type LineStringZ = types::LineString<4326, PointZ>;
pub type MultiLineStringZ = types::MultiLineString<4326, PointZ>;
pub type PolygonZ = types::Polygon<4326, PointZ>;
pub type MultiPolygonZ = types::MultiPolygon<4326, PointZ>;
pub type GeometryContainerZ = types::GeometryContainer<4326, PointZ>;
pub type GeometryCollectionZ = types::GeometryCollection<4326, PointZ>;
