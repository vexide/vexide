use super::Point2;

impl<T: nalgebra::Scalar> From<nalgebra::Point2<T>> for Point2<T> {
    fn from(na_point: nalgebra::Point2<T>) -> Self {
        // nalgebra appears to have a highly optimized conversion to mint.
        let coords: mint::Point2<T> = na_point.into();
        Self {
            x: coords.x,
            y: coords.y,
        }
    }
}

impl<T: nalgebra::Scalar> From<Point2<T>> for nalgebra::Point2<T> {
    fn from(point: Point2<T>) -> Self {
        Self::new(point.x, point.y)
    }
}
