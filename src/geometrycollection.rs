use std::{fmt::Debug, io::Cursor};

use crate::{
    error::check_srid,
    ewkb::{read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    geometry_container::{read_geometry_container, write_geometry_container},
    points::Dimension,
    sql_types::*,
    types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

impl<const SRID: u32, T> GeometryCollection<SRID, T>
where
    T: PointT<SRID> + Clone,
{
    pub fn new() -> Self {
        Self {
            geometries: Vec::new(),
        }
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(geometry) = self.geometries.first() {
            dimension |= geometry.dimension();
        }
        dimension
    }
}

impl<const SRID: u32, T> EwkbSerializable for GeometryCollection<SRID, T>
where
    T: PointT<SRID> + Clone,
{
    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::GeometryCollection as u32;
        if let Some(polygon) = self.geometries.first() {
            g_type |= polygon.dimension();
        }
        g_type
    }
}

impl<const SRID: u32, T> ToSql<Geometry, Pg> for GeometryCollection<SRID, T>
where
    T: PointT<SRID> + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_geometry_collection(self, out)
    }
}

impl<const SRID: u32, T> FromSql<Geometry, Pg> for GeometryCollection<SRID, T>
where
    T: PointT<SRID> + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_geometry_collection::<SRID, BigEndian, T>(&mut r)
        } else {
            read_geometry_collection::<SRID, LittleEndian, T>(&mut r)
        }
    }
}

pub fn write_geometry_collection<const SRID: u32, T>(
    geometrycollection: &GeometryCollection<SRID, T>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT<SRID> + EwkbSerializable + Clone,
{
    write_ewkb_header(geometrycollection, Some(SRID), out)?;
    out.write_u32::<LittleEndian>(geometrycollection.geometries.len() as u32)?;
    for g_container in geometrycollection.geometries.iter() {
        write_geometry_container(g_container, out)?;
    }
    Ok(IsNull::No)
}

fn read_geometry_collection<const SRID: u32, T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<GeometryCollection<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::GeometryCollection, cursor)?;
    check_srid(g_header.srid, SRID)?;
    read_geometry_collection_body::<SRID, T, P>(cursor)
}

pub fn read_geometry_collection_body<const SRID: u32, T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<GeometryCollection<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let geometries_n = cursor.read_u32::<T>()?;
    let mut g_collection = GeometryCollection::new();
    for _i in 0..geometries_n {
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        let g_container = read_geometry_container::<SRID, T, P>(cursor)?;
        g_collection.geometries.push(g_container);
    }
    Ok(g_collection)
}
