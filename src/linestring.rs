use std::{fmt::Debug, io::Cursor};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::{read_point_coordinates, write_point_coordinates, Dimension},
    sql_types::*,
    types::{LineString, PointT}, error::check_srid,
};

impl<const SRID: u32, T> EwkbSerializable for LineString<SRID, T>
where
    T: PointT<SRID>,
{
    fn geometry_type(&self) -> u32 {
        GeometryType::LineString as u32 | self.dimension()
    }
}

impl<const SRID: u32, T> LineString<SRID, T>
where
    T: PointT<SRID>,
{
    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(point) = self.points.first() {
            dimension |= point.dimension();
        }
        dimension
    }
}

impl<const SRID: u32, T> FromSql<Geometry, Pg> for LineString<SRID, T>
where
    T: PointT<SRID> + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_linestring::<SRID, BigEndian, T>(&mut r)
        } else {
            read_linestring::<SRID, LittleEndian, T>(&mut r)
        }
    }
}

impl<const SRID: u32, T> ToSql<Geometry, Pg> for LineString<SRID, T>
where
    T: PointT<SRID> + Debug + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_linestring(self, out)
    }
}

pub fn write_linestring<const SRID: u32, T>(
    linestring: &LineString<SRID, T>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT<SRID> + EwkbSerializable,
{
    write_ewkb_header(linestring, Some(SRID), out)?;
    // size and points
    out.write_u32::<LittleEndian>(linestring.points.len() as u32)?;
    for point in linestring.points.iter() {
        write_point_coordinates(point, out)?;
    }
    Ok(IsNull::No)
}

fn read_linestring<const SRID: u32, T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<LineString<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::LineString, cursor)?;
    check_srid(g_header.srid, SRID)?;
    read_linestring_body::<SRID, T, P>(g_header.g_type, cursor)
}

pub fn read_linestring_body<const SRID: u32, T, P>(
    g_type: u32,
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<LineString<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let len = cursor.read_u32::<T>()?;
    let mut points = Vec::with_capacity(len as usize);
    for _i in 0..len {
        points.push(read_point_coordinates::<SRID, T, P>(cursor, g_type)?);
    }
    Ok(LineString { points })
}
