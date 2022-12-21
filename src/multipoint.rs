use std::{fmt::Debug, io::Cursor, iter::FromIterator};

use crate::{
    error::check_srid,
    ewkb::{read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::{write_point, Dimension},
    types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::{points::read_point_coordinates, sql_types::*};

impl<const SRID: u32, P: PointT<SRID>> FromIterator<P> for MultiPoint<SRID, P> {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        let points = iter.into_iter().collect();
        Self { points }
    }
}

impl<const SRID: u32, T> MultiPoint<SRID, T>
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

impl<const SRID: u32, T> EwkbSerializable for MultiPoint<SRID, T>
where
    T: PointT<SRID>,
{
    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiPoint as u32;
        if let Some(point) = self.points.first() {
            g_type |= point.dimension();
        }
        g_type
    }
}

impl<const SRID: u32, T> FromSql<Geometry, Pg> for MultiPoint<SRID, T>
where
    T: PointT<SRID> + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multipoint::<SRID, BigEndian, T>(&mut r)
        } else {
            read_multipoint::<SRID, LittleEndian, T>(&mut r)
        }
    }
}

impl<const SRID: u32, T> ToSql<Geometry, Pg> for MultiPoint<SRID, T>
where
    T: PointT<SRID> + Debug + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_multi_point(self, out)
    }
}

pub fn write_multi_point<const SRID: u32, T>(
    multipoint: &MultiPoint<SRID, T>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT<SRID> + EwkbSerializable,
{
    write_ewkb_header(multipoint, Some(SRID), out)?;
    // size and points
    out.write_u32::<LittleEndian>(multipoint.points.len() as u32)?;
    for point in multipoint.points.iter() {
        write_point(point, out)?;
    }
    Ok(IsNull::No)
}

fn read_multipoint<const SRID: u32, T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<MultiPoint<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::MultiPoint, cursor)?;
    check_srid(g_header.srid, SRID)?;
    read_multi_point_body::<SRID, T, P>(g_header.g_type, cursor)
}

pub fn read_multi_point_body<const SRID: u32, T, P>(
    g_type: u32,
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<MultiPoint<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let len = cursor.read_u32::<T>()?;
    let mut points = Vec::with_capacity(len as usize);
    for _i in 0..len {
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        points.push(read_point_coordinates::<SRID, T, P>(cursor, g_type)?);
    }
    Ok(MultiPoint { points })
}
