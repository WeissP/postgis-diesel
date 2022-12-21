use std::{fmt::Debug, io::Cursor, iter::FromIterator};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::{
    ewkb::{read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    linestring::write_linestring,
    points::Dimension,
    types::{LineString, MultiLineString, PointT}, error::check_srid,
};

use crate::{points::read_point_coordinates, sql_types::*};

impl<const SRID: u32, P: PointT<SRID>> FromIterator<LineString<SRID, P>> for MultiLineString<SRID, P> {
    fn from_iter<T: IntoIterator<Item = LineString<SRID, P>>>(iter: T) -> Self {
        let lines = iter.into_iter().collect();
        Self { lines }
    }
}

impl<const SRID: u32, T> MultiLineString<SRID, T>
where
    T: PointT<SRID> + Clone,
{
    pub fn new() -> Self {
        MultiLineString { lines: Vec::new() }
    }

    pub fn add_line<'a>(&'a mut self) -> &mut Self {
        self.lines.push(LineString {
            points: Vec::new(),
        });
        self
    }

    pub fn add_point<'a>(&'a mut self, point: T) -> &mut Self {
        if self.lines.last().is_none() {
            self.add_line();
        }
        self.lines.last_mut().unwrap().points.push(point);
        self
    }

    pub fn add_points<'a>(&'a mut self, points: &[T]) -> &mut Self {
        if self.lines.last().is_none() {
            self.add_line();
        }
        let last = self.lines.last_mut().unwrap();
        for point in points {
            last.points.push(point.to_owned());
        }
        self
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(line) = self.lines.first() {
            dimension |= line.dimension();
        }
        dimension
    }
}

impl<const SRID: u32, T> EwkbSerializable for MultiLineString<SRID, T>
where
    T: PointT<SRID>,
{
    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiLineString as u32;
        if let Some(line) = self.lines.first() {
            g_type |= line.dimension();
        }
        g_type
    }
}

impl<const SRID: u32, T> ToSql<Geometry, Pg> for MultiLineString<SRID, T>
where
    T: PointT<SRID> + Debug + PartialEq + EwkbSerializable + Clone,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_multiline(self, out)
    }
}

impl<const SRID: u32, T> FromSql<Geometry, Pg> for MultiLineString<SRID, T>
where
    T: PointT<SRID> + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multiline::<SRID, BigEndian, T>(&mut r)
        } else {
            read_multiline::<SRID, LittleEndian, T>(&mut r)
        }
    }
}

pub fn write_multiline<const SRID: u32, T>(
    multiline: &MultiLineString<SRID, T>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT<SRID> + EwkbSerializable + Clone,
{
    write_ewkb_header(multiline, Some(SRID), out)?;
    // number of lines
    out.write_u32::<LittleEndian>(multiline.lines.len() as u32)?;
    for line in multiline.lines.iter() {
        write_linestring(line, out)?;
    }
    Ok(IsNull::No)
}

fn read_multiline<const SRID: u32, T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<MultiLineString<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::MultiLineString, cursor)?;
    check_srid(g_header.srid, SRID)?;
    read_multiline_body::<SRID, T, P>(g_header.g_type, cursor)
}

pub fn read_multiline_body<const SRID: u32, T, P>(
    g_type: u32,
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<MultiLineString<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let lines_n = cursor.read_u32::<T>()?;
    let mut multiline = MultiLineString::new();
    for _i in 0..lines_n {
        multiline.add_line();
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        let points_n = cursor.read_u32::<T>()?;
        for _p in 0..points_n {
            multiline.add_point(read_point_coordinates::<SRID, T, P>(cursor, g_type)?);
        }
    }
    Ok(multiline)
}
