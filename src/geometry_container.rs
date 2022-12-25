use std::{fmt::Debug, io::Cursor};

use crate::{
    error::check_srid,
    ewkb::{EwkbSerializable, GeometryType, BIG_ENDIAN, self},
    geometrycollection::{read_geometry_collection_body, write_geometry_collection},
    linestring::{read_linestring_body, write_linestring},
    multiline::{read_multiline_body, write_multiline},
    multipoint::{read_multi_point_body, write_multi_point},
    multipolygon::{read_multi_polygon_body, write_multi_polygon},
    points::{read_point_coordinates, write_point},
    polygon::*,
    types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::sql_types::*;

impl<const SRID: u32, T> GeometryContainer<SRID, T>
where
    T: PointT<SRID> + Clone,
{
    pub fn dimension(&self) -> u32 {
        match self {
            GeometryContainer::Point(g) => g.dimension(),
            GeometryContainer::LineString(g) => g.dimension(),
            GeometryContainer::Polygon(g) => g.dimension(),
            GeometryContainer::MultiPoint(g) => g.dimension(),
            GeometryContainer::MultiLineString(g) => g.dimension(),
            GeometryContainer::MultiPolygon(g) => g.dimension(),
            GeometryContainer::GeometryCollection(g) => g.dimension(),
        }
    }
}

impl<const SRID: u32, T> ToSql<Geometry, Pg> for GeometryContainer<SRID, T>
where
    T: PointT<SRID> + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_geometry_container(self, out)
    }
}

impl<const SRID: u32, T> FromSql<Geometry, Pg> for GeometryContainer<SRID, T>
where
    T: PointT<SRID> + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_geometry_container::<SRID, BigEndian, T>(&mut r)
        } else {
            read_geometry_container::<SRID, LittleEndian, T>(&mut r)
        }
    }
}

pub fn read_geometry_container<const SRID: u32, T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<GeometryContainer<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let g_type = cursor.read_u32::<T>()?;
    if g_type & ewkb::SRID == ewkb::SRID {
        let srid = cursor.read_u32::<T>()?;
        check_srid(Some(srid), SRID)?;
    }
    Ok(match GeometryType::from(g_type) {
        GeometryType::Point => {
            GeometryContainer::Point(read_point_coordinates::<SRID, T, P>(cursor, g_type)?)
        }
        GeometryType::LineString => {
            GeometryContainer::LineString(read_linestring_body::<SRID, T, P>(g_type, cursor)?)
        }
        GeometryType::Polygon => {
            GeometryContainer::Polygon(read_polygon_body::<SRID, T, P>(g_type, cursor)?)
        }
        GeometryType::MultiPoint => {
            GeometryContainer::MultiPoint(read_multi_point_body::<SRID, T, P>(g_type, cursor)?)
        }
        GeometryType::MultiLineString => {
            GeometryContainer::MultiLineString(read_multiline_body::<SRID, T, P>(g_type, cursor)?)
        }
        GeometryType::MultiPolygon => {
            GeometryContainer::MultiPolygon(read_multi_polygon_body::<SRID, T, P>(g_type, cursor)?)
        }
        GeometryType::GeometryCollection => GeometryContainer::GeometryCollection(
            read_geometry_collection_body::<SRID, T, P>(cursor)?,
        ),
    })
}

pub fn write_geometry_container<const SRID: u32, T>(
    geometry_container: &GeometryContainer<SRID, T>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT<SRID> + EwkbSerializable + Clone,
{
    match geometry_container {
        GeometryContainer::Point(g) => write_point(g, out)?,
        GeometryContainer::LineString(g) => write_linestring(g, out)?,
        GeometryContainer::Polygon(g) => write_polygon(g, out)?,
        GeometryContainer::MultiPoint(g) => write_multi_point(g, out)?,
        GeometryContainer::MultiLineString(g) => write_multiline(g, out)?,
        GeometryContainer::MultiPolygon(g) => write_multi_polygon(g, out)?,
        GeometryContainer::GeometryCollection(g) => write_geometry_collection(g, out)?,
    };
    Ok(IsNull::No)
}
