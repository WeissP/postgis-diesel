use crate::{
    error::check_srid,
    ewkb::{read_ewkb_header, write_ewkb_header, EwkbSerializable, GeometryType, BIG_ENDIAN},
    points::{read_point_coordinates, write_point_coordinates, Dimension},
    sql_types::*,
    types::{LineString, PointT, Polygon},
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use diesel::{
    deserialize::{self, FromSql},
    pg::{self, Pg},
    serialize::{self, IsNull, Output, ToSql},
};
use std::{fmt::Debug, io::Cursor, iter::FromIterator};

impl<const SRID: u32, P: PointT<SRID>> Default for Polygon<SRID, P> {
    fn default() -> Self {
        Self {
            rings: vec![LineString::default()],
        }
    }
}

impl<const SRID: u32, P: PointT<SRID>> FromIterator<LineString<SRID, P>> for Polygon<SRID, P> {
    fn from_iter<T: IntoIterator<Item = LineString<SRID, P>>>(iter: T) -> Self {
        let rings = iter.into_iter().collect();
        Self { rings }
    }
}

impl<const SRID: u32, T> Polygon<SRID, T>
where
    T: PointT<SRID> + Clone,
{
    pub fn new() -> Self {
        Polygon { rings: Vec::new() }
    }

    pub fn add_ring<'a>(&'a mut self) -> &mut Self {
        self.rings.push(LineString::default());
        self
    }

    pub fn add_point<'a>(&'a mut self, point: T) -> &mut Self {
        if self.rings.last().is_none() {
            self.add_ring();
        }
        self.rings.last_mut().unwrap().points.push(point);
        self
    }

    pub fn add_points<'a>(&'a mut self, points: &[T]) -> &mut Self {
        if self.rings.last().is_none() {
            self.add_ring();
        }
        let last = self.rings.last_mut().unwrap();
        for point in points {
            last.points.push(point.to_owned());
        }
        self
    }

    pub fn dimension(&self) -> u32 {
        let mut dimension = Dimension::None as u32;
        if let Some(ring) = self.rings.first() {
            if let Some(point) = ring.points.first() {
                dimension |= point.dimension();
            }
        }
        dimension
    }
}

impl<const SRID: u32, T> EwkbSerializable for Polygon<SRID, T>
where
    T: PointT<SRID> + Clone,
{
    fn geometry_type(&self) -> u32 {
        GeometryType::Polygon as u32 | self.dimension()
    }
}

impl<const SRID: u32, T> ToSql<Geometry, Pg> for Polygon<SRID, T>
where
    T: PointT<SRID> + Debug + PartialEq + Clone + EwkbSerializable,
{
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        write_polygon(self, out)
    }
}

pub fn write_polygon<const SRID: u32, T>(
    polygon: &Polygon<SRID, T>,
    out: &mut Output<Pg>,
) -> serialize::Result
where
    T: PointT<SRID> + EwkbSerializable + Clone,
{
    write_ewkb_header(polygon, Some(SRID), out)?;
    // number of rings
    out.write_u32::<LittleEndian>(polygon.rings.len() as u32)?;
    for ring in polygon.rings.iter() {
        //number of points in ring
        out.write_u32::<LittleEndian>(ring.points.len() as u32)?;
        for point in ring.points.iter() {
            write_point_coordinates(point, out)?;
        }
    }
    Ok(IsNull::No)
}

impl<const SRID: u32, T> FromSql<Geometry, Pg> for Polygon<SRID, T>
where
    T: PointT<SRID> + Debug + Clone,
{
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        let mut r = Cursor::new(bytes.as_bytes());
        let end = r.read_u8()?;
        if end == BIG_ENDIAN {
            read_polygon::<SRID, BigEndian, T>(&mut r)
        } else {
            read_polygon::<SRID, LittleEndian, T>(&mut r)
        }
    }
}

fn read_polygon<const SRID: u32, T, P>(
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<Polygon<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let g_header = read_ewkb_header::<T>(GeometryType::Polygon, cursor)?;
    check_srid(g_header.srid, SRID)?;
    read_polygon_body::<SRID, T, P>(g_header.g_type, cursor)
}

pub fn read_polygon_body<const SRID: u32, T, P>(
    g_type: u32,
    cursor: &mut Cursor<&[u8]>,
) -> deserialize::Result<Polygon<SRID, P>>
where
    T: byteorder::ByteOrder,
    P: PointT<SRID> + Clone,
{
    let rings_n = cursor.read_u32::<T>()?;
    let mut polygon = Polygon::new();
    for _i in 0..rings_n {
        polygon.add_ring();
        let points_n = cursor.read_u32::<T>()?;
        for _p in 0..points_n {
            polygon.add_point(read_point_coordinates::<SRID, T, P>(cursor, g_type)?);
        }
    }
    Ok(polygon)
}
