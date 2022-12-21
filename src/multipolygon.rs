use std::{fmt::Debug, io::Cursor, iter::FromIterator};

use crate::{
    error::check_srid,
    ewkb::{read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::Dimension,
    polygon::{read_polygon_body, write_polygon},
    types::*,
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};

use crate::sql_types::*;

impl<const SRID: u32, P: PointT<SRID>> FromIterator<Polygon<SRID, P>> for MultiPolygon<SRID, P> {
    fn from_iter<T: IntoIterator<Item = Polygon<SRID, P>>>(iter: T) -> Self {
        let polygons = iter.into_iter().collect();
        Self { polygons }
    }
}

impl<const SRID: u32, T> MultiPolygon<SRID, T>
where
    T: PointT<SRID> + Clone,
{
    pub fn new() -> Self {
        MultiPolygon {
            polygons: Vec::new(),
        }
    }

    pub fn add_empty_polygon<'a>(&'a mut self) -> &mut Self {
        self.polygons.push(Polygon { rings: Vec::new() });
        self
    }

    pub fn add_point<'a>(&'a mut self, point: T) -> &mut Self {
        if self.polygons.last().is_none() {
            self.add_empty_polygon();
        }
        self.polygons.last_mut().unwrap().add_point(point);
        self
    }

    pub fn add_points<'a>(&'a mut self, points: &[T]) -> &mut Self {
        if self.polygons.last().is_none() {
            self.add_empty_polygon();
        }
        let last = self.polygons.last_mut().unwrap();
        for point in points {
            last.add_point(point.to_owned());
        }
        self
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(polygon) = self.polygons.first() {
            dimension |= polygon.dimension();
        }
        dimension
    }
}

impl<const SRID: u32, T> EwkbSerializable for MultiPolygon<SRID, T>
where
    T: PointT<SRID> + Clone,
{
    fn geometry_type(&self) -> u32 {
        let mut g_type = GeometryType::MultiPolygon as u32;
        if let Some(polygon) = self.polygons.first() {
            g_type |= polygon.dimension();
        }
        g_type
    }
}

impl<const SRID: u32, T> ToSql<Geometry, Pg> for MultiPolygon<SRID, T>
where
    T: PointT<SRID> + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_multi_polygon(self, out)
    }
}

impl<const SRID: u32, T> FromSql<Geometry, Pg> for MultiPolygon<SRID, T>
where
    T: PointT<SRID> + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_multi_polygon::<SRID, BigEndian, T>(&mut r)
        } else {
            read_multi_polygon::<SRID, LittleEndian, T>(&mut r)
        }
    }
}

pub fn write_multi_polygon<const SRID: u32, T>(
    multipolygon: &MultiPolygon<SRID, T>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT<SRID> + EwkbSerializable + Clone,
{
    write_ewkb_header(multipolygon, Some(SRID), out)?;
    // number of polygons
    out.write_u32::<LittleEndian>(multipolygon.polygons.len() as u32)?;
    for polygon in multipolygon.polygons.iter() {
        write_polygon(polygon, out)?;
    }
    Ok(IsNull::No)
}

fn read_multi_polygon<const SRID: u32, T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<MultiPolygon<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::MultiPolygon, cursor)?;
    check_srid(g_header.srid, SRID)?;
    read_multi_polygon_body::<SRID, T, P>(g_header.g_type, cursor)
}

pub fn read_multi_polygon_body<const SRID: u32, T, P>(
    g_type: u32,
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<MultiPolygon<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let polygons_n = cursor.read_u32::<T>()?;
    let mut polygon = MultiPolygon::new();

    for _i in 0..polygons_n {
        // skip 1 byte for byte order and 4 bytes for point type
        cursor.read_u8()?;
        cursor.read_u32::<T>()?;
        polygon
            .polygons
            .push(read_polygon_body::<SRID, T, P>(g_type, cursor)?);
    }
    Ok(polygon)
}
